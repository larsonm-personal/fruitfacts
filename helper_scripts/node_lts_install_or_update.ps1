param(
    [string]$InstallRoot = "$env:LOCALAPPDATA\fruitfacts\node"
)

$ErrorActionPreference = "Stop"

$arch = "win-x64"
$indexUrl = "https://nodejs.org/dist/index.json"

New-Item -ItemType Directory -Force -Path $InstallRoot | Out-Null

Write-Host "Fetching Node release index"
$releases = Invoke-RestMethod -Uri $indexUrl
$latestLts = $releases | Where-Object { $_.lts -ne $false } | Select-Object -First 1
if ($null -eq $latestLts) {
    throw "No Node LTS release found"
}

$version = $latestLts.version
$zipName = "node-$version-$arch.zip"
$nodeDir = Join-Path $InstallRoot "node-$version-$arch"
$nodeExe = Join-Path $nodeDir "node.exe"

if (!(Test-Path -LiteralPath $nodeExe)) {
    $zipPath = Join-Path $InstallRoot $zipName
    $zipUrl = "https://nodejs.org/dist/$version/$zipName"

    Write-Host "Downloading Node $version"
    Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath

    Write-Host "Expanding Node $version"
    Expand-Archive -Path $zipPath -DestinationPath $InstallRoot -Force
    Remove-Item -LiteralPath $zipPath -Force
}
else {
    Write-Host "Node $version already installed"
}

$npmGlobalDir = Join-Path $env:APPDATA "npm"
New-Item -ItemType Directory -Force -Path $npmGlobalDir | Out-Null

$envBatPath = Join-Path $InstallRoot "node_env.bat"
$envLines = @(
    "@echo off",
    "set ""PATH=$nodeDir;$npmGlobalDir;%PATH%""",
    "set ""FRUITFACTS_NODE_VERSION=$version"""
)
Set-Content -LiteralPath $envBatPath -Value $envLines -Encoding Ascii

Write-Host "Node $version ready"
