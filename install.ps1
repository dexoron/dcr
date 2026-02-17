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
        Fail "Не найдено: $name"
    }
}

function Select-Mode {
    Write-Host "Выбери способ установки:"
    Write-Host "  1) Скачать готовый бинарник из GitHub Release (рекомендуется)"
    Write-Host "  2) Собрать из git"
    $choice = Read-Host "Введите 1 или 2 [1]"
    if ([string]::IsNullOrWhiteSpace($choice)) { $choice = '1' }

    switch ($choice) {
        '1' { return 'release' }
        '2' { return 'build' }
        default { Fail 'Неизвестный вариант' }
    }
}

function Get-Target {
    if ($env:PROCESSOR_ARCHITECTURE -ne 'AMD64') {
        Fail "Поддерживается только x86_64 Windows, текущая архитектура: $env:PROCESSOR_ARCHITECTURE"
    }
    return 'x86_64-pc-windows-msvc'
}

function Install-FromRelease {
    param([string]$target)

    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null

    $assetName = "dcr-$target.exe"
    Info "Получение последнего релиза..."
    $release = Invoke-RestMethod -Method Get -Uri $ApiUrl

    $asset = $release.assets | Where-Object { $_.name -eq $assetName } | Select-Object -First 1
    if (-not $asset) {
        Fail "Не найден ассет $assetName в релизе $($release.tag_name)"
    }

    Info "Скачивание $assetName..."
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $BinaryPath
    Ok "Скачан бинарник $assetName"
}

function Install-FromSource {
    param([string]$target)

    Require-Cmd 'git'
    Require-Cmd 'cargo'

    $tmp = Join-Path $env:TEMP 'dcr-install'
    if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }

    Info 'Клонирование репозитория...'
    git clone --depth 1 $RepoUrl $tmp | Out-Null

    Info 'Сборка release-бинарника...'
    Push-Location $tmp
    cargo build --release --target $target
    Pop-Location

    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
    Copy-Item (Join-Path $tmp "target\$target\release\dcr.exe") $BinaryPath -Force
    Ok 'Установлен бинарник из исходников'

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
        Warn "Добавлен путь в PATH: $BinPath"
    }

    Ok "Команда dcr доступна через $linkPath"
}

Info 'Запуск установки DCR'
$mode = Select-Mode
$target = Get-Target

if ($mode -eq 'build') {
    Install-FromSource -target $target
} else {
    Install-FromRelease -target $target
}

Setup-Path
Ok 'Установка завершена успешно'
