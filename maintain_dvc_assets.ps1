param(
    [switch]$RedoAllThumbnails,
    [switch]$RefreshAllDvcPointers,
    [switch]$ShowDvcDiff,
    [switch]$SkipReferencePdfDiscovery,
    [switch]$SkipThumbnails,
    [switch]$SkipPdfThumbnails,
    [switch]$SkipWebsiteThumbnails,
    [switch]$RequireWebsiteThumbnails,
    [switch]$SkipNodeInstall,
    [switch]$SkipDvc,
    [switch]$SkipDvcInstall,
    [switch]$NoPush,
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Usage {
    Write-Host "Usage: .\maintain_dvc_assets.ps1 [-RedoAllThumbnails] [-RefreshAllDvcPointers] [-ShowDvcDiff] [-SkipReferencePdfDiscovery] [-SkipThumbnails] [-SkipPdfThumbnails] [-SkipWebsiteThumbnails] [-RequireWebsiteThumbnails] [-SkipNodeInstall] [-SkipDvc] [-SkipDvcInstall] [-NoPush]"
    Write-Host ""
    Write-Host "Discovers reference PDF companions, generates missing thumbnails, adds new PDF/JPG assets to DVC, and pushes cached data"
    Write-Host "By default, DVC add only runs for files missing a neighboring .dvc file"
    Write-Host "Website thumbnail dependencies are installed through helper_scripts\web_screenshot_install_or_update.ps1 when needed"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\maintain_dvc_assets.ps1"
    Write-Host "  .\maintain_dvc_assets.ps1 -NoPush"
    Write-Host "  .\maintain_dvc_assets.ps1 -RedoAllThumbnails"
    Write-Host "  .\maintain_dvc_assets.ps1 -SkipWebsiteThumbnails"
    Write-Host "  .\maintain_dvc_assets.ps1 -SkipReferencePdfDiscovery"
    Write-Host "  .\maintain_dvc_assets.ps1 -SkipNodeInstall"
    Write-Host "  .\maintain_dvc_assets.ps1 -RequireWebsiteThumbnails"
    Write-Host "  .\maintain_dvc_assets.ps1 -RefreshAllDvcPointers -ShowDvcDiff -NoPush"
}

function Invoke-Checked {
    param(
        [string]$Label,
        [string]$Command,
        [string[]]$Arguments
    )

    Write-Host $Label
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
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

function Get-LocalAppData {
    if ($env:LOCALAPPDATA) {
        return $env:LOCALAPPDATA
    }

    if ($env:USERPROFILE) {
        return Join-Path $env:USERPROFILE "AppData\Local"
    }

    return $null
}

function Get-LocalNodeDir {
    $localAppData = Get-LocalAppData
    if (!$localAppData) {
        return $null
    }

    $nodeRoot = Join-Path $localAppData "fruitfacts\node"
    if (!(Test-Path -LiteralPath $nodeRoot)) {
        return $null
    }

    $nodes = @(Get-ChildItem -LiteralPath $nodeRoot -Directory -Filter "node-v*-win-x64" |
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

function Enable-NodeForCurrentProcess {
    $pathNode = Get-Command node -ErrorAction SilentlyContinue
    if ($pathNode) {
        return Split-Path -Parent $pathNode.Source
    }

    $localNodeDir = Get-LocalNodeDir
    if ($localNodeDir) {
        Add-PathEntry -PathEntry $localNodeDir

        $npmGlobalDir = Join-Path $env:APPDATA "npm"
        Add-PathEntry -PathEntry $npmGlobalDir
        return $localNodeDir
    }

    return $null
}

function Get-WebsiteThumbnailDependencyIssue {
    param(
        [string]$RepoRoot
    )

    if (!(Enable-NodeForCurrentProcess)) {
        return "node was not found on PATH"
    }

    $nodeModules = Join-Path $RepoRoot "backend\web_screenshot\node_modules"
    if (!(Test-Path -LiteralPath $nodeModules)) {
        return "backend\web_screenshot\node_modules was not found"
    }

    return $null
}

function Install-WebsiteThumbnailDependencies {
    param(
        [string]$RepoRoot
    )

    $installer = Join-Path $RepoRoot "helper_scripts\web_screenshot_install_or_update.ps1"
    if (!(Test-Path -LiteralPath $installer)) {
        throw "Web screenshot install helper was not found at $installer"
    }

    & $installer

    $null = Enable-NodeForCurrentProcess
}

function Get-RepoRoot {
    return (Resolve-Path (Join-Path $PSScriptRoot ".")).Path
}

function Get-RelativePath {
    param(
        [string]$Root,
        [string]$Path
    )

    $rootFull = [System.IO.Path]::GetFullPath($Root)
    if (!$rootFull.EndsWith([System.IO.Path]::DirectorySeparatorChar)) {
        $rootFull += [System.IO.Path]::DirectorySeparatorChar
    }
    $pathFull = [System.IO.Path]::GetFullPath($Path)
    $rootUri = [System.Uri]::new($rootFull)
    $pathUri = [System.Uri]::new($pathFull)
    return [System.Uri]::UnescapeDataString($rootUri.MakeRelativeUri($pathUri).ToString())
}

function Get-NewAssetFiles {
    param(
        [string]$RepoRoot,
        [string]$SearchRoot,
        [string]$Filter
    )

    $root = Join-Path $RepoRoot $SearchRoot
    if (!(Test-Path -LiteralPath $root)) {
        return @()
    }

    $files = [System.Collections.Generic.List[string]]::new()
    Get-ChildItem -LiteralPath $root -Recurse -File -Filter $Filter | ForEach-Object {
        if (!(Test-Path -LiteralPath "$($_.FullName).dvc")) {
            $files.Add((Get-RelativePath -Root $RepoRoot -Path $_.FullName))
        }
    }
    return $files.ToArray()
}

function Invoke-DvcAddFiles {
    param(
        [string]$DvcExe,
        [string[]]$Files,
        [string]$Label,
        [string]$EmptyMessage
    )

    if (!$Files -or $Files.Count -eq 0) {
        Write-Host $EmptyMessage
        return
    }

    Write-Host "${Label}: $($Files.Count)"
    $batch = @()
    foreach ($file in $Files) {
        $batch += $file
        if ($batch.Count -ge 40) {
            Invoke-Checked $Label $DvcExe (@("add") + $batch)
            $batch = @()
        }
    }
    if ($batch.Count -gt 0) {
        Invoke-Checked $Label $DvcExe (@("add") + $batch)
    }
}

function Get-LocalDvcExe {
    $localAppData = $env:LOCALAPPDATA
    if (!$localAppData -and $env:USERPROFILE) {
        $localAppData = Join-Path $env:USERPROFILE "AppData\Local"
    }
    if (!$localAppData) {
        return $null
    }
    return Join-Path $localAppData "fruitfacts\dvc-venv\Scripts\dvc.exe"
}

function Get-DvcExe {
    param(
        [string]$RepoRoot
    )

    $localDvc = Get-LocalDvcExe
    if ($localDvc -and (Test-Path $localDvc)) {
        return $localDvc
    }

    if (!$SkipDvcInstall) {
        $installer = Join-Path $RepoRoot "helper_scripts\dvc_install_or_update.ps1"
        if (Test-Path $installer) {
            & $installer
            if ($LASTEXITCODE -ne 0) {
                throw "DVC install helper failed with exit code $LASTEXITCODE"
            }
            if ($localDvc -and (Test-Path $localDvc)) {
                return $localDvc
            }
        }
    }

    $pathDvc = Get-Command dvc -ErrorAction SilentlyContinue
    if ($pathDvc) {
        return $pathDvc.Source
    }

    throw "DVC executable was not found"
}

if ($Help) {
    Write-Usage
    exit 0
}

$repoRoot = Get-RepoRoot
Push-Location $repoRoot
try {
    if (!$SkipReferencePdfDiscovery) {
        Invoke-Checked "Discovering reference PDF companions" "python" @("helper_scripts\archive_reference_pdfs.py")
    }

    if (!$SkipThumbnails) {
        $thumbnailArgs = @()
        if ($RedoAllThumbnails) {
            $thumbnailArgs += "--redo_all"
        }

        Push-Location (Join-Path $repoRoot "backend")
        try {
            if (!$SkipPdfThumbnails) {
                Invoke-Checked "Generating missing PDF thumbnails" "cargo" (@("run", "--bin", "pdf_to_thumbnail", "--") + $thumbnailArgs)
            }
            if (!$SkipWebsiteThumbnails) {
                $websiteIssue = Get-WebsiteThumbnailDependencyIssue -RepoRoot $repoRoot
                if ($websiteIssue -and !$SkipNodeInstall) {
                    Write-Host "Preparing website thumbnail dependencies because $websiteIssue"
                    Install-WebsiteThumbnailDependencies -RepoRoot $repoRoot
                    $websiteIssue = Get-WebsiteThumbnailDependencyIssue -RepoRoot $repoRoot
                }
                if ($websiteIssue) {
                    if ($RequireWebsiteThumbnails) {
                        throw "$websiteIssue; run helper_scripts\web_screenshot_install_or_update.ps1"
                    }
                    Write-Host "Skipping website thumbnails because $websiteIssue"
                    Write-Host "Run helper_scripts\web_screenshot_install_or_update.ps1 to enable website thumbnails"
                } else {
                    Invoke-Checked "Generating missing website thumbnails" "cargo" (@("run", "--bin", "web_thumbnails", "--") + $thumbnailArgs)
                }
            }
        } finally {
            Pop-Location
        }
    }

    if (!$SkipDvc) {
        $dvcExe = Get-DvcExe -RepoRoot $repoRoot
        if ($RefreshAllDvcPointers) {
            Write-Host "Refreshing all DVC pointers for PDF and JPG assets"
            Invoke-Checked "Adding reference PDFs to DVC" $dvcExe @("add", "--glob", "plant_database\references\**\*.pdf")
            Invoke-Checked "Adding generated thumbnails to DVC" $dvcExe @("add", "--glob", "frontend\public\data\**\*.jpg")
        } else {
            $newPdfFiles = @(Get-NewAssetFiles -RepoRoot $repoRoot -SearchRoot "plant_database\references" -Filter "*.pdf")
            $newJpgFiles = @(Get-NewAssetFiles -RepoRoot $repoRoot -SearchRoot "frontend\public\data" -Filter "*.jpg")
            Invoke-DvcAddFiles $dvcExe $newPdfFiles "Adding new reference PDFs to DVC" "No new reference PDFs need DVC add"
            Invoke-DvcAddFiles $dvcExe $newJpgFiles "Adding new generated thumbnails to DVC" "No new generated thumbnails need DVC add"
        }

        if ($ShowDvcDiff) {
            Invoke-Checked "Showing DVC diff" $dvcExe @("diff")
        }
        if (!$NoPush) {
            Invoke-Checked "Pushing DVC cache changes" $dvcExe @("push")
        } else {
            Write-Host "Skipping DVC push because -NoPush was set"
        }
        Write-Host "Next step is to git add and commit changed .dvc files"
    }
} finally {
    Pop-Location
}
