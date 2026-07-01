param(
    [int]$Port = 3030,
    [string]$BackendBase = "http://127.0.0.1:3001",
    [int]$TimeoutSec = 30,
    [int]$Limit = 0,
    [switch]$SkipBuild,
    [switch]$KeepServer
)

$ErrorActionPreference = "Stop"

$frontendRoot = $PSScriptRoot
$repoRoot = Split-Path -Parent $frontendRoot
$referenceRoot = Join-Path $repoRoot "plant_database\references"
$serverScript = Join-Path $frontendRoot "start_local_test_server.ps1"

function Convert-NameToRoutePath {
    param([string]$Name)
    $segments = $Name -split "/"
    $encoded = foreach ($segment in $segments) {
        [System.Uri]::EscapeDataString(($segment -replace " ", "_"))
    }
    return $encoded -join "/"
}

function Add-Route {
    param(
        [System.Collections.Generic.HashSet[string]]$Routes,
        [string]$Route
    )
    [void]$Routes.Add($Route)
}

function Stop-ProcessTree {
    param([int]$ProcessId)
    $children = Get-CimInstance Win32_Process -Filter "ParentProcessId = $ProcessId" -ErrorAction SilentlyContinue
    foreach ($child in $children) {
        Stop-ProcessTree -ProcessId ([int]$child.ProcessId)
    }
    Stop-Process -Id $ProcessId -Force -ErrorAction SilentlyContinue
}

function Read-LogTail {
    param([string]$Path)
    if (Test-Path -LiteralPath $Path) {
        Get-Content -LiteralPath $Path -Tail 120
    }
}

if (!(Test-Path -LiteralPath $serverScript)) {
    throw "Frontend server helper was not found at $serverScript"
}
if (!(Test-Path -LiteralPath $referenceRoot)) {
    throw "Reference root was not found at $referenceRoot"
}
if (Test-NetConnection -ComputerName "127.0.0.1" -Port $Port -InformationLevel Quiet -WarningAction SilentlyContinue) {
    throw "Port $Port is already in use"
}

$routes = [System.Collections.Generic.HashSet[string]]::new()
Add-Route $routes "/"
Add-Route $routes "/plants"
Add-Route $routes "/dirs"
Add-Route $routes "/search?searchType=base&page=1&perPage=50&orderBy=name_then_type&order=asc"

$references = Get-ChildItem -LiteralPath $referenceRoot -Recurse -File -Filter "*.json5"
foreach ($reference in $references) {
    $relative = [System.IO.Path]::GetRelativePath($referenceRoot, $reference.FullName).Replace("\", "/")
    $name = $relative.Substring(0, $relative.Length - ".json5".Length)
    Add-Route $routes ("/collections/" + (Convert-NameToRoutePath $name))

    $parts = $name -split "/"
    for ($index = 1; $index -lt $parts.Count; $index++) {
        $directory = ($parts[0..($index - 1)] -join "/")
        Add-Route $routes ("/dirs/" + (Convert-NameToRoutePath $directory))
    }
}

$routeList = @($routes | Sort-Object)
if ($Limit -gt 0) {
    $routeList = @($routeList | Select-Object -First $Limit)
}

$stdout = Join-Path $env:TEMP ("fruitfacts-route-check-{0}.out" -f ([guid]::NewGuid()))
$stderr = Join-Path $env:TEMP ("fruitfacts-route-check-{0}.err" -f ([guid]::NewGuid()))
$server = $null
$oldBackendOverride = $env:FRUITFACTS_SERVER_BACKEND_BASE

try {
    $env:FRUITFACTS_SERVER_BACKEND_BASE = $BackendBase
    $arguments = @(
        "-NoProfile",
        "-ExecutionPolicy", "Bypass",
        "-File", $serverScript,
        "-Production",
        "-Port", "$Port"
    )
    if ($SkipBuild) {
        $arguments += "-SkipBuild"
    }

    Write-Host "Starting local frontend route check on port $Port"
    $server = Start-Process -FilePath "powershell" -ArgumentList $arguments -WorkingDirectory $frontendRoot -RedirectStandardOutput $stdout -RedirectStandardError $stderr -WindowStyle Hidden -PassThru

    $deadline = (Get-Date).AddMinutes(3)
    while ((Get-Date) -lt $deadline) {
        if (Test-NetConnection -ComputerName "127.0.0.1" -Port $Port -InformationLevel Quiet -WarningAction SilentlyContinue) {
            break
        }
        if ($server.HasExited) {
            Write-Host "Server exited before listening"
            Read-LogTail $stdout
            Read-LogTail $stderr
            exit 1
        }
        Start-Sleep -Milliseconds 500
    }
    if (!(Test-NetConnection -ComputerName "127.0.0.1" -Port $Port -InformationLevel Quiet -WarningAction SilentlyContinue)) {
        Write-Host "Server did not listen before timeout"
        Read-LogTail $stdout
        Read-LogTail $stderr
        exit 1
    }

    $baseUrl = "http://127.0.0.1:$Port"
    $failures = [System.Collections.Generic.List[string]]::new()
    $checked = 0
    foreach ($route in $routeList) {
        $checked++
        if ($checked % 50 -eq 0) {
            Write-Host "Checked $checked/$($routeList.Count) routes"
        }
        $url = $baseUrl + $route
        try {
            $response = Invoke-WebRequest -UseBasicParsing -Uri $url -TimeoutSec $TimeoutSec -MaximumRedirection 5
            if ([int]$response.StatusCode -ge 400) {
                $failures.Add("$($response.StatusCode) $route")
            }
        }
        catch {
            $status = 0
            if ($_.Exception.Response -and $_.Exception.Response.StatusCode) {
                $status = [int]$_.Exception.Response.StatusCode
            }
            $failures.Add("$status $route $($_.Exception.Message)")
        }
    }

    if ($failures.Count -gt 0) {
        Write-Host "Route check failed for $($failures.Count) routes"
        $failures | Select-Object -First 40 | ForEach-Object { Write-Host $_ }
        Write-Host "Server log tail"
        Read-LogTail $stdout
        Read-LogTail $stderr
        exit 1
    }

    Write-Host "Checked $checked routes successfully"
}
finally {
    $env:FRUITFACTS_SERVER_BACKEND_BASE = $oldBackendOverride
    if ($server -and !$server.HasExited -and !$KeepServer) {
        Stop-ProcessTree -ProcessId $server.Id
    }
    if (!$KeepServer) {
        Remove-Item -LiteralPath $stdout, $stderr -Force -ErrorAction SilentlyContinue
    }
}
