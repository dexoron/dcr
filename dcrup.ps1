#Requires -Version 5.1
$ErrorActionPreference = 'Stop'

$DcrupHome = if ($env:DCRUP_HOME) { $env:DCRUP_HOME } else { Join-Path $env:USERPROFILE '.dcr' }
$DcrupBin  = if ($env:DCRUP_BIN)  { $env:DCRUP_BIN  } else { Join-Path $DcrupHome 'bin' }
$DcrupTc   = Join-Path $DcrupHome 'toolchains'
$DcrupMeta = Join-Path $DcrupHome 'meta'
$RepoUrl   = if ($env:DCRUP_REPO) { $env:DCRUP_REPO } else { 'https://github.com/dexoron/dcr' }
$ApiLatest = 'https://api.github.com/repos/dexoron/dcr/releases/latest'
$ApiAll    = 'https://api.github.com/repos/dexoron/dcr/releases'
$Features  = if ($env:DCR_FEATURES) { $env:DCR_FEATURES } else { 'archive' }

function Info($m) { Write-Host "[dcrup] $m" -ForegroundColor Cyan }
function Ok($m)   { Write-Host "[ok] $m" -ForegroundColor Green }
function Warn($m) { Write-Host "[warn] $m" -ForegroundColor Yellow }
function Die($m)  { Write-Host "[error] $m" -ForegroundColor Red; exit 1 }

function Show-Usage {
    @'
dcrup — install and switch DCR versions (Windows)

Usage:
  dcrup install <spec> [--build|--release] [--force]
  dcrup default <spec>
  dcrup update
  dcrup list
  dcrup show
  dcrup which
  dcrup uninstall [<spec>|--all]
  dcrup self-install [--to DIR]
  dcrup help

Spec:
  stable | dev | night
  VERSION              → VERSION@stable  (e.g. 0.8.2)
  VERSION@stable
  VERSION@dev
  night                → always build from branch "dev" HEAD

Modes:
  --release   download GitHub Release binary (default for stable/dev)
  --build     cargo build --release --features archive
  night       always --build (prebuilt not available)

Env:
  DCRUP_HOME      default ~/.dcr
  DCRUP_BIN       default $DCRUP_HOME/bin
  DCR_FEATURES    default archive
  DCRUP_REPO      git/GitHub repo URL
'@ | Write-Host
}

function Ensure-Dirs {
    New-Item -ItemType Directory -Force -Path $DcrupTc, $DcrupBin, $DcrupMeta | Out-Null
}

function Get-Target {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($env:PROCESSOR_ARCHITEW6432) { $arch = $env:PROCESSOR_ARCHITEW6432 }
    switch ($arch) {
        'AMD64' { return 'x86_64-pc-windows-msvc' }
        'ARM64' { return 'aarch64-pc-windows-msvc' }
        'x86'   { return 'i686-pc-windows-msvc' }
        default { Die "unsupported arch: $arch" }
    }
}

function Normalize-Version([string]$v) {
    if ($v.StartsWith('v')) { return $v.Substring(1) }
    return $v
}

function Parse-Spec([string]$raw) {
    $script:SpecRaw = $raw
    $script:SpecChannel = $null
    $script:SpecVersion = $null
    $script:SpecFloating = $false

    if ($raw -in @('stable', 'dev', 'night')) {
        $script:SpecChannel = $raw
        $script:SpecFloating = $true
        return
    }
    if ($raw -match '^(.*)@(.*)$') {
        $script:SpecVersion = Normalize-Version $Matches[1]
        $script:SpecChannel = $Matches[2]
        if ($script:SpecChannel -eq 'night') { Die 'cannot pin version on night' }
        if ($script:SpecChannel -notin @('stable', 'dev')) { Die "unknown channel: $($script:SpecChannel)" }
        return
    }
    $script:SpecVersion = Normalize-Version $raw
    $script:SpecChannel = 'stable'
}

function Toolchain-Id([string]$extra = '') {
    if ($script:SpecChannel -eq 'night') {
        return "night-$extra"
    }
    if ($script:SpecFloating) {
        return "$($script:SpecChannel)-$extra"
    }
    return "$($script:SpecVersion)-$($script:SpecChannel)"
}

function Active-Path { Join-Path $DcrupMeta 'active' }

function Read-Active {
    $p = Active-Path
    if (Test-Path $p) { return (Get-Content $p -Raw).Trim() }
    return $null
}

function Write-Active([string]$id) {
    Set-Content -Path (Active-Path) -Value $id -NoNewline
}

function Link-Default([string]$id) {
    $src = Join-Path (Join-Path $DcrupTc $id) 'dcr.exe'
    if (-not (Test-Path $src)) { Die "toolchain not installed: $id" }
    $dst = Join-Path $DcrupBin 'dcr.exe'
    Copy-Item $src $dst -Force
    Write-Active $id
    Ok "default → $id ($dst)"
}

function Fetch-Release($channel, $version) {
    if ($version) {
        return Invoke-RestMethod -Uri "https://api.github.com/repos/dexoron/dcr/releases/tags/v$version"
    }
    if ($channel -eq 'dev') {
        $all = Invoke-RestMethod -Uri $ApiAll
        $pre = $all | Where-Object { $_.prerelease } | Select-Object -First 1
        if (-not $pre) { Die 'no prerelease (dev) found' }
        return $pre
    }
    return Invoke-RestMethod -Uri $ApiLatest
}

function Install-Prebuilt([switch]$Force) {
    $triple = Get-Target
    $rel = Fetch-Release $script:SpecChannel $script:SpecVersion
    $tag = $rel.tag_name
    $ver = Normalize-Version $tag
    $assetName = "dcr-$triple-$ver.exe"
    $asset = $rel.assets | Where-Object { $_.name -eq $assetName } | Select-Object -First 1
    if (-not $asset) { Die "asset not found: $assetName" }

    $id = if ($script:SpecFloating) { Toolchain-Id $ver } else { Toolchain-Id }
    $dest = Join-Path $DcrupTc $id
    $bin = Join-Path $dest 'dcr.exe'
    if ((Test-Path $bin) -and -not $Force) {
        Ok "already installed: $id"
        Link-Default $id
        return
    }
    Ensure-Dirs
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    Info "downloading $assetName …"
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $bin
    @"
channel=$($script:SpecChannel)
version=$ver
tag=$tag
source=release
triple=$triple
"@ | Set-Content (Join-Path $dest 'dcrup-meta')
    Ok "installed prebuilt $id"
    Link-Default $id
}

function Install-Build([switch]$Force) {
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) { Die 'git not found' }
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Die 'cargo not found' }

    $tmp = Join-Path $env:TEMP ("dcrup-" + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmp | Out-Null
    try {
        $src = Join-Path $tmp 'src'
        if ($script:SpecChannel -eq 'night') {
            Info "cloning dev for night …"
            git clone --depth 1 --branch dev $RepoUrl $src 2>$null
            if ($LASTEXITCODE -ne 0) { git clone --depth 1 $RepoUrl $src | Out-Null }
            $sha = (git -C $src rev-parse --short HEAD).Trim()
            $id = Toolchain-Id $sha
            $ver = 'night'
        } elseif ($script:SpecFloating) {
            $rel = Fetch-Release $script:SpecChannel $null
            $tag = $rel.tag_name
            $ver = Normalize-Version $tag
            $id = Toolchain-Id $ver
            Info "cloning $tag …"
            git clone --depth 1 --branch $tag $RepoUrl $src 2>$null
            if ($LASTEXITCODE -ne 0) {
                git clone --depth 1 $RepoUrl $src | Out-Null
                git -C $src fetch --depth 1 origin "refs/tags/${tag}:refs/tags/${tag}"
                git -C $src checkout $tag
            }
            $sha = (git -C $src rev-parse --short HEAD).Trim()
        } else {
            $tag = "v$($script:SpecVersion)"
            $ver = $script:SpecVersion
            $id = Toolchain-Id
            Info "cloning $tag …"
            git clone --depth 1 --branch $tag $RepoUrl $src 2>$null
            if ($LASTEXITCODE -ne 0) {
                git clone --depth 1 $RepoUrl $src | Out-Null
                git -C $src fetch --depth 1 origin "refs/tags/${tag}:refs/tags/${tag}"
                git -C $src checkout $tag
            }
            $sha = (git -C $src rev-parse --short HEAD).Trim()
        }

        $dest = Join-Path $DcrupTc $id
        $bin = Join-Path $dest 'dcr.exe'
        if ((Test-Path $bin) -and -not $Force) {
            Ok "already installed: $id"
            Link-Default $id
            return
        }

        $triple = Get-Target
        Info "building release (features: $Features) …"
        Push-Location $src
        cargo build --release --features $Features --target $triple
        if ($LASTEXITCODE -ne 0) { Die 'cargo build failed' }
        Pop-Location

        New-Item -ItemType Directory -Force -Path $dest | Out-Null
        Copy-Item (Join-Path $src "target\$triple\release\dcr.exe") $bin -Force
        @"
channel=$($script:SpecChannel)
version=$ver
git=$sha
source=build
features=$Features
"@ | Set-Content (Join-Path $dest 'dcrup-meta')
        Ok "installed build $id"
        Link-Default $id
    } finally {
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
}

function Ensure-UserPath {
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not $userPath) { $userPath = '' }
    if (-not ($userPath -split ';' | Where-Object { $_ -eq $DcrupBin })) {
        $new = if ($userPath) { "$userPath;$DcrupBin" } else { $DcrupBin }
        [Environment]::SetEnvironmentVariable('Path', $new, 'User')
        $env:Path = "$env:Path;$DcrupBin"
        Warn "added to user PATH: $DcrupBin (restart shell if needed)"
    }
}

function Install-CmdShim {
    $cmd = Join-Path $DcrupBin 'dcrup.cmd'
    $ps1 = Join-Path $DcrupBin 'dcrup.ps1'
    @"
@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0dcrup.ps1" %*
"@ | Set-Content -Path $cmd -Encoding ASCII
    if (-not (Test-Path $ps1)) {
        Copy-Item $PSCommandPath $ps1 -Force
    }
}

function Cmd-Install {
    param([string[]]$Args)
    $mode = 'release'
    $force = $false
    $spec = $null
    foreach ($a in $Args) {
        switch -Regex ($a) {
            '^--build$' { $mode = 'build'; continue }
            '^--release$' { $mode = 'release'; continue }
            '^--force$' { $force = $true; continue }
            '^-' { Die "unknown flag: $a" }
            default {
                if ($spec) { Die "unexpected: $a" }
                $spec = $a
            }
        }
    }
    if (-not $spec) { Die 'usage: dcrup install <spec> [--build|--release] [--force]' }
    Parse-Spec $spec
    Ensure-Dirs
    if ($script:SpecChannel -eq 'night') { $mode = 'build' }
    Info "install spec=$spec channel=$($script:SpecChannel) mode=$mode"
    if ($mode -eq 'build') { Install-Build -Force:$force } else { Install-Prebuilt -Force:$force }
    Ensure-UserPath
    Install-CmdShim
}

function Cmd-List {
    Ensure-Dirs
    $active = Read-Active
    Get-ChildItem $DcrupTc -Directory -ErrorAction SilentlyContinue | ForEach-Object {
        $mark = if ($_.Name -eq $active) { ' *' } else { '  ' }
        Write-Host "$mark $($_.Name)"
    }
}

function Cmd-Show {
    Ensure-Dirs
    Write-Host "DCRUP_HOME=$DcrupHome"
    Write-Host "DCRUP_BIN=$DcrupBin"
    Write-Host "active=$(Read-Active)"
    $exe = Join-Path $DcrupBin 'dcr.exe'
    if (Test-Path $exe) { & $exe --version }
}

function Cmd-SelfInstall {
    Ensure-Dirs
    Copy-Item $PSCommandPath (Join-Path $DcrupBin 'dcrup.ps1') -Force
    Install-CmdShim
    Ensure-UserPath
    Ok "dcrup installed to $DcrupBin (use: dcrup …)"
}

$cmd = if ($args.Count -ge 1) { $args[0] } else { 'help' }
$rest = @()
if ($args.Count -gt 1) { $rest = $args[1..($args.Count - 1)] }

switch ($cmd) {
    { $_ -in @('help', '-h', '--help') } { Show-Usage }
    'install' { Cmd-Install $rest }
    'list' { Cmd-List }
    'show' { Cmd-Show }
    'which' {
        $p = Join-Path $DcrupBin 'dcr.exe'
        if (Test-Path $p) { $p } else { Die 'dcr not installed' }
    }
    'default' {
        if ($rest.Count -ne 1) { Die 'usage: dcrup default <spec>' }
        Parse-Spec $rest[0]
        if ($script:SpecFloating -or $script:SpecChannel -eq 'night') {
            $id = Get-ChildItem $DcrupTc -Directory |
                Where-Object { $_.Name -like "$($script:SpecChannel)-*" } |
                Sort-Object Name | Select-Object -Last 1
            if (-not $id) { Die "not installed: $($rest[0])" }
            Link-Default $id.Name
        } else {
            Link-Default (Toolchain-Id)
        }
    }
    'update' {
        $a = Read-Active
        if (-not $a) { Die 'no active toolchain' }
        if ($a -like 'night-*') { Cmd-Install @('night', '--build', '--force') }
        elseif ($a -like 'stable-*') { Cmd-Install @('stable', '--force') }
        elseif ($a -like 'dev-*') { Cmd-Install @('dev', '--force') }
        else { Ok "pinned $a — install a newer version explicitly" }
    }
    'uninstall' {
        if ($rest -contains '--all') {
            Remove-Item -Recurse -Force $DcrupTc -ErrorAction SilentlyContinue
            Remove-Item (Join-Path $DcrupBin 'dcr.exe') -ErrorAction SilentlyContinue
            Remove-Item (Active-Path) -ErrorAction SilentlyContinue
            Ok 'removed all'
        } else {
            Die 'use: dcrup uninstall --all  (or install a replacement first)'
        }
    }
    'self-install' { Cmd-SelfInstall }
    default { Die "unknown command: $cmd" }
}
