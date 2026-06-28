param(
    [string]$VenvPath,
    [string]$DvcVersion = "",
    [switch]$Pull,
    [switch]$Doctor,
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Usage {
    Write-Host "Usage: .\helper_scripts\dvc_install_or_update.ps1 [-VenvPath path] [-DvcVersion version] [-Pull] [-Doctor]"
    Write-Host ""
    Write-Host "Installs or updates DVC with Google Drive support using pip in a local virtualenv"
    Write-Host "This avoids Chocolatey and the Windows exe installer path noted as unreliable in README.md"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\helper_scripts\dvc_install_or_update.ps1"
    Write-Host "  .\helper_scripts\dvc_install_or_update.ps1 -Pull"
    Write-Host "  .\helper_scripts\dvc_install_or_update.ps1 -DvcVersion 3.60.1"
}

function Get-LocalAppData {
    if ($env:LOCALAPPDATA) {
        return $env:LOCALAPPDATA
    }

    if ($env:USERPROFILE) {
        return Join-Path $env:USERPROFILE "AppData\Local"
    }

    throw "LOCALAPPDATA and USERPROFILE are not set"
}

function Get-Python {
    $candidates = @(
        [pscustomobject]@{ Command = "py"; Arguments = @("-3") },
        [pscustomobject]@{ Command = "python"; Arguments = @() },
        [pscustomobject]@{ Command = "python3"; Arguments = @() }
    )

    foreach ($candidate in $candidates) {
        try {
            $versionText = & $candidate.Command @($candidate.Arguments) -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')"
            if ($LASTEXITCODE -ne 0) {
                continue
            }

            if ([version]$versionText -lt [version]"3.9") {
                continue
            }

            return $candidate
        } catch {
        }
    }

    throw "Python 3.9 or newer was not found"
}

function Invoke-Python {
    param(
        [object]$Python,
        [string[]]$Arguments
    )

    & $Python.Command @($Python.Arguments) @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Python command failed with exit code $LASTEXITCODE"
    }
}

if ($Help) {
    Write-Usage
    exit 0
}

$scriptDir = Split-Path -Parent $PSCommandPath
$repoRoot = Resolve-Path (Join-Path $scriptDir "..")

if (-not $VenvPath) {
    $VenvPath = Join-Path (Get-LocalAppData) "fruitfacts\dvc-venv"
}

$python = Get-Python
Write-Host "Using Python launcher: $($python.Command) $($python.Arguments -join ' ')"
Write-Host "Using virtualenv: $VenvPath"

if (!(Test-Path $VenvPath)) {
    $parent = Split-Path -Parent $VenvPath
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    Write-Host "Creating virtualenv"
    Invoke-Python -Python $python -Arguments @("-m", "venv", $VenvPath)
}

$venvPython = Join-Path $VenvPath "Scripts\python.exe"
$dvcExe = Join-Path $VenvPath "Scripts\dvc.exe"

if (!(Test-Path $venvPython)) {
    throw "Virtualenv python was not found at $venvPython"
}

Write-Host "Updating pip tooling"
& $venvPython -m pip install --upgrade pip setuptools wheel
if ($LASTEXITCODE -ne 0) {
    throw "pip tooling update failed with exit code $LASTEXITCODE"
}

$packages = @()
if ($DvcVersion) {
    $packages += "dvc[gdrive]==$DvcVersion"
} else {
    $packages += "dvc[gdrive]"
}
$packages += "pydrive2"

Write-Host "Installing DVC packages: $($packages -join ', ')"
& $venvPython -m pip install --upgrade @packages
if ($LASTEXITCODE -ne 0) {
    throw "DVC package install failed with exit code $LASTEXITCODE"
}

if (!(Test-Path $dvcExe)) {
    throw "DVC executable was not found at $dvcExe"
}

Write-Host "DVC executable: $dvcExe"
& $dvcExe --version
if ($LASTEXITCODE -ne 0) {
    throw "DVC version check failed with exit code $LASTEXITCODE"
}

$credentialPath = Join-Path $repoRoot "michael-gdrive-credentials.json"
if (Test-Path $credentialPath) {
    Write-Host "Found Google Drive credential file: $credentialPath"
} else {
    Write-Warning "Google Drive credential file was not found at $credentialPath"
}

if ($Doctor) {
    Write-Host "Running dvc doctor"
    & $dvcExe doctor
    if ($LASTEXITCODE -ne 0) {
        throw "dvc doctor failed with exit code $LASTEXITCODE"
    }
}

if ($Pull) {
    Write-Host "Running dvc pull"
    Push-Location $repoRoot
    try {
        & $dvcExe pull
        if ($LASTEXITCODE -ne 0) {
            throw "dvc pull failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
}

Write-Host "Finished DVC install/update"
Write-Host "Run DVC from this repo with:"
Write-Host "  & `"$dvcExe`" pull"
