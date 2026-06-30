param(
    [string]$NodeInstallRoot = "$env:LOCALAPPDATA\fruitfacts\node",
    [switch]$SkipNodeInstall,
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Usage {
    Write-Host "Usage: .\helper_scripts\web_screenshot_install_or_update.ps1 [-NodeInstallRoot path] [-SkipNodeInstall]"
    Write-Host ""
    Write-Host "Installs local Node.js if needed and installs backend web screenshot npm dependencies"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\helper_scripts\web_screenshot_install_or_update.ps1"
    Write-Host "  .\helper_scripts\web_screenshot_install_or_update.ps1 -SkipNodeInstall"
}

function Get-NodeDirectoryFromInstallRoot {
    param(
        [string]$InstallRoot
    )

    if (!(Test-Path -LiteralPath $InstallRoot)) {
        return $null
    }

    $nodes = @(Get-ChildItem -LiteralPath $InstallRoot -Directory -Filter "node-v*-win-x64" |
        Where-Object { Test-Path -LiteralPath (Join-Path $_.FullName "node.exe") } |
        Sort-Object -Property @{
            Expression = {
                if ($_.Name -match "node-v([0-9]+\.[0-9]+\.[0-9]+)-win-x64") {
                    [version]$Matches[1]
                } else {
                    [version]"0.0.0"
                }
            }
            Descending = $true
        })

    if ($nodes.Count -eq 0) {
        return $null
    }

    return $nodes[0].FullName
}

function Add-PathEntry {
    param(
        [string]$PathEntry
    )

    if (!$PathEntry) {
        return
    }

    $parts = @($env:PATH -split ";") | Where-Object { $_ }
    if ($parts -notcontains $PathEntry) {
        $env:PATH = "$PathEntry;$env:PATH"
    }
}

function Get-NodeDirectory {
    param(
        [string]$InstallRoot
    )

    $pathNode = Get-Command node -ErrorAction SilentlyContinue
    if ($pathNode) {
        return Split-Path -Parent $pathNode.Source
    }

    return Get-NodeDirectoryFromInstallRoot -InstallRoot $InstallRoot
}

if ($Help) {
    Write-Usage
    exit 0
}

$scriptDir = Split-Path -Parent $PSCommandPath
$repoRoot = Resolve-Path (Join-Path $scriptDir "..")

if (!$SkipNodeInstall) {
    $nodeInstaller = Join-Path $scriptDir "node_lts_install_or_update.ps1"
    if (!(Test-Path -LiteralPath $nodeInstaller)) {
        throw "Node install helper was not found at $nodeInstaller"
    }

    & $nodeInstaller -InstallRoot $NodeInstallRoot
}

$nodeDir = Get-NodeDirectory -InstallRoot $NodeInstallRoot
if (!$nodeDir) {
    throw "Node.js was not found"
}

$npmGlobalDir = Join-Path $env:APPDATA "npm"
Add-PathEntry -PathEntry $nodeDir
Add-PathEntry -PathEntry $npmGlobalDir

$nodeExe = Join-Path $nodeDir "node.exe"
$npmCmd = Join-Path $nodeDir "npm.cmd"
if (!(Test-Path -LiteralPath $nodeExe)) {
    throw "node.exe was not found at $nodeExe"
}
if (!(Test-Path -LiteralPath $npmCmd)) {
    throw "npm.cmd was not found at $npmCmd"
}

Write-Host "Using Node: $nodeExe"
& $nodeExe --version
if ($LASTEXITCODE -ne 0) {
    throw "Node version check failed with exit code $LASTEXITCODE"
}

$webScreenshotDir = Join-Path $repoRoot "backend\web_screenshot"
if (!(Test-Path -LiteralPath (Join-Path $webScreenshotDir "package.json"))) {
    throw "backend web screenshot package.json was not found"
}

Push-Location $webScreenshotDir
try {
    Write-Host "Installing backend web screenshot npm dependencies"
    & $npmCmd install
    if ($LASTEXITCODE -ne 0) {
        throw "npm install failed with exit code $LASTEXITCODE"
    }

    Write-Host "Checking Puppeteer browser"
    & $nodeExe -e "const puppeteer=require('./node_modules/puppeteer'); console.log(puppeteer.executablePath())"
    if ($LASTEXITCODE -ne 0) {
        throw "Puppeteer browser check failed with exit code $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

Write-Host "Finished web screenshot install/update"
