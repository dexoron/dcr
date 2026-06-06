$ErrorActionPreference = 'Stop'

$RepoUrl    = 'https://github.com/dexoron/dcr'
$ApiLatest  = 'https://api.github.com/repos/dexoron/dcr/releases/latest'
$ApiAll     = 'https://api.github.com/repos/dexoron/dcr/releases'
$InstallPath = Join-Path $env:LOCALAPPDATA 'dcr'
$BinPath     = Join-Path $env:USERPROFILE '.local\bin'
$BinaryPath  = Join-Path $InstallPath 'dcr.exe'

function Info($msg)  { Write-Host "[INFO] $msg"  -ForegroundColor Cyan   }
function Ok($msg)    { Write-Host "[OK] $msg"    -ForegroundColor Green  }
function Warn($msg)  { Write-Host "[WARN] $msg"  -ForegroundColor Yellow }
function Fail($msg)  { Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }

function Require-Cmd($name) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        Fail "Not found: $name"
    }
}

function Select-Channel {
    Write-Host "Choose channel:"
    Write-Host "  1) Latest stable release (default)"
    Write-Host "  2) Latest dev (pre-release)"
    $choice = Read-Host "Enter 1 or 2 [1]"
    if ([string]::IsNullOrWhiteSpace($choice)) { $choice = '1' }

    switch ($choice) {
        '1' { return 'stable' }
        '2' { return 'dev'    }
        default { Fail 'Unknown option' }
    }
}

function Select-Mode {
    Write-Host "Choose installation mode:"
    Write-Host "  1) Download prebuilt binary from GitHub Release (recommended)"
    Write-Host "  2) Build from git"
    $choice = Read-Host "Enter 1 or 2 [1]"
    if ([string]::IsNullOrWhiteSpace($choice)) { $choice = '1' }

    switch ($choice) {
        '1' { return 'release' }
        '2' { return 'build'   }
        default { Fail 'Unknown option' }
    }
}

function Get-Target {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($env:PROCESSOR_ARCHITEW6432) {
        $arch = $env:PROCESSOR_ARCHITEW6432
    }
    switch ($arch) {
        'AMD64' { return 'x86_64-pc-windows-msvc' }
        'ARM64' { return 'aarch64-pc-windows-msvc' }
        'x86'   { return 'i686-pc-windows-msvc' }
        default { Fail "Unsupported Windows architecture: $arch" }
    }
}

function Fetch-ReleaseJson($channel) {
    if ($channel -eq 'dev') {
        Info "Looking for latest dev (pre-release)..."
        $releases = Invoke-RestMethod -Method Get -Uri $ApiAll
        $pre = $releases | Where-Object { $_.prerelease -eq $true } | Select-Object -First 1
        if (-not $pre) { Fail "No dev (pre-release) found on GitHub" }
        return $pre
    } else {
        return Invoke-RestMethod -Method Get -Uri $ApiLatest
    }
}

function Install-FromRelease {
    param([string]$target, [string]$channel)

    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null

    $release = Fetch-ReleaseJson $channel
    $tag     = $release.tag_name
    $version = $tag.TrimStart('v')

    # Имя бинарника: dcr-<triple>-<version>.exe
    $assetName = "dcr-$target-$version.exe"

    Info "Fetching release $tag (channel: $channel)..."

    $asset = $release.assets | Where-Object { $_.name -eq $assetName } | Select-Object -First 1
    if (-not $asset) {
        Fail "Asset $assetName not found in release $tag"
    }

    Info "Downloading $assetName..."
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $BinaryPath
    Ok "Binary downloaded: $assetName"
}

function Install-FromSource {
    param([string]$target, [string]$channel)

    Require-Cmd 'git'
    Require-Cmd 'cargo'

    $tmp = Join-Path $env:TEMP 'dcr-install'
    if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }

    Info 'Cloning repository...'
    if ($channel -eq 'dev') {
        # Пробуем ветку dev
        $result = git clone --depth 1 --branch dev $RepoUrl $tmp 2>&1
        if ($LASTEXITCODE -ne 0) {
            git clone --depth 1 $RepoUrl $tmp | Out-Null
        }
    } else {
        git clone --depth 1 $RepoUrl $tmp | Out-Null
    }

    Info 'Building release binary...'
    Push-Location $tmp
    cargo build --release --target $target
    Pop-Location

    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
    Copy-Item (Join-Path $tmp "target\$target\release\dcr.exe") $BinaryPath -Force
    Ok 'Binary installed from source'

    Remove-Item -Recurse -Force $tmp
}

function Setup-Path {
    New-Item -ItemType Directory -Path $BinPath -Force | Out-Null
    $linkPath = Join-Path $BinPath 'dcr.exe'
    Copy-Item $BinaryPath $linkPath -Force

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not $userPath) { $userPath = '' }

    if (-not ($userPath -split ';' | Where-Object { $_ -eq $BinPath })) {
        $newPath = if ($userPath) { "$userPath;$BinPath" } else { $BinPath }
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        $env:Path = "$env:Path;$BinPath"
        Warn "Added path to PATH: $BinPath"
    }

    Ok "dcr command is available via $linkPath"
}

Info 'Starting DCR installation'
$channel = Select-Channel
$mode    = Select-Mode
$target  = Get-Target

if ($mode -eq 'build') {
    Install-FromSource -target $target -channel $channel
} else {
    Install-FromRelease -target $target -channel $channel
}

Setup-Path
Ok 'Installation completed successfully'