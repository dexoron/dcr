#Requires -Version 5.1
# dcrup - install and switch DCR versions (Windows PowerShell 5.1+ / PowerShell 7+)
#
# Quick install (interactive or unattended, no arguments needed):
#   irm https://ps1.dcr-tool.ru | iex
# With arguments:
#   & ([scriptblock]::Create((irm https://ps1.dcr-tool.ru))) install stable --quiet
# Unattended / corporate (environment variables also work with `irm | iex`):
#   $env:DCRUP_VERSION='0.9.5'; $env:DCRUP_NO_MODIFY_PATH='1'; irm https://ps1.dcr-tool.ru | iex
#
# This file is ASCII-only on purpose: Windows PowerShell 5.1 reads BOM-less
# files in the ANSI code page, which corrupts non-ASCII characters.

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'   # Invoke-WebRequest is very slow with progress on PS 5.1

$script:DcrupVersion = '2.0.0'

# ---------------------------------------------------------------- settings --

function Get-EnvFlag([string]$name) {
    $v = [Environment]::GetEnvironmentVariable($name)
    return ($v -and $v -notin @('0', 'false', 'no', 'off'))
}

$script:System      = Get-EnvFlag 'DCRUP_SYSTEM'
$script:Quiet       = Get-EnvFlag 'DCRUP_QUIET'
$script:NoModifyPath = Get-EnvFlag 'DCRUP_NO_MODIFY_PATH'
$script:Mirror      = if ($env:DCRUP_MIRROR) { $env:DCRUP_MIRROR } else { 'auto' }
$script:TargetPref  = if ($env:DCRUP_TARGET) { $env:DCRUP_TARGET } else { 'auto' }
$script:RepoUrl     = if ($env:DCRUP_REPO) { $env:DCRUP_REPO } else { 'https://github.com/dexoron/dcr' }
$script:InstallUrl  = if ($env:DCRUP_INSTALL_URL) { $env:DCRUP_INSTALL_URL } else { 'https://ps1.dcr-tool.ru' }
$script:Features    = if ($env:DCR_FEATURES) { $env:DCR_FEATURES } else { 'archive' }
$script:ApiBase     = 'https://api.github.com/repos/dexoron/dcr'
$script:Mirrors     = @('cf', 'gh', 'sf', 'ya')

function Set-Layout {
    if ($env:DCRUP_HOME) {
        $script:Home_ = $env:DCRUP_HOME
    } elseif ($script:System) {
        $script:Home_ = Join-Path $env:ProgramData 'dcr'
    } else {
        $script:Home_ = Join-Path $env:USERPROFILE '.dcr'
    }
    $script:Home_ = [IO.Path]::GetFullPath($script:Home_)
    $script:Bin  = if ($env:DCRUP_BIN) { [IO.Path]::GetFullPath($env:DCRUP_BIN) } else { Join-Path $script:Home_ 'bin' }
    $script:Tc   = Join-Path $script:Home_ 'toolchains'
    $script:Meta = Join-Path $script:Home_ 'meta'
}

# ----------------------------------------------------------------- output --

function Info($m) { if (-not $script:Quiet) { Write-Host "[dcrup] $m" -ForegroundColor Cyan } }
function Ok($m)   { if (-not $script:Quiet) { Write-Host "[ok] $m" -ForegroundColor Green } }
function Warn($m) { Write-Host "[warn] $m" -ForegroundColor Yellow }
# Never call `exit` here: under `irm | iex` it would close the user's shell.
function Die($m)  {
    $e = New-Object System.InvalidOperationException ([string]$m)
    $e.Data['dcrup'] = $true
    throw $e
}

# Runs a native command with stderr shown but not treated as a PowerShell error
# (Windows PowerShell 5.1 turns redirected stderr into terminating errors).
function Invoke-Native([string]$exe, [string[]]$argv) {
    $ErrorActionPreference = 'Continue'
    & $exe @argv 2>&1 | ForEach-Object { if (-not $script:Quiet) { Write-Host "  $_" } }
    return ($LASTEXITCODE -eq 0)
}

function Show-Usage {
    @"
dcrup $($script:DcrupVersion) - install and switch DCR versions (Windows)

Usage:
  dcrup install <spec> [options]   install a toolchain and make it default
  dcrup default <spec>             switch the default toolchain
  dcrup update                     update the default floating toolchain
  dcrup list                       list installed toolchains (* = default)
  dcrup show                       show layout and the default toolchain
  dcrup which                      print the path of the active dcr.exe
  dcrup uninstall <spec>|--all     remove toolchain(s)
  dcrup self-install               install dcrup itself and add it to PATH
  dcrup self-uninstall             remove dcrup, all toolchains and PATH entry
  dcrup help | --version

  Run without arguments (e.g. 'irm https://ps1.dcr-tool.ru | iex'):
  self-install + install `$env:DCRUP_VERSION (default: stable).

Spec:
  stable | dev | night | VERSION (= VERSION@stable) | VERSION@stable | VERSION@dev

Install options:
  --force                reinstall even if present
  --build                build from source with cargo (night always builds)
  --mirror auto|cf|gh|sf|ya   download host (auto = cf, gh, sf, ya)
  --target auto|msvc|gnu      Windows ABI of the prebuilt binary
  --no-modify-path       do not touch PATH
  --system               machine-wide install to %ProgramData%\dcr (admin)
  -q, --quiet            only warnings and errors
  -y, --yes              accepted for compatibility; dcrup never prompts

Environment (same meaning as the options; work with 'irm | iex'):
  DCRUP_VERSION          spec used when run without arguments (default: stable)
  DCRUP_HOME             default %USERPROFILE%\.dcr (or %ProgramData%\dcr with --system)
  DCRUP_BIN              default DCRUP_HOME\bin
  DCRUP_MIRROR           auto|cf|gh|sf|ya
  DCRUP_TARGET           auto|msvc|gnu
  DCRUP_SYSTEM=1, DCRUP_QUIET=1, DCRUP_NO_MODIFY_PATH=1
  DCRUP_PROXY            explicit proxy URL (default: system proxy, current user credentials)
  GITHUB_TOKEN           optional, raises GitHub API rate limit
  DCRUP_REPO, DCR_FEATURES    used by --build / night

Exit code is 0 on success and 1 on error.
"@ | Write-Host
}

# ---------------------------------------------------------------- network --

function Initialize-Network {
    if ($PSVersionTable.PSVersion.Major -lt 6) {
        # Windows PowerShell 5.1 on older systems may not offer TLS 1.2,
        # which GitHub and the CDNs require.
        try {
            [Net.ServicePointManager]::SecurityProtocol =
                [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        } catch {}
    }
    $script:WebArgs = @{
        UseBasicParsing = $true
        UserAgent       = "dcrup/$($script:DcrupVersion)"
        TimeoutSec      = 300
    }
    if ($env:DCRUP_PROXY) {
        $script:WebArgs.Proxy = $env:DCRUP_PROXY
        $script:WebArgs.ProxyUseDefaultCredentials = $true
    } else {
        # Corporate proxies usually need the logged-on user's credentials.
        try { [Net.WebRequest]::DefaultWebProxy.Credentials = [Net.CredentialCache]::DefaultNetworkCredentials } catch {}
    }
}

function Get-HttpStatus($err) {
    try { return [int]$err.Exception.Response.StatusCode } catch { return 0 }
}

# Returns the parsed JSON, $null on 404, throws on other failures.
function Invoke-Api([string]$path) {
    $headers = @{ Accept = 'application/vnd.github+json' }
    if ($env:GITHUB_TOKEN) { $headers.Authorization = "Bearer $($env:GITHUB_TOKEN)" }
    try {
        return Invoke-RestMethod -Uri "$($script:ApiBase)$path" -Headers $headers @script:WebArgs
    } catch {
        $code = Get-HttpStatus $_
        if ($code -eq 404) { return $null }
        if ($code -in @(403, 429)) { Die 'GitHub API rate limit exceeded; set GITHUB_TOKEN or retry later' }
        Die "GitHub API request failed ($path): $($_.Exception.Message)"
    }
}

# ---------------------------------------------------------------- platform --

function Get-Arch {
    $a = $null
    try { $a = [string][Runtime.InteropServices.RuntimeInformation]::OSArchitecture } catch {}
    if (-not $a) {
        $a = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
    }
    switch ($a) {
        { $_ -in @('X64', 'AMD64') } { return 'x86_64' }
        'Arm64'                      { return 'aarch64' }
        { $_ -in @('X86', 'x86') }   { return 'i686' }
        default                      { Die "unsupported CPU architecture: $a" }
    }
}

function Test-VcRuntime([string]$arch) {
    $win = $env:windir
    $is64os = [Environment]::Is64BitOperatingSystem
    if ($arch -eq 'i686' -and $is64os) {
        $dir = Join-Path $win 'SysWOW64'
    } elseif ($is64os -and -not [Environment]::Is64BitProcess) {
        $dir = Join-Path $win 'Sysnative'
    } else {
        $dir = Join-Path $win 'System32'
    }
    return (Test-Path (Join-Path $dir 'vcruntime140.dll'))
}

# Picks the release triple. msvc builds need the Visual C++ runtime
# (vcruntime140.dll); x86_64 also has a self-contained gnu build.
function Get-Target {
    $arch = Get-Arch
    $pref = $script:TargetPref
    if ($pref -notin @('auto', 'msvc', 'gnu')) { Die "invalid --target: $pref (use auto|msvc|gnu)" }
    if ($pref -eq 'gnu') {
        if ($arch -ne 'x86_64') { Die "gnu build is only published for x86_64 (this machine: $arch)" }
        return 'x86_64-pc-windows-gnu'
    }
    if ($pref -eq 'auto' -and -not (Test-VcRuntime $arch)) {
        if ($arch -eq 'x86_64') {
            Info 'Visual C++ runtime not found; using the self-contained gnu build'
            return 'x86_64-pc-windows-gnu'
        }
        Warn 'Visual C++ runtime (vcruntime140.dll) not found; dcr.exe may not start.'
        Warn 'Install it from https://aka.ms/vs/17/release/vc_redist.x64.exe (or the arm64/x86 variant).'
    }
    switch ($arch) {
        'x86_64'  { return 'x86_64-pc-windows-msvc' }
        'aarch64' { return 'aarch64-pc-windows-msvc' }
        'i686'    { return 'i686-pc-windows-msvc' }
    }
}

# -------------------------------------------------------------------- spec --

function Normalize-Version([string]$v) {
    $v = $v.Trim()
    if ($v.StartsWith('v')) { $v = $v.Substring(1) }
    return $v
}

# Returns @{ Raw; Channel; Version; Floating }
function Parse-Spec([string]$raw) {
    if ([string]::IsNullOrWhiteSpace($raw)) { Die 'missing version/channel spec (e.g. stable, 0.9.5, 0.9.5@dev)' }
    $raw = $raw.Trim()
    if ($raw -in @('stable', 'dev', 'night')) {
        return @{ Raw = $raw; Channel = $raw; Version = $null; Floating = $true }
    }
    $channel = 'stable'
    $ver = $raw
    if ($raw -match '^(.+)@(.+)$') {
        $ver = $Matches[1]
        $channel = $Matches[2]
        if ($channel -eq 'night') { Die 'cannot pin a version on night (use: dcrup install night)' }
        if ($channel -notin @('stable', 'dev')) { Die "unknown channel: $channel (use stable|dev)" }
    }
    $ver = Normalize-Version $ver
    if ($ver -notmatch '^\d+\.\d+\.\d+([-.][0-9A-Za-z.-]+)?$') { Die "invalid version: $raw (expected e.g. 0.9.5)" }
    return @{ Raw = $raw; Channel = $channel; Version = $ver; Floating = $false }
}

function Get-ToolchainId($spec, [string]$resolved) {
    if ($spec.Floating) { return "$($spec.Channel)-$resolved" }
    return "$($spec.Version)-$($spec.Channel)"
}

# ------------------------------------------------------------------ layout --

function Ensure-Dirs {
    foreach ($d in @($script:Tc, $script:Bin, $script:Meta)) {
        if (-not (Test-Path $d)) { New-Item -ItemType Directory -Force -Path $d | Out-Null }
    }
}

function Get-ActivePath { Join-Path $script:Meta 'active' }

function Read-Active {
    $p = Get-ActivePath
    if (Test-Path $p) {
        $v = (Get-Content -LiteralPath $p -Raw)
        if ($v) { return $v.Trim() }
    }
    return $null
}

function Get-Installed {
    if (-not (Test-Path $script:Tc)) { return @() }
    return @(Get-ChildItem -LiteralPath $script:Tc -Directory |
        Where-Object { Test-Path (Join-Path $_.FullName 'dcr.exe') } |
        ForEach-Object { $_.Name })
}

# Sort key so that 0.10.0 > 0.9.5 (plain string sort gets this wrong).
function Get-VersionKey([string]$id) {
    $m = [regex]::Match($id, '(\d+)\.(\d+)\.(\d+)')
    if (-not $m.Success) { return '' }
    return '{0:D6}.{1:D6}.{2:D6}' -f [int]$m.Groups[1].Value, [int]$m.Groups[2].Value, [int]$m.Groups[3].Value
}

# Resolves an installed toolchain id for a spec, or $null.
function Find-Installed($spec) {
    $all = Get-Installed
    if ($spec.Floating) {
        $hits = @($all | Where-Object { $_ -like "$($spec.Channel)-*" })
        if ($hits.Count -eq 0) { return $null }
        if ($spec.Channel -eq 'night') {
            return ($hits | Sort-Object { (Get-Item (Join-Path $script:Tc $_)).LastWriteTimeUtc } | Select-Object -Last 1)
        }
        return ($hits | Sort-Object { Get-VersionKey $_ } | Select-Object -Last 1)
    }
    $id = Get-ToolchainId $spec $null
    if ($all -contains $id) { return $id }
    # A plain VERSION also matches a floating install of that version.
    foreach ($cand in @("stable-$($spec.Version)", "dev-$($spec.Version)")) {
        if ($spec.Channel -eq 'stable' -and $all -contains $cand) { return $cand }
    }
    return $null
}

function Write-Meta([string]$dir, [hashtable]$data) {
    $lines = $data.GetEnumerator() | Sort-Object Key | ForEach-Object { "$($_.Key)=$($_.Value)" }
    Set-Content -LiteralPath (Join-Path $dir 'dcrup-meta') -Value $lines -Encoding ASCII
}

# Copies (not links) the toolchain into bin: Windows symlinks need admin or
# developer mode, and a hard link would block replacing a running dcr.exe.
function Set-Default([string]$id) {
    $src = Join-Path (Join-Path $script:Tc $id) 'dcr.exe'
    if (-not (Test-Path $src)) { Die "toolchain not installed: $id" }
    Ensure-Dirs
    $dst = Join-Path $script:Bin 'dcr.exe'
    $tmp = "$dst.new"
    Copy-Item -LiteralPath $src -Destination $tmp -Force
    if (Test-Path $dst) {
        # A running dcr.exe cannot be overwritten but can be renamed.
        $old = "$dst.old"
        Remove-Item -LiteralPath $old -Force -ErrorAction SilentlyContinue
        try { Move-Item -LiteralPath $dst -Destination $old -Force }
        catch { Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue; Die "cannot replace $dst (is dcr running?): $($_.Exception.Message)" }
        Remove-Item -LiteralPath $old -Force -ErrorAction SilentlyContinue
    }
    Move-Item -LiteralPath $tmp -Destination $dst -Force
    Set-Content -LiteralPath (Get-ActivePath) -Value $id -NoNewline -Encoding ASCII
    Ok "default -> $id ($dst)"
}

# ----------------------------------------------------------------- release --

# Returns the GitHub release object for a spec.
function Get-Release($spec) {
    if (-not $spec.Floating) {
        $rel = Invoke-Api "/releases/tags/v$($spec.Version)"
        if (-not $rel -and $spec.Channel -eq 'dev') { $rel = Invoke-Api "/releases/tags/v$($spec.Version)-dev" }
        if (-not $rel) { Die "release v$($spec.Version) not found (see $($script:RepoUrl)/releases)" }
        return $rel
    }
    if ($spec.Channel -eq 'dev') {
        # dev = the newest published release, pre-release or not. An old
        # pre-release must never win over a newer stable one.
        $all = Invoke-Api '/releases?per_page=30'
        $rel = @($all | Where-Object { -not $_.draft } |
            Sort-Object { [datetime]$_.published_at } -Descending) | Select-Object -First 1
        if (-not $rel) { Die 'no releases found' }
        if (-not $rel.prerelease) { Info "no pre-release newer than stable; dev uses $($rel.tag_name)" }
        return $rel
    }
    $rel = Invoke-Api '/releases/latest'
    if (-not $rel) { Die 'no stable release found' }
    return $rel
}

function Get-MirrorUrl([string]$mirror, [string]$tag, [string]$asset) {
    switch ($mirror) {
        'cf' { return "https://cdn-global.dcr-tool.ru/$tag/$asset" }
        'gh' { return "https://github.com/dexoron/dcr/releases/download/$tag/$asset" }
        'sf' { return "https://sourceforge.net/projects/dcr-tool/files/$tag/$asset/download" }
        'ya' { return "https://cdn-ru.dcr-tool.ru/$tag/$asset" }
    }
    Die "invalid mirror: $mirror"
}

function Get-MirrorsToTry {
    $m = $script:Mirror
    if ($m -eq 'auto') { return $script:Mirrors }
    if ($script:Mirrors -contains $m) { return @($m) }
    Die "invalid --mirror: $m (use auto|cf|gh|sf|ya)"
}

function Get-FileSha256([string]$path) {
    return (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Test-PeFile([string]$path) {
    $fs = [IO.File]::OpenRead($path)
    try { $b = New-Object byte[] 2; $n = $fs.Read($b, 0, 2) } finally { $fs.Dispose() }
    return ($n -eq 2 -and $b[0] -eq 0x4D -and $b[1] -eq 0x5A)   # 'MZ'
}

# Downloads $asset to $dest, verifying the PE header and, when known, SHA256.
# Returns the mirror code that worked.
function Get-Asset([string]$tag, [string]$asset, [string]$dest, [string]$sha256) {
    $tmp = "$dest.download"
    $errors = @()
    foreach ($m in (Get-MirrorsToTry)) {
        $url = Get-MirrorUrl $m $tag $asset
        Info "downloading from $m ..."
        Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
        try {
            Invoke-WebRequest -Uri $url -OutFile $tmp @script:WebArgs
        } catch {
            $code = Get-HttpStatus $_
            $why = if ($code) { "HTTP $code" } else { $_.Exception.Message }
            if ($script:Quiet) { } else { Warn "mirror ${m} failed: $why" }
            $errors += "${m}=$why"
            continue
        }
        if (-not (Test-Path $tmp) -or -not (Test-PeFile $tmp)) {
            Warn "mirror ${m} returned something that is not a Windows executable"
            $errors += "${m}=not-an-exe"
            Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
            continue
        }
        if ($sha256) {
            $got = Get-FileSha256 $tmp
            if ($got -ne $sha256) {
                Warn "mirror ${m}: SHA256 mismatch (got $got)"
                $errors += "${m}=sha256-mismatch"
                Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
                continue
            }
        }
        Move-Item -LiteralPath $tmp -Destination $dest -Force
        return $m
    }
    Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
    Die "could not download $asset from any mirror ($($errors -join ', ')). Check network/proxy (DCRUP_PROXY) or pick one with --mirror"
}

# ----------------------------------------------------------------- install --

function Install-Prebuilt($spec, [bool]$force) {
    $triple = Get-Target
    $rel = Get-Release $spec
    $tag = $rel.tag_name
    $ver = Normalize-Version $tag
    $assetName = "dcr-$triple-$ver.exe"
    $asset = @($rel.assets | Where-Object { $_.name -eq $assetName }) | Select-Object -First 1
    if (-not $asset) { Die "release $tag has no asset $assetName" }
    $sha = $null
    if ($asset.PSObject.Properties['digest'] -and "$($asset.digest)" -like 'sha256:*') {
        $sha = "$($asset.digest)".Substring(7).ToLowerInvariant()
    }

    $id = Get-ToolchainId $spec $ver
    $dir = Join-Path $script:Tc $id
    $exe = Join-Path $dir 'dcr.exe'
    if ((Test-Path $exe) -and -not $force) {
        Ok "already installed: $id"
        Set-Default $id
        return $id
    }
    Ensure-Dirs
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
    Info "installing $assetName ($tag)"
    $mirror = Get-Asset $tag $assetName $exe $sha
    if (-not $sha) { Warn 'release has no published SHA256; integrity was checked by file type only' }

    # Smoke test: catches a missing VC++ runtime or a broken download early.
    $out = $null
    $code = 1
    try {
        $ErrorActionPreference = 'Continue'
        $out = (& $exe --version 2>&1 | Out-String).Trim()
        $code = $LASTEXITCODE
    } catch { $out = $_.Exception.Message }
    finally { $ErrorActionPreference = 'Stop' }
    if ($code -ne 0 -or -not ("$out" -match 'dcr')) {
        Remove-Item -LiteralPath $dir -Recurse -Force -ErrorAction SilentlyContinue
        $hint = if ($triple -like '*-msvc') { ' Install the Visual C++ runtime (https://aka.ms/vs/17/release/vc_redist.x64.exe) or use --target gnu.' } else { '' }
        Die "installed dcr.exe does not run: $out.$hint"
    }

    Write-Meta $dir @{ channel = $spec.Channel; version = $ver; tag = $tag; source = 'release'; triple = $triple; mirror = $mirror; sha256 = $(if ($sha) { $sha } else { Get-FileSha256 $exe }) }
    Ok "installed $id ($triple via $mirror)"
    Set-Default $id
    return $id
}

function Install-Build($spec, [bool]$force) {
    foreach ($tool in @('git', 'cargo')) {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) { Die "$tool not found in PATH (needed for --build / night)" }
    }
    $tmp = Join-Path ([IO.Path]::GetTempPath()) ('dcrup-' + [guid]::NewGuid().ToString('N'))
    $src = Join-Path $tmp 'src'
    New-Item -ItemType Directory -Path $tmp | Out-Null
    $pushed = $false
    try {
        if ($spec.Channel -eq 'night') {
            $ref = 'dev'
        } elseif ($spec.Floating) {
            $ref = (Get-Release $spec).tag_name
        } else {
            $ref = "v$($spec.Version)"
        }
        Info "cloning $($script:RepoUrl) ($ref)"
        if (-not (Invoke-Native 'git' @('clone', '--quiet', '--depth', '1', '--branch', $ref, $script:RepoUrl, $src))) { Die "git clone of $ref failed" }
        $sha = (& git -C $src rev-parse --short HEAD).Trim()
        $ver = if ($spec.Channel -eq 'night') { $sha } else { Normalize-Version $ref }

        $id = Get-ToolchainId $spec $ver
        $dir = Join-Path $script:Tc $id
        $exe = Join-Path $dir 'dcr.exe'
        if ((Test-Path $exe) -and -not $force) {
            Ok "already installed: $id"
            Set-Default $id
            return $id
        }

        Info "building with cargo (features: $($script:Features)); this takes a few minutes"
        Push-Location $src; $pushed = $true
        if (-not (Invoke-Native 'cargo' @('build', '--release', '--locked', '--features', $script:Features))) { Die 'cargo build failed' }
        Pop-Location; $pushed = $false

        Ensure-Dirs
        if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
        Copy-Item -LiteralPath (Join-Path $src 'target\release\dcr.exe') -Destination $exe -Force
        Write-Meta $dir @{ channel = $spec.Channel; version = $(if ($spec.Channel -eq 'night') { 'night' } else { $ver }); git = $sha; source = 'build'; features = $script:Features }
        Ok "built $id"
        Set-Default $id
        return $id
    } finally {
        if ($pushed) { Pop-Location }
        Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# -------------------------------------------------------------------- PATH --

function Get-PathKey([bool]$writable) {
    if ($script:System) {
        return [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey('SYSTEM\CurrentControlSet\Control\Session Manager\Environment', $writable)
    }
    return [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $writable)
}

function Get-RawPath {
    $key = Get-PathKey $false
    try {
        if (-not $key) { return '' }
        $v = $key.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
        return [string]$v
    } finally { if ($key) { $key.Close() } }
}

function Set-RawPath([string]$value) {
    $key = Get-PathKey $true
    if (-not $key) { Die 'cannot open the PATH registry key for writing' }
    try { $key.SetValue('Path', $value, [Microsoft.Win32.RegistryValueKind]::ExpandString) } finally { $key.Close() }
    # Tell Explorer and new shells that the environment changed (WM_SETTINGCHANGE),
    # without compiling P/Invoke code: .NET broadcasts on every Set call.
    $scope = if ($script:System) { 'Machine' } else { 'User' }
    try {
        [Environment]::SetEnvironmentVariable('DCRUP_PATH_REFRESH', '1', $scope)
        [Environment]::SetEnvironmentVariable('DCRUP_PATH_REFRESH', $null, $scope)
    } catch {}
}

function Test-SamePath([string]$a, [string]$b) {
    $na = [Environment]::ExpandEnvironmentVariables($a).TrimEnd('\')
    $nb = [Environment]::ExpandEnvironmentVariables($b).TrimEnd('\')
    return [string]::Equals($na, $nb, [StringComparison]::OrdinalIgnoreCase)
}

function Add-ToPath {
    $dir = $script:Bin
    # Current session first, so dcr works right away after `irm | iex`.
    if (-not (@($env:Path -split ';') | Where-Object { $_ -and (Test-SamePath $_ $dir) })) {
        $env:Path = "$dir;$env:Path"
    }
    if ($script:NoModifyPath) {
        Info "PATH not modified (--no-modify-path); add $dir yourself"
        return
    }
    $raw = Get-RawPath
    # Keep the existing value byte-for-byte (empty entries included); only prepend.
    $parts = @($raw -split ';')
    if ($parts | Where-Object { $_ -and (Test-SamePath $_ $dir) }) { return }
    Set-RawPath $(if ($raw) { "$dir;$raw" } else { $dir })
    $scope = if ($script:System) { 'system' } else { 'user' }
    Ok "added $dir to the $scope PATH (open a new terminal to pick it up)"
}

function Remove-FromPath {
    if ($script:NoModifyPath) { return }
    $raw = Get-RawPath
    # Remove only our own entry; leave every other element (even empty ones) as is.
    $parts = @($raw -split ';')
    $keep = @($parts | Where-Object { -not ($_ -and (Test-SamePath $_ $script:Bin)) })
    if ($keep.Count -ne $parts.Count) {
        Set-RawPath ($keep -join ';')
        Ok "removed $($script:Bin) from PATH"
    }
    $env:Path = (@($env:Path -split ';') | Where-Object { $_ -and -not (Test-SamePath $_ $script:Bin) }) -join ';'
}

# ------------------------------------------------------------ self-install --

function Test-Admin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    return (New-Object Security.Principal.WindowsPrincipal $id).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Writes dcrup.ps1 + dcrup.cmd into bin. Uses the running script's own text
# when available, so the installed dcrup is exactly what was run.
function Install-Self {
    Ensure-Dirs
    $dst = Join-Path $script:Bin 'dcrup.ps1'
    $text = $script:SelfText
    if ($script:SelfPath -and (Test-SamePath $script:SelfPath $dst)) {
        $text = $null   # running the installed copy; nothing to copy
    } elseif (-not $text -or $text -notmatch 'DcrupVersion') {
        Info "fetching dcrup from $($script:InstallUrl)"
        $text = (Invoke-WebRequest -Uri $script:InstallUrl @script:WebArgs).Content
        if ($text -is [byte[]]) { $text = [Text.Encoding]::UTF8.GetString($text) }
        if ($text -notmatch 'DcrupVersion') { Die "unexpected content from $($script:InstallUrl)" }
    }
    if ($text) {
        $text = $text.TrimStart([char]0xFEFF)
        [IO.File]::WriteAllText($dst, $text, (New-Object Text.UTF8Encoding $false))
    }
    $cmd = @(
        '@echo off',
        'powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0dcrup.ps1" %*',
        'exit /b %ERRORLEVEL%'
    )
    Set-Content -LiteralPath (Join-Path $script:Bin 'dcrup.cmd') -Value $cmd -Encoding ASCII
    Add-ToPath
    Ok "dcrup installed to $($script:Bin)"
}

# ---------------------------------------------------------------- commands --

function Read-Options([string[]]$argv) {
    $o = @{ Spec = $null; Force = $false; Build = $false; Rest = @() }
    for ($i = 0; $i -lt $argv.Count; $i++) {
        $a = $argv[$i]
        $hasNext = ($i + 1 -lt $argv.Count)
        # `switch -Regex` runs every matching clause, so each one must `break`.
        switch -Regex ($a) {
            '^--force$'          { $o.Force = $true; break }
            '^--build$'          { $o.Build = $true; break }
            '^--release$'        { $o.Build = $false; break }
            '^(-q|--quiet)$'     { $script:Quiet = $true; break }
            '^(-y|--yes)$'       { break }
            '^--system$'         { $script:System = $true; break }
            '^--no-modify-path$' { $script:NoModifyPath = $true; break }
            '^--mirror=(.+)$'    { $script:Mirror = $Matches[1]; break }
            '^--target=(.+)$'    { $script:TargetPref = $Matches[1]; break }
            '^--mirror$'         { if (-not $hasNext) { Die '--mirror needs auto|cf|gh|sf|ya' }; $i++; $script:Mirror = $argv[$i]; break }
            '^--target$'         { if (-not $hasNext) { Die '--target needs auto|msvc|gnu' }; $i++; $script:TargetPref = $argv[$i]; break }
            '^--all$'            { $o.Rest += $a; break }
            '^-'                 { Die "unknown option: $a (see: dcrup help)" }
            default              { $o.Rest += $a }
        }
    }
    if ($script:Mirror -ne 'auto' -and $script:Mirrors -notcontains $script:Mirror) { Die "invalid --mirror: $($script:Mirror) (use auto|cf|gh|sf|ya)" }
    if ($script:TargetPref -notin @('auto', 'msvc', 'gnu')) { Die "invalid --target: $($script:TargetPref) (use auto|msvc|gnu)" }
    return $o
}

function Assert-SystemRights {
    if ($script:System -and -not (Test-Admin)) { Die '--system / DCRUP_SYSTEM needs an elevated (Administrator) PowerShell' }
}

function Cmd-Install($o, [bool]$withSelf) {
    if ($o.Rest.Count -gt 1) { Die "unexpected argument: $($o.Rest[1])" }
    $raw = if ($o.Rest.Count -eq 1) { $o.Rest[0] } elseif ($env:DCRUP_VERSION) { $env:DCRUP_VERSION } else { 'stable' }
    $spec = Parse-Spec $raw
    Assert-SystemRights
    Ensure-Dirs
    $build = $o.Build -or $spec.Channel -eq 'night'
    Info "install $($spec.Raw) (mode: $(if ($build) { 'build' } else { 'prebuilt' }), mirror: $($script:Mirror), home: $($script:Home_))"
    if ($build) { $id = Install-Build $spec $o.Force } else { $id = Install-Prebuilt $spec $o.Force }
    if ($withSelf) { Install-Self } else { Add-ToPath }
    if (-not $script:Quiet) {
        Write-Host ''
        Write-Host "DCR $id is ready. Run: dcr --version" -ForegroundColor Green
    }
}

function Cmd-Default($o) {
    if ($o.Rest.Count -ne 1) { Die 'usage: dcrup default <spec>' }
    $spec = Parse-Spec $o.Rest[0]
    $id = Find-Installed $spec
    if (-not $id) { Die "not installed: $($spec.Raw) (run: dcrup install $($spec.Raw))" }
    Set-Default $id
}

function Cmd-Update($o) {
    $a = Read-Active
    if (-not $a) { Die 'no default toolchain (run: dcrup install stable)' }
    $o.Force = $false
    switch -Wildcard ($a) {
        'night-*'  { $o.Force = $true; $o.Rest = @('night') }
        'stable-*' { $o.Rest = @('stable') }
        'dev-*'    { $o.Rest = @('dev') }
        default    { Ok "default $a is pinned; install a newer version explicitly (dcrup install stable)"; return }
    }
    Cmd-Install $o $false
}

function Cmd-List {
    $active = Read-Active
    $all = Get-Installed
    if ($all.Count -eq 0) { Write-Host 'no toolchains installed'; return }
    $all | Sort-Object { Get-VersionKey $_ }, { $_ } | ForEach-Object {
        if ($_ -eq $active) { Write-Host " * $_" } else { Write-Host "   $_" }
    }
}

function Cmd-Show {
    Write-Host "dcrup      $($script:DcrupVersion)"
    Write-Host "DCRUP_HOME $($script:Home_)"
    Write-Host "DCRUP_BIN  $($script:Bin)"
    $a = Read-Active
    Write-Host "default    $(if ($a) { $a } else { 'none' })"
    if ($a) {
        $m = Join-Path (Join-Path $script:Tc $a) 'dcrup-meta'
        if (Test-Path $m) { Get-Content -LiteralPath $m | ForEach-Object { Write-Host "  $_" } }
    }
    $exe = Join-Path $script:Bin 'dcr.exe'
    if (Test-Path $exe) {
        $ErrorActionPreference = 'Continue'
        Write-Host (& $exe --version 2>&1 | Out-String).Trim()
    }
}

function Cmd-Uninstall($o) {
    if ($o.Rest -contains '--all') {
        if (Test-Path $script:Tc) { Remove-Item -LiteralPath $script:Tc -Recurse -Force }
        Remove-Item -LiteralPath (Join-Path $script:Bin 'dcr.exe') -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath (Get-ActivePath) -Force -ErrorAction SilentlyContinue
        Ok 'removed all toolchains'
        return
    }
    if ($o.Rest.Count -ne 1) { Die 'usage: dcrup uninstall <spec>|--all' }
    $spec = Parse-Spec $o.Rest[0]
    $id = Find-Installed $spec
    if (-not $id) { Die "not installed: $($spec.Raw)" }
    Remove-Item -LiteralPath (Join-Path $script:Tc $id) -Recurse -Force
    if ((Read-Active) -eq $id) {
        Remove-Item -LiteralPath (Join-Path $script:Bin 'dcr.exe') -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath (Get-ActivePath) -Force -ErrorAction SilentlyContinue
        $left = @(Get-Installed)
        if ($left.Count -gt 0) {
            Set-Default ($left | Sort-Object { Get-VersionKey $_ } | Select-Object -Last 1)
        } else {
            Warn 'removed the default toolchain; run: dcrup install stable'
        }
    }
    Ok "removed $id"
}

function Cmd-SelfUninstall {
    Assert-SystemRights
    Remove-FromPath
    if (Test-Path $script:Home_) {
        Remove-Item -LiteralPath $script:Home_ -Recurse -Force -ErrorAction SilentlyContinue
        if (Test-Path $script:Home_) { Warn "could not fully remove $($script:Home_) (files in use?)" }
    }
    if ((Test-Path $script:Bin) -and -not ($script:Bin.StartsWith($script:Home_, [StringComparison]::OrdinalIgnoreCase))) {
        foreach ($f in @('dcr.exe', 'dcrup.ps1', 'dcrup.cmd')) { Remove-Item -LiteralPath (Join-Path $script:Bin $f) -Force -ErrorAction SilentlyContinue }
    }
    Ok 'dcrup and all toolchains removed'
}

# ------------------------------------------------------------- entry point --

function Invoke-Dcrup([string[]]$argv) {
    $cmd = if ($argv.Count -ge 1) { $argv[0] } else { '' }
    $rest = if ($argv.Count -gt 1) { [string[]]$argv[1..($argv.Count - 1)] } else { [string[]]@() }
    if ($cmd -like '-*' -and $cmd -notin @('-h', '--help', '--version', '-V')) {
        # `dcrup --quiet` etc.: options without a command mean the default install.
        $rest = [string[]]$argv; $cmd = ''
    }
    switch ($cmd) {
        { $_ -in @('help', '-h', '--help') } { Show-Usage; return }
        { $_ -in @('--version', '-V') }      { Write-Host "dcrup $($script:DcrupVersion)"; return }
    }
    $o = Read-Options $rest
    Set-Layout
    Initialize-Network
    switch ($cmd) {
        ''               { Cmd-Install $o $true }
        'install'        { Cmd-Install $o $true }
        'default'        { Cmd-Default $o }
        'update'         { Cmd-Update $o }
        'list'           { Cmd-List }
        'show'           { Cmd-Show }
        'which' {
            $p = Join-Path $script:Bin 'dcr.exe'
            if (Test-Path $p) { Write-Output $p } else { Die 'dcr is not installed (run: dcrup install stable)' }
        }
        'uninstall'      { Cmd-Uninstall $o }
        'self-install'   { Assert-SystemRights; Install-Self }
        'self-uninstall' { Cmd-SelfUninstall }
        default          { Die "unknown command: $cmd (see: dcrup help)" }
    }
}

# Remember how we were started: as a file (-File / dcrup.cmd), as a
# scriptblock (& ([scriptblock]::Create(...))) or via `irm | iex`.
$script:SelfPath = $PSCommandPath
$script:SelfText = $null
try { $script:SelfText = $MyInvocation.MyCommand.ScriptBlock.ToString() } catch {}
$script:AsFile = [bool]$PSCommandPath

$__dcrupFailed = $false
try {
    Invoke-Dcrup ([string[]]$args)
} catch {
    $__dcrupFailed = $true
    $msg = if ($_.Exception.Data['dcrup']) { $_.Exception.Message } else { "$($_.Exception.Message) (at line $($_.InvocationInfo.ScriptLineNumber))" }
    Write-Host "[error] $msg" -ForegroundColor Red
} finally {
    $ProgressPreference = 'Continue'
}
if ($__dcrupFailed) {
    # Exit the process only when running as a file; under iex/scriptblock
    # that would close the user's PowerShell window.
    if ($script:AsFile) { exit 1 }
    $global:LASTEXITCODE = 1
} elseif ($script:AsFile) {
    exit 0
}
