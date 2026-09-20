[CmdletBinding()]
param(
    [string]$ApplicationPath = (Join-Path $PSScriptRoot '../target/release/power-plan-sync.exe'),
    [string]$FixturePath = (Join-Path $PSScriptRoot '../target/release/examples/demo_target.exe'),
    [string]$EvidenceDirectory = (Join-Path $PSScriptRoot '../.audit/windows-verification'),
    [string]$TargetPlanGuid,
    [ValidateRange(1, 100)][int]$Cycles = 100,
    [ValidateRange(10, 300)][int]$MinimumProcessSeconds = 10,
    [ValidateRange(10, 1000)][int]$PollMilliseconds = 50,
    [switch]$AllowPowerSwitch
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$application = [IO.Path]::GetFullPath($ApplicationPath)
$fixture = [IO.Path]::GetFullPath($FixturePath)
$evidenceRoot = [IO.Path]::GetFullPath($EvidenceDirectory)
$runId = '{0}-{1}' -f (Get-Date -Format 'yyyyMMdd-HHmmss'), ([guid]::NewGuid().ToString('N').Substring(0, 8))
$evidence = Join-Path $evidenceRoot $runId
$dataDirectory = Join-Path $evidence 'data'
$reportPath = Join-Path $evidence 'result.json'
$activationLimitMs = 2000
$returnLimitMs = 5000

function Read-ActivePlan {
    $output = & powercfg.exe /getactivescheme
    if ($LASTEXITCODE -ne 0) { throw 'Impossible de lire le plan Windows actif.' }
    $match = [regex]::Match(($output -join "`n"), '[0-9a-fA-F]{8}(?:-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}')
    if (-not $match.Success) { throw 'GUID absent de la réponse de powercfg.' }
    $match.Value.ToLowerInvariant()
}

function Read-InstalledPlans {
    $output = & powercfg.exe /list
    if ($LASTEXITCODE -ne 0) { throw 'Impossible de lire les plans Windows installés.' }
    [regex]::Matches(($output -join "`n"), '[0-9a-fA-F]{8}(?:-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}') |
        ForEach-Object { $_.Value.ToLowerInvariant() } |
        Select-Object -Unique
}

function Convert-GuidToPlanId([string]$Guid) {
    $hex = $Guid.Replace('-', '')
    [System.Numerics.BigInteger]::Parse(
        "0$hex",
        [Globalization.NumberStyles]::AllowHexSpecifier,
        [Globalization.CultureInfo]::InvariantCulture
    )
}

function Convert-PathToUnits([string]$Path) {
    $resolved = (Resolve-Path -LiteralPath $Path).ProviderPath.ToUpperInvariant()
    @($resolved.ToCharArray() | ForEach-Object { [int][char]$_ })
}

function Wait-ForPlan([string]$Expected, [datetime]$Since, [int]$TimeoutMs) {
    $deadline = $Since.AddMilliseconds($TimeoutMs)
    do {
        $observed = Read-ActivePlan
        if ($observed -eq $Expected) {
            return [math]::Round(((Get-Date) - $Since).TotalMilliseconds, 3)
        }
        Start-Sleep -Milliseconds $PollMilliseconds
    } while ((Get-Date) -le $deadline)
    throw "Le plan $Expected n'a pas été observé dans le délai de ${TimeoutMs} ms. Dernier plan : $observed."
}

function Get-ProcessSample([Diagnostics.Process]$Process) {
    $Process.Refresh()
    [ordered]@{
        timestamp = (Get-Date).ToUniversalTime().ToString('o')
        working_set_bytes = $Process.WorkingSet64
        private_memory_bytes = $Process.PrivateMemorySize64
        cpu_seconds = [math]::Round($Process.TotalProcessorTime.TotalSeconds, 6)
        handles = $Process.HandleCount
    }
}

function Stop-OwnedProcess([Diagnostics.Process]$Process, [string]$ExpectedPath) {
    if ($null -eq $Process -or $Process.HasExited) { return }
    $actualPath = $Process.MainModule.FileName
    if (-not [string]::Equals(
        [IO.Path]::GetFullPath($actualPath),
        [IO.Path]::GetFullPath($ExpectedPath),
        [StringComparison]::OrdinalIgnoreCase
    )) {
        throw "Refus d'arrêter le PID $($Process.Id), son chemin ne correspond pas à l'exécutable lancé."
    }
    Stop-Process -Id $Process.Id -Force
    $Process.WaitForExit(5000) | Out-Null
}

foreach ($path in @($application, $fixture)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Exécutable release manquant : $path"
    }
}
if (-not $AllowPowerSwitch) {
    throw 'Ce harnais change réellement le plan actif. Relancez avec -AllowPowerSwitch pour autoriser ces écritures.'
}
if (Get-Process -Name 'power-plan-sync' -ErrorAction SilentlyContinue) {
    throw 'PowerPlanSync est déjà en cours. Fermez toutes ses instances avant cette vérification.'
}

$initialPlan = Read-ActivePlan
$installedPlans = @(Read-InstalledPlans)
if ($TargetPlanGuid) {
    $targetPlan = $TargetPlanGuid.ToLowerInvariant()
    if ($targetPlan -notmatch '^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$') {
        throw 'TargetPlanGuid doit être un GUID de plan Windows.'
    }
    if ($targetPlan -notin $installedPlans) { throw "Le plan cible $targetPlan n'est pas installé." }
} else {
    $targetPlan = $installedPlans | Where-Object { $_ -ne $initialPlan } | Select-Object -First 1
}
if (-not $targetPlan -or $targetPlan -eq $initialPlan) {
    throw 'La vérification exige un plan cible installé distinct du plan initial.'
}

[IO.Directory]::CreateDirectory($dataDirectory) | Out-Null
$initialRecord = [ordered]@{
    captured_at = (Get-Date).ToUniversalTime().ToString('o')
    initial_plan = $initialPlan
    target_plan = $targetPlan
}
$initialRecord | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $evidence 'initial-plan.json') -Encoding utf8NoBOM
$configuration = [ordered]@{
    schema_version = 1
    rules = @([ordered]@{
        id = 1
        label = 'demo_target verification'
        executable = Convert-PathToUnits $fixture
        plan = Convert-GuidToPlanId $targetPlan
        enabled = $true
    })
    return_plan = Convert-GuidToPlanId $initialPlan
    automation_enabled = $true
    paused = $false
    allow_on_battery = $true
    start_at_login = $false
}
$configuration | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $dataDirectory 'config.json') -Encoding utf8NoBOM

$applicationProcess = $null
$fixtureProcess = $null
$results = [Collections.Generic.List[object]]::new()
$failure = $null
$confirmedPlan = $null
$startedAt = (Get-Date).ToUniversalTime()
try {
    $quotedDataDirectory = '"' + $dataDirectory + '"'
    $applicationProcess = Start-Process -FilePath $application -ArgumentList @('--data-dir', $quotedDataDirectory, '--start-minimized') -PassThru
    Start-Sleep -Milliseconds 500
    if ($applicationProcess.HasExited) { throw "PowerPlanSync s'est arrêté avec le code $($applicationProcess.ExitCode)." }

    for ($cycle = 1; $cycle -le $Cycles; $cycle++) {
        if ((Read-ActivePlan) -ne $initialPlan) {
            throw "Le cycle $cycle ne commence pas sur le plan initial $initialPlan."
        }
        $fixtureProcess = Start-Process -FilePath $fixture -ArgumentList @('--sleep-ms', "$(1000 * $MinimumProcessSeconds)") -PassThru
        $processStartTime = $fixtureProcess.StartTime
        $activationMs = Wait-ForPlan $targetPlan $processStartTime $activationLimitMs
        $sample = Get-ProcessSample $applicationProcess
        $fixtureProcess.WaitForExit()
        $fixtureProcess.Refresh()
        $exitTime = $fixtureProcess.ExitTime
        $fixtureExitCode = $fixtureProcess.ExitCode
        $fixtureProcess = $null
        if ($fixtureExitCode -ne 0) { throw "demo_target a quitté avec le code $fixtureExitCode au cycle $cycle." }
        $returnMs = Wait-ForPlan $initialPlan $exitTime $returnLimitMs
        $results.Add([ordered]@{
            cycle = $cycle
            fixture_start_time = $processStartTime.ToUniversalTime().ToString('o')
            fixture_exit_time = $exitTime.ToUniversalTime().ToString('o')
            activation_ms = $activationMs
            return_ms = $returnMs
            target_plan = $targetPlan
            returned_plan = $initialPlan
            application = $sample
        })
        if ($cycle -eq 1 -or $cycle -eq $Cycles -or $cycle % 10 -eq 0) {
            Write-Output "Cycle $cycle/$Cycles : activation ${activationMs} ms, retour ${returnMs} ms"
        }
    }
} catch {
    $failure = $_.Exception.Message
} finally {
    try { Stop-OwnedProcess $fixtureProcess $fixture } catch { if (-not $failure) { $failure = $_.Exception.Message } }
    try { Stop-OwnedProcess $applicationProcess $application } catch { if (-not $failure) { $failure = $_.Exception.Message } }
    try {
        & powercfg.exe /setactive $initialPlan | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "La restauration du plan initial $initialPlan a échoué." }
        $confirmedPlan = Read-ActivePlan
        if ($confirmedPlan -ne $initialPlan) {
            throw "Le plan restauré $confirmedPlan diffère du plan initial $initialPlan."
        }
    } catch {
        if ($failure) { $failure = "$failure Restauration : $($_.Exception.Message)" } else { $failure = $_.Exception.Message }
    }

    $activationValues = @($results | ForEach-Object { $_.activation_ms } | Sort-Object)
    $returnValues = @($results | ForEach-Object { $_.return_ms } | Sort-Object)
    $percentileIndex = if ($results.Count) { [math]::Ceiling($results.Count * 0.95) - 1 } else { 0 }
    $activationP95 = if ($activationValues.Count) { $activationValues[$percentileIndex] } else { $null }
    $returnP95 = if ($returnValues.Count) { $returnValues[$percentileIndex] } else { $null }
    $passed = -not $failure -and $results.Count -eq $Cycles -and
        $activationP95 -le $activationLimitMs -and $returnP95 -le $returnLimitMs
    $report = [ordered]@{
        schema_version = 1
        started_at = $startedAt.ToString('o')
        finished_at = (Get-Date).ToUniversalTime().ToString('o')
        machine = $env:COMPUTERNAME
        os = [Environment]::OSVersion.VersionString
        application_path = $application
        fixture_path = $fixture
        data_directory = $dataDirectory
        initial_plan = $initialPlan
        target_plan = $targetPlan
        requested_cycles = $Cycles
        completed_cycles = $results.Count
        minimum_process_seconds = $MinimumProcessSeconds
        poll_milliseconds = $PollMilliseconds
        thresholds = [ordered]@{ activation_p95_ms = $activationLimitMs; return_p95_ms = $returnLimitMs }
        measurements = [ordered]@{ activation_p95_ms = $activationP95; return_p95_ms = $returnP95 }
        restored_plan = $confirmedPlan
        passed = $passed
        failure = $failure
        cycles = $results
    }
    [IO.Directory]::CreateDirectory($evidence) | Out-Null
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding utf8NoBOM
}

if ($failure) { throw $failure }
if (-not $passed) { throw "Seuils non respectés. Consultez $reportPath" }
Write-Output "Vérification réussie : $Cycles cycles, activation p95 ${activationP95} ms, retour p95 ${returnP95} ms."
Write-Output "Preuve : $reportPath"
