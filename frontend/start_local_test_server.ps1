param(
    [int]$Port = 3000,
    [switch]$Production,
    [switch]$SkipNodeInstall,
    [switch]$SkipNpmInstall,
    [switch]$SkipBuild,
    [switch]$CheckOnly,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

function Write-Usage {
    Write-Host "Usage: .\start_local_test_server.ps1 [-Port 3000] [-Production] [-SkipNodeInstall] [-SkipNpmInstall] [-SkipBuild] [-CheckOnly]"
    Write-Host "Starts the FruitFacts frontend with repo-local Node/npm when they are not on PATH"
    Write-Host "Requires local.fruitfacts.xyz to resolve to localhost"
    Write-Host "Default mode runs npm run dev"
    Write-Host "Production mode runs npm run build, then npm run start"
}

if ($Help) {
    Write-Usage
    return
}

if ($Port -lt 1 -or $Port -gt 65535) {
    throw "Port must be between 1 and 65535"
}

$frontendRoot = $PSScriptRoot
$repoRoot = Split-Path -Parent $frontendRoot
$nodeInstallRoot = Join-Path $env:LOCALAPPDATA "fruitfacts\node"

function Add-PathEntry {
    param([string]$PathEntry)
    if (!$PathEntry -or !(Test-Path -LiteralPath $PathEntry)) {
        return
    }
    $parts = $env:PATH -split ";"
    if ($parts -notcontains $PathEntry) {
        $env:PATH = "$PathEntry;$env:PATH"
    }
}

function Get-NodeDirectoryFromInstallRoot {
    param([string]$InstallRoot)
    if (!(Test-Path -LiteralPath $InstallRoot)) {
        return $null
    }
    $nodes = @(Get-ChildItem -LiteralPath $InstallRoot -Directory -Filter "node-v*-win-x64" |
        Where-Object { Test-Path -LiteralPath (Join-Path $_.FullName "node.exe") } |
        Sort-Object {
            if ($_.Name -match "node-v([0-9]+\.[0-9]+\.[0-9]+)-win-x64") {
                [version]$Matches[1]
            }
            else {
                [version]"0.0.0"
            }
        } -Descending)
    if ($nodes.Count -eq 0) {
        return $null
    }
    return $nodes[0].FullName
}

function Get-NpmCommand {
    $npmCmd = Get-Command npm.cmd -ErrorAction SilentlyContinue
    if ($npmCmd) {
        return $npmCmd.Source
    }
    $npm = Get-Command npm -ErrorAction SilentlyContinue
    if ($npm) {
        return $npm.Source
    }
    return $null
}

function Test-NodeReady {
    $node = Get-Command node -ErrorAction SilentlyContinue
    $npm = Get-NpmCommand
    return ($null -ne $node -and $null -ne $npm)
}

function Assert-LocalFruitfactsHostAlias {
    $hostName = "local.fruitfacts.xyz"
    Write-Host "Checking $hostName host alias"
    try {
        $addresses = @([System.Net.Dns]::GetHostAddresses($hostName))
    }
    catch {
        throw "$hostName must resolve to localhost before starting the frontend; add a hosts entry like: 127.0.0.1 $hostName"
    }

    $nonLoopback = @($addresses | Where-Object { -not [System.Net.IPAddress]::IsLoopback($_) })
    if ($addresses.Count -eq 0 -or $nonLoopback.Count -gt 0) {
        $resolved = if ($addresses.Count -gt 0) { $addresses -join ", " } else { "no addresses" }
        throw "$hostName must resolve only to localhost before starting the frontend; currently resolves to: $resolved"
    }

    Write-Host "$hostName resolves to $($addresses -join ', ')"
}

function Enable-NodeForCurrentProcess {
    if (Test-NodeReady) {
        return
    }

    $nodeEnvPs = Join-Path $nodeInstallRoot "node_env.ps1"
    if (Test-Path -LiteralPath $nodeEnvPs) {
        . $nodeEnvPs
    }
    else {
        $nodeDir = Get-NodeDirectoryFromInstallRoot -InstallRoot $nodeInstallRoot
        if ($nodeDir) {
            Add-PathEntry -PathEntry $nodeDir
            Add-PathEntry -PathEntry (Join-Path $env:APPDATA "npm")
        }
    }
    if (Test-NodeReady) {
        return
    }

    if ($SkipNodeInstall) {
        throw "Node.js and npm were not found; rerun without -SkipNodeInstall or run ..\helper_scripts\node_lts_install_or_update.ps1"
    }

    $installer = Join-Path $repoRoot "helper_scripts\node_lts_install_or_update.ps1"
    if (!(Test-Path -LiteralPath $installer)) {
        throw "Node install helper was not found at $installer"
    }

    Write-Host "Installing local Node.js"
    & $installer -InstallRoot $nodeInstallRoot
    if ($LASTEXITCODE) {
        throw "Node install helper failed with exit code $LASTEXITCODE"
    }

    $nodeEnvPs = Join-Path $nodeInstallRoot "node_env.ps1"
    if (Test-Path -LiteralPath $nodeEnvPs) {
        . $nodeEnvPs
    }
    if (!(Test-NodeReady)) {
        throw "Node.js and npm were not found after local install"
    }
}

function Invoke-Checked {
    param(
        [string]$Label,
        [string]$Command,
        [string[]]$Arguments
    )
    Write-Host $Label
    & $Command @Arguments
    if ($LASTEXITCODE) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

Assert-LocalFruitfactsHostAlias
Enable-NodeForCurrentProcess

$nodeCommand = (Get-Command node -ErrorAction Stop).Source
$npmCommand = Get-NpmCommand
if (!$npmCommand) {
    throw "npm was not found after Node setup"
}

Write-Host "Using node: $nodeCommand"
& $nodeCommand --version
if ($LASTEXITCODE) {
    throw "node --version failed with exit code $LASTEXITCODE"
}

Write-Host "Using npm: $npmCommand"
& $npmCommand --version
if ($LASTEXITCODE) {
    throw "npm --version failed with exit code $LASTEXITCODE"
}

if ($CheckOnly) {
    Write-Host "Node/npm check passed"
    return
}

Push-Location $frontendRoot
try {
    if (!(Test-Path -LiteralPath "node_modules") -and !$SkipNpmInstall) {
        Invoke-Checked "Installing frontend npm dependencies" $npmCommand @("install", "--force")
    }

    if ($Production) {
        if (!$SkipBuild) {
            Invoke-Checked "Building frontend" $npmCommand @("run", "build")
        }
        Write-Host "Starting frontend production server on http://local.fruitfacts.xyz:$Port"
        & $npmCommand @("run", "start", "--", "-p", "$Port")
        exit $LASTEXITCODE
    }

    Write-Host "Starting frontend dev server on http://local.fruitfacts.xyz:$Port"
    & $npmCommand @("run", "dev", "--", "-p", "$Port")
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
