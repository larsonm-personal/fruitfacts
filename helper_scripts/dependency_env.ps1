param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("dvc", "node")]
    [string]$Name,
    [string]$RepoRoot,
    [switch]$EmitBatchEnv,
    [switch]$Quiet,
    [switch]$SkipVersionCheck
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-LocalAppData {
    if ($env:LOCALAPPDATA) {
        return $env:LOCALAPPDATA
    }
    if ($env:USERPROFILE) {
        return Join-Path $env:USERPROFILE "AppData\Local"
    }
    return $null
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

function Get-CommandPath {
    param(
        [string]$Command
    )

    $found = Get-Command $Command -ErrorAction SilentlyContinue
    if ($found -and $found.Source) {
        return $found.Source
    }
    return $null
}

function Add-ExistingCandidate {
    param(
        [System.Collections.Generic.List[object]]$Candidates,
        [string]$Path,
        [string]$Source
    )

    if ($Path -and (Test-Path -LiteralPath $Path -PathType Leaf)) {
        $Candidates.Add([pscustomobject]@{ Path = (Resolve-Path -LiteralPath $Path).Path; Source = $Source })
    }
}

function Add-WildcardCandidates {
    param(
        [System.Collections.Generic.List[object]]$Candidates,
        [string]$Pattern,
        [string]$Source
    )

    if (!$Pattern) {
        return
    }

    $matches = @(Get-Item -Path $Pattern -ErrorAction SilentlyContinue |
        Where-Object { -not $_.PSIsContainer } |
        Sort-Object FullName -Descending)
    foreach ($match in $matches) {
        $Candidates.Add([pscustomobject]@{ Path = $match.FullName; Source = $Source })
    }
}

function Test-Candidate {
    param(
        [string]$Path,
        [string]$Argument
    )

    if ($SkipVersionCheck) {
        return $true
    }

    try {
        & $Path $Argument *> $null
        return $LASTEXITCODE -eq 0
    } catch {
        return $false
    }
}

function Get-DvcCandidates {
    param(
        [string]$RepoRoot
    )

    $candidates = [System.Collections.Generic.List[object]]::new()
    $localAppData = Get-LocalAppData

    Add-ExistingCandidate $candidates $env:DVC_EXE "DVC_EXE"
    if ($localAppData) {
        Add-ExistingCandidate $candidates (Join-Path $localAppData "fruitfacts\dvc-venv\Scripts\dvc.exe") "fruitfacts local DVC virtualenv"
    }
    if ($RepoRoot) {
        Add-ExistingCandidate $candidates (Join-Path $RepoRoot ".venv\Scripts\dvc.exe") "repo .venv"
        Add-ExistingCandidate $candidates (Join-Path $RepoRoot "venv\Scripts\dvc.exe") "repo venv"
    }
    Add-ExistingCandidate $candidates (Get-CommandPath "dvc") "PATH"
    if ($env:APPDATA) {
        Add-WildcardCandidates $candidates (Join-Path $env:APPDATA "Python\Python*\Scripts\dvc.exe") "user Python scripts"
    }
    if ($localAppData) {
        Add-WildcardCandidates $candidates (Join-Path $localAppData "Programs\Python\Python*\Scripts\dvc.exe") "local Python scripts"
        Add-ExistingCandidate $candidates (Join-Path $localAppData "pipx\venvs\dvc\Scripts\dvc.exe") "local pipx"
    }
    if ($env:USERPROFILE) {
        Add-ExistingCandidate $candidates (Join-Path $env:USERPROFILE ".local\bin\dvc.exe") "user local bin"
        Add-ExistingCandidate $candidates (Join-Path $env:USERPROFILE "pipx\venvs\dvc\Scripts\dvc.exe") "user pipx"
        Add-ExistingCandidate $candidates (Join-Path $env:USERPROFILE "miniconda3\Scripts\dvc.exe") "miniconda"
        Add-ExistingCandidate $candidates (Join-Path $env:USERPROFILE "anaconda3\Scripts\dvc.exe") "anaconda"
    }
    if ($env:ProgramFiles) {
        Add-ExistingCandidate $candidates (Join-Path $env:ProgramFiles "DVC\dvc.exe") "Program Files"
    }

    return $candidates
}

function Get-NodeCandidates {
    param(
        [string]$RepoRoot
    )

    $candidates = [System.Collections.Generic.List[object]]::new()
    $localAppData = Get-LocalAppData

    Add-ExistingCandidate $candidates $env:NODE_EXE "NODE_EXE"
    Add-ExistingCandidate $candidates (Get-CommandPath "node") "PATH"
    if ($localAppData) {
        Add-WildcardCandidates $candidates (Join-Path $localAppData "fruitfacts\node\node-v*-win-x64\node.exe") "fruitfacts local Node"
        Add-WildcardCandidates $candidates (Join-Path $localAppData "Programs\nodejs\node.exe") "local nodejs"
    }
    if ($env:ProgramFiles) {
        Add-ExistingCandidate $candidates (Join-Path $env:ProgramFiles "nodejs\node.exe") "Program Files"
    }
    if (${env:ProgramFiles(x86)}) {
        Add-ExistingCandidate $candidates (Join-Path ${env:ProgramFiles(x86)} "nodejs\node.exe") "Program Files x86"
    }

    return $candidates
}

function Resolve-Dependency {
    param(
        [string]$Name,
        [string]$RepoRoot
    )

    $argument = "--version"
    if ($Name -eq "dvc") {
        $candidates = Get-DvcCandidates -RepoRoot $RepoRoot
    } elseif ($Name -eq "node") {
        $candidates = Get-NodeCandidates -RepoRoot $RepoRoot
    } else {
        throw "Unsupported dependency $Name"
    }

    $seen = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
    foreach ($candidate in $candidates) {
        if (!$seen.Add($candidate.Path)) {
            continue
        }
        if (Test-Candidate -Path $candidate.Path -Argument $argument) {
            return $candidate
        }
    }

    throw "$Name was not found in known install locations"
}

if (!$RepoRoot) {
    $RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
}

$resolved = Resolve-Dependency -Name $Name -RepoRoot $RepoRoot
$toolDir = Split-Path -Parent $resolved.Path
Add-PathEntry -PathEntry $toolDir

if ($Name -eq "dvc") {
    $env:DVC_EXE = $resolved.Path
} elseif ($Name -eq "node") {
    $env:NODE_EXE = $resolved.Path
}

if ($EmitBatchEnv) {
    if ($Name -eq "dvc") {
        Write-Output "set `"DVC_EXE=$($resolved.Path)`""
    } elseif ($Name -eq "node") {
        Write-Output "set `"NODE_EXE=$($resolved.Path)`""
    }
    Write-Output "set `"PATH=$toolDir;%PATH%`""
} elseif (!$Quiet) {
    Write-Output $resolved.Path
}
