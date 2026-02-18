$ErrorActionPreference = 'Stop'

$RepoUrl = 'https://github.com/dexoron/dcr'
$ApiUrl = 'https://api.github.com/repos/dexoron/dcr/releases/latest'
$InstallPath = Join-Path $env:LOCALAPPDATA 'dcr'
$BinPath = Join-Path $env:USERPROFILE '.local\bin'
$BinaryPath = Join-Path $InstallPath 'dcr.exe'

function Info($msg) { Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Ok($msg) { Write-Host "[OK] $msg" -ForegroundColor Green }
function Warn($msg) { Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Fail($msg) { Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }

function Require-Cmd($name) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        Fail "Not found: $name"
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
        '2' { return 'build' }
        default { Fail 'Unknown option' }
    }
}

function Get-Target {
    if ($env:PROCESSOR_ARCHITECTURE -ne 'AMD64') {
        Fail "Only x86_64 Windows is supported, current architecture: $env:PROCESSOR_ARCHITECTURE"
    }
    return 'x86_64-pc-windows-msvc'
}

function Install-FromRelease {
    param([string]$target)

    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null

    $assetName = "dcr-$target.exe"
    Info "Fetching latest release..."
    $release = Invoke-RestMethod -Method Get -Uri $ApiUrl

    $asset = $release.assets | Where-Object { $_.name -eq $assetName } | Select-Object -First 1
    if (-not $asset) {
        Fail "Asset $assetName not found in release $($release.tag_name)"
    }

    Info "Downloading $assetName..."
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $BinaryPath
    Ok "Binary downloaded: $assetName"
}

function Install-FromSource {
    param([string]$target)

    Require-Cmd 'git'
    Require-Cmd 'cargo'

    $tmp = Join-Path $env:TEMP 'dcr-install'
    if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }

    Info 'Cloning repository...'
    git clone --depth 1 $RepoUrl $tmp | Out-Null

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
$mode = Select-Mode
$target = Get-Target

if ($mode -eq 'build') {
    Install-FromSource -target $target
} else {
    Install-FromRelease -target $target
}

Setup-Path
Ok 'Installation completed successfully'
