#requires -Version 7.5

[CmdletBinding()]
param(
    [string]$ApplicationPath = (Join-Path $PSScriptRoot '../target/release/power-plan-sync.exe'),
    [string]$FixturePath = (Join-Path $PSScriptRoot '../target/release/examples/demo_target.exe'),
    [string]$EvidenceDirectory = (Join-Path $PSScriptRoot '../.audit/behavior-verification'),
    [string]$TargetPlanGuid,
    [string]$ExternalPlanGuid,
    [ValidateRange(10, 1000)][int]$PollMilliseconds = 50,
    [switch]$FixtureTimingSmoke,
    [switch]$AllowPowerSwitch
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$application = [IO.Path]::GetFullPath($ApplicationPath)
$fixture = [IO.Path]::GetFullPath($FixturePath)
$runId = '{0}-{1}' -f (Get-Date -Format 'yyyyMMdd-HHmmss'), ([guid]::NewGuid().ToString('N').Substring(0, 8))
$evidence = Join-Path ([IO.Path]::GetFullPath($EvidenceDirectory)) $runId
$reportPath = Join-Path $evidence 'result.json'
$transitionTimeoutMs = 10000
$results = [Collections.Generic.List[object]]::new()
$activeOwned = [Collections.Generic.List[object]]::new()
$failure = $null
$initialPlan = $null

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
    @([regex]::Matches(($output -join "`n"), '[0-9a-fA-F]{8}(?:-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}') |
        ForEach-Object { $_.Value.ToLowerInvariant() } | Select-Object -Unique)
}

function Set-ActivePlan([string]$Plan) {
    & powercfg.exe /setactive $Plan | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "L'activation du plan $Plan a échoué." }
    $observed = Read-ActivePlan
    if ($observed -ne $Plan) { throw "Le plan observé $observed diffère du plan demandé $Plan." }
}

function Wait-ForPlan([string]$Expected, [datetime]$Since, [int]$TimeoutMs = $transitionTimeoutMs) {
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

# Attend `Expected` en échouant sur tout autre plan observé que `From` : une transition entre
# deux gagnants ne passe par aucun plan intermédiaire.
function Wait-ForPlanDirect([string]$From, [string]$Expected, [datetime]$Since, [int]$TimeoutMs = $transitionTimeoutMs) {
    $deadline = $Since.AddMilliseconds($TimeoutMs)
    do {
        $observed = Read-ActivePlan
        if ($observed -eq $Expected) {
            return [math]::Round(((Get-Date) - $Since).TotalMilliseconds, 3)
        }
        if ($observed -ne $From) { throw "Plan intermédiaire $observed observé entre $From et $Expected." }
        Start-Sleep -Milliseconds $PollMilliseconds
    } while ((Get-Date) -le $deadline)
    throw "Le plan $Expected n'a pas été observé dans le délai de ${TimeoutMs} ms. Dernier plan : $observed."
}

function Assert-PlanStays([string]$Expected, [int]$DurationMs) {
    $started = Get-Date
    do {
        $observed = Read-ActivePlan
        if ($observed -ne $Expected) { throw "Le plan a changé vers $observed, $Expected était attendu." }
        Start-Sleep -Milliseconds $PollMilliseconds
    } while (((Get-Date) - $started).TotalMilliseconds -lt $DurationMs)
}

function Convert-GuidToPlanId([string]$Guid) {
    [System.Numerics.BigInteger]::Parse(
        '0' + $Guid.Replace('-', ''),
        [Globalization.NumberStyles]::AllowHexSpecifier,
        [Globalization.CultureInfo]::InvariantCulture
    )
}

function Convert-PathToUnits([string]$Path) {
    $resolved = (Resolve-Path -LiteralPath $Path).ProviderPath.ToUpperInvariant()
    @($resolved.ToCharArray() | ForEach-Object { [int][char]$_ })
}

function Write-Configuration([string]$DataDirectory, [string]$Executable, [string]$ReturnPlan) {
    Write-RulesConfiguration $DataDirectory @(@{ Executable = $Executable; Plan = $script:targetPlan }) $ReturnPlan
}

# `Rules` dans l'ordre de priorité : la première règle ouverte décide du plan.
function Write-RulesConfiguration([string]$DataDirectory, [object[]]$Rules, [string]$ReturnPlan) {
    [IO.Directory]::CreateDirectory($DataDirectory) | Out-Null
    $targetId = Convert-GuidToPlanId $Rules[0].Plan
    $returnId = Convert-GuidToPlanId $ReturnPlan
    $index = 0
    $configuration = [ordered]@{
        schema_version = 1
        rules = @($Rules | ForEach-Object {
            $index++
            [ordered]@{
                id = $index
                label = "behavior verification target $index"
                executable = Convert-PathToUnits $_.Executable
                plan = Convert-GuidToPlanId $_.Plan
                enabled = $true
            }
        })
        return_plan = $returnId
        automation_enabled = $true
        paused = $false
        allow_on_battery = $true
        start_at_login = $false
    }
    $json = $configuration | ConvertTo-Json -Depth 6
    $targetPattern = '"plan"\s*:\s*' + [regex]::Escape($targetId.ToString([Globalization.CultureInfo]::InvariantCulture))
    $returnPattern = '"return_plan"\s*:\s*' + [regex]::Escape($returnId.ToString([Globalization.CultureInfo]::InvariantCulture))
    if ($json -notmatch $targetPattern -or $json -notmatch $returnPattern) {
        throw 'PowerShell n''a pas sérialisé les identifiants de plan comme nombres JSON décimaux exacts.'
    }
    $json | Set-Content -LiteralPath (Join-Path $DataDirectory 'config.json') -Encoding utf8NoBOM
}

function Start-Owned([string]$Path, [string[]]$Arguments, [string]$LogPrefix) {
    $stdout = Join-Path $script:currentCasePath "$LogPrefix.stdout.log"
    $stderr = Join-Path $script:currentCasePath "$LogPrefix.stderr.log"
    $process = Start-Process -FilePath $Path -ArgumentList $Arguments -WindowStyle Hidden -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru
    $record = [pscustomobject]@{ Process = $process; Path = [IO.Path]::GetFullPath($Path) }
    $script:activeOwned.Add($record)
    $process
}

function Stop-Owned([object]$Record) {
    $process = $Record.Process
    if ($null -eq $process -or $process.HasExited) { return }
    $actual = $process.MainModule.FileName
    if (-not [string]::Equals([IO.Path]::GetFullPath($actual), $Record.Path, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refus d'arrêter le PID $($process.Id), son chemin ne correspond pas au processus lancé."
    }
    Stop-Process -Id $process.Id -Force
    $process.WaitForExit(5000) | Out-Null
}

function Stop-AllOwned {
    $cleanupErrors = [Collections.Generic.List[string]]::new()
    for ($index = $script:activeOwned.Count - 1; $index -ge 0; $index--) {
        try { Stop-Owned $script:activeOwned[$index] } catch { $cleanupErrors.Add($_.Exception.Message) }
    }
    $script:activeOwned.Clear()
    if ($cleanupErrors.Count) { throw ($cleanupErrors -join ' ') }
}

function Start-Application([string]$DataDirectory, [string]$Name = 'application') {
    $quotedDataDirectory = '"' + $DataDirectory + '"'
    $process = Start-Owned $script:application @('--data-dir', $quotedDataDirectory, '--start-minimized') $Name
    Start-Sleep -Milliseconds 500
    if ($process.HasExited) { throw "PowerPlanSync s'est arrêté avec le code $($process.ExitCode)." }
    $process
}

function Find-OwnedChild([string]$Path, [datetime]$Since) {
    $expected = [IO.Path]::GetFullPath($Path)
    $deadline = (Get-Date).AddSeconds(5)
    do {
        foreach ($process in @(Get-Process -Name ([IO.Path]::GetFileNameWithoutExtension($Path)) -ErrorAction SilentlyContinue)) {
            try {
                if ($process.StartTime -ge $Since -and [string]::Equals([IO.Path]::GetFullPath($process.MainModule.FileName), $expected, [StringComparison]::OrdinalIgnoreCase)) {
                    # Retenir le handle avant la sortie permet à .NET de lire sa véritable ExitTime.
                    $null = $process.SafeHandle
                    $record = [pscustomobject]@{ Process = $process; Path = $expected }
                    $script:activeOwned.Add($record)
                    return $process
                }
            } catch {}
        }
        Start-Sleep -Milliseconds $PollMilliseconds
    } while ((Get-Date) -le $deadline)
    throw "Le processus enfant $expected n'a pas été retrouvé."
}

function Invoke-Case([string]$Name, [scriptblock]$Body) {
    $script:currentCasePath = Join-Path $script:evidence $Name
    [IO.Directory]::CreateDirectory($script:currentCasePath) | Out-Null
    $started = (Get-Date).ToUniversalTime()
    $caseResult = [ordered]@{ name = $Name; started_at = $started.ToString('o'); passed = $false; measurements = [ordered]@{}; error = $null }
    $caseError = $null
    try {
        if ((Read-ActivePlan) -ne $script:initialPlan) { Set-ActivePlan $script:initialPlan }
        & $Body $caseResult.measurements
        $caseResult.passed = $true
    } catch {
        $caseError = $_.Exception.Message
        $caseResult.error = $caseError
    } finally {
        try { Stop-AllOwned } catch { $caseError = if ($caseError) { "$caseError Nettoyage : $($_.Exception.Message)" } else { $_.Exception.Message } }
        try { Set-ActivePlan $script:initialPlan } catch { $caseError = if ($caseError) { "$caseError Restauration : $($_.Exception.Message)" } else { $_.Exception.Message } }
        $caseResult.finished_at = (Get-Date).ToUniversalTime().ToString('o')
        $caseResult.final_plan = try { Read-ActivePlan } catch { $null }
        if ($caseError) { $caseResult.passed = $false; $caseResult.error = $caseError }
        $script:results.Add($caseResult)
    }
    Write-Output "$Name : $(if ($caseResult.passed) { 'réussi' } else { 'échec' })"
    if ($caseError) { throw $caseError }
}

if ($FixtureTimingSmoke) {
    if (-not (Test-Path -LiteralPath $fixture -PathType Leaf)) { throw "Témoin manquant : $fixture" }
    $script:currentCasePath = Join-Path $evidence 'fixture-timing'
    [IO.Directory]::CreateDirectory($script:currentCasePath) | Out-Null
    $child = Join-Path $script:currentCasePath 'timing-child.exe'
    Copy-Item -LiteralPath $fixture -Destination $child
    try {
        $launcher = Start-Owned $fixture @('--launch-child', ('"' + $child + '"'), '1000', '3000') 'launcher'
        $childProcess = Find-OwnedChild $child $launcher.StartTime
        $childStarted = $childProcess.get_StartTime()
        $childProcess.WaitForExit()
        $childProcess.Refresh()
        $childExited = $childProcess.get_ExitTime()
        if ($childExited -le $childStarted) { throw 'La date réelle de sortie ne suit pas la création.' }
        if ($launcher.HasExited) { throw 'Le lanceur doit encore fonctionner après la sortie de son enfant.' }
        [ordered]@{
            mode = 'fixture-timing-only'
            passed = $true
            child_pid = $childProcess.Id
            child_started_utc = $childStarted.ToUniversalTime().ToString('o')
            child_exited_utc = $childExited.ToUniversalTime().ToString('o')
            child_lifetime_ms = ($childExited - $childStarted).TotalMilliseconds
            launcher_running_after_child_exit = $true
            timestamp_source = 'System.Diagnostics.Process with retained SafeHandle'
        } | ConvertTo-Json | Set-Content -LiteralPath $reportPath -Encoding utf8NoBOM
        $launcher.WaitForExit()
    } finally {
        Stop-AllOwned
    }
    Write-Output "Horodatage du témoin vérifié sans PowerPlanSync ni accès aux plans. Preuve : $reportPath"
    return
}

foreach ($path in @($application, $fixture)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Exécutable release manquant : $path" }
}
if (-not $AllowPowerSwitch) { throw 'Relancez avec -AllowPowerSwitch pour autoriser les changements réels de plan.' }
if (Get-Process -Name 'power-plan-sync' -ErrorAction SilentlyContinue) {
    throw 'PowerPlanSync est déjà en cours. Fermez toutes ses instances avant cette vérification.'
}

$initialPlan = Read-ActivePlan
$installedPlans = @(Read-InstalledPlans)
$targetPlan = if ($TargetPlanGuid) { $TargetPlanGuid.ToLowerInvariant() } else { $installedPlans | Where-Object { $_ -ne $initialPlan } | Select-Object -First 1 }
$externalPlan = if ($ExternalPlanGuid) { $ExternalPlanGuid.ToLowerInvariant() } else { $installedPlans | Where-Object { $_ -ne $initialPlan -and $_ -ne $targetPlan } | Select-Object -First 1 }
foreach ($plan in @($targetPlan, $externalPlan)) {
    if (-not $plan -or $plan -notin $installedPlans) { throw 'La vérification exige trois plans installés distincts.' }
}
if (@(@($initialPlan, $targetPlan, $externalPlan) | Select-Object -Unique).Count -ne 3) {
    throw 'Les plans initial, cible et externe doivent être distincts.'
}
[IO.Directory]::CreateDirectory($evidence) | Out-Null
[ordered]@{ captured_at = (Get-Date).ToUniversalTime().ToString('o'); initial_plan = $initialPlan; target_plan = $targetPlan; external_plan = $externalPlan } |
    ConvertTo-Json | Set-Content -LiteralPath (Join-Path $evidence 'initial-journal.json') -Encoding utf8NoBOM

try {
    Invoke-Case 'two-instances' {
        param($measurements)
        $data = Join-Path $script:currentCasePath 'data'
        Write-Configuration $data $script:fixture $script:initialPlan
        Start-Application $data | Out-Null
        $first = Start-Owned $script:fixture @('--sleep-ms', '10000') 'fixture-1'
        $measurements.activation_ms = Wait-ForPlan $script:targetPlan $first.StartTime
        $second = Start-Owned $script:fixture @('--sleep-ms', '20000') 'fixture-2'
        $first.WaitForExit()
        Assert-PlanStays $script:targetPlan 1500
        $measurements.plan_after_first_exit = Read-ActivePlan
        $second.WaitForExit(); $second.Refresh()
        $measurements.return_ms = Wait-ForPlan $script:initialPlan $second.ExitTime
    }

    Invoke-Case 'homonym-other-path' {
        param($measurements)
        $data = Join-Path $script:currentCasePath 'data'
        $copyDirectory = Join-Path $script:currentCasePath 'homonym'
        [IO.Directory]::CreateDirectory($copyDirectory) | Out-Null
        $homonym = Join-Path $copyDirectory 'demo_target.exe'
        Copy-Item -LiteralPath $script:fixture -Destination $homonym
        Write-Configuration $data $script:fixture $script:initialPlan
        Start-Application $data | Out-Null
        $other = Start-Owned $homonym @('--sleep-ms', '10000') 'homonym'
        Assert-PlanStays $script:initialPlan 10000
        $other.WaitForExit()
        $measurements.observed_plan = Read-ActivePlan
    }

    Invoke-Case 'launcher-child' {
        param($measurements)
        $data = Join-Path $script:currentCasePath 'data'
        $child = Join-Path $script:currentCasePath 'child-target.exe'
        Copy-Item -LiteralPath $script:fixture -Destination $child
        Write-Configuration $data $child $script:initialPlan
        Start-Application $data | Out-Null
        $launcher = Start-Owned $script:fixture @('--launch-child', ('"' + $child + '"'), '10000', '20000') 'launcher'
        $measurements.activation_ms = Wait-ForPlan $script:targetPlan $launcher.StartTime
        $childProcess = Find-OwnedChild $child $launcher.StartTime
        $measurements.child_started_utc = $childProcess.get_StartTime().ToUniversalTime().ToString('o')
        $childProcess.WaitForExit(); $childProcess.Refresh()
        $childExited = $childProcess.get_ExitTime()
        $measurements.child_exited_utc = $childExited.ToUniversalTime().ToString('o')
        $measurements.return_ms = Wait-ForPlan $script:initialPlan $childExited
        if ($launcher.HasExited) { throw 'Le lanceur s''est arrêté avant le retour du plan.' }
        $measurements.launcher_running_after_return = $true
        $launcher.WaitForExit()
    }

    Invoke-Case 'forced-fixture-stop' {
        param($measurements)
        $data = Join-Path $script:currentCasePath 'data'
        Write-Configuration $data $script:fixture $script:initialPlan
        Start-Application $data | Out-Null
        $target = Start-Owned $script:fixture @('--sleep-ms', '60000') 'fixture'
        $measurements.activation_ms = Wait-ForPlan $script:targetPlan $target.StartTime
        $record = $script:activeOwned | Where-Object { $_.Process.Id -eq $target.Id } | Select-Object -First 1
        Stop-Owned $record
        $target.Refresh()
        $measurements.return_ms = Wait-ForPlan $script:initialPlan $target.ExitTime
    }

    Invoke-Case 'manual-external-override' {
        param($measurements)
        $data = Join-Path $script:currentCasePath 'data'
        Write-Configuration $data $script:fixture $script:initialPlan
        Start-Application $data | Out-Null
        $target = Start-Owned $script:fixture @('--sleep-ms', '10000') 'fixture'
        $measurements.activation_ms = Wait-ForPlan $script:targetPlan $target.StartTime
        Set-ActivePlan $script:externalPlan
        $measurements.manual_plan = Read-ActivePlan
        $target.WaitForExit(); $target.Refresh()
        Assert-PlanStays $script:externalPlan 5500
        $measurements.plan_after_last_exit = Read-ActivePlan
    }

    Invoke-Case 'crash-restart-recovery' {
        param($measurements)
        $data = Join-Path $script:currentCasePath 'data'
        Write-Configuration $data $script:fixture $script:initialPlan
        $app = Start-Application $data 'application-before-crash'
        $target = Start-Owned $script:fixture @('--sleep-ms', '10000') 'fixture'
        $measurements.activation_ms = Wait-ForPlan $script:targetPlan $target.StartTime
        $appRecord = $script:activeOwned | Where-Object { $_.Process.Id -eq $app.Id } | Select-Object -First 1
        Stop-Owned $appRecord
        $target.WaitForExit()
        $measurements.plan_after_crash_and_exit = Read-ActivePlan
        if ($measurements.plan_after_crash_and_exit -ne $script:targetPlan) { throw 'Le plan cible n est pas resté actif après le crash.' }
        Write-Configuration $data $script:fixture $script:externalPlan
        $restartTime = Get-Date
        Start-Application $data 'application-after-crash' | Out-Null
        $measurements.recovery_ms = Wait-ForPlan $script:initialPlan $restartTime
        $measurements.recovered_plan = Read-ActivePlan
    }

    Invoke-Case 'two-rules-priority' {
        param($measurements)
        $data = Join-Path $script:currentCasePath 'data'
        $first = Join-Path $script:currentCasePath 'first\demo_target.exe'
        $second = Join-Path $script:currentCasePath 'second\demo_target.exe'
        foreach ($copy in @($first, $second)) {
            [IO.Directory]::CreateDirectory((Split-Path -Parent $copy)) | Out-Null
            Copy-Item -LiteralPath $script:fixture -Destination $copy
        }
        Write-RulesConfiguration $data @(
            @{ Executable = $first; Plan = $script:targetPlan },
            @{ Executable = $second; Plan = $script:externalPlan }
        ) $script:initialPlan
        Start-Application $data | Out-Null
        $low = Start-Owned $second @('--sleep-ms', '30000') 'second-rule'
        $measurements.second_rule_activation_ms = Wait-ForPlan $script:externalPlan $low.StartTime
        $high = Start-Owned $first @('--sleep-ms', '10000') 'first-rule'
        $measurements.first_rule_takeover_ms = Wait-ForPlanDirect $script:externalPlan $script:targetPlan $high.StartTime
        $high.WaitForExit(); $high.Refresh()
        $measurements.back_to_second_rule_ms = Wait-ForPlanDirect $script:targetPlan $script:externalPlan $high.ExitTime
        $record = $script:activeOwned | Where-Object { $_.Process.Id -eq $low.Id } | Select-Object -First 1
        Stop-Owned $record
        $low.Refresh()
        $measurements.return_ms = Wait-ForPlanDirect $script:externalPlan $script:initialPlan $low.ExitTime
    }
} catch {
    $failure = $_.Exception.Message
} finally {
    try { Stop-AllOwned } catch { $failure = if ($failure) { "$failure Nettoyage final : $($_.Exception.Message)" } else { $_.Exception.Message } }
    if ($initialPlan) {
        try { Set-ActivePlan $initialPlan } catch { $failure = if ($failure) { "$failure Restauration finale : $($_.Exception.Message)" } else { $_.Exception.Message } }
    }
    $report = [ordered]@{
        schema_version = 1
        finished_at = (Get-Date).ToUniversalTime().ToString('o')
        application_path = $application
        fixture_path = $fixture
        initial_plan = $initialPlan
        target_plan = $targetPlan
        external_plan = $externalPlan
        transition_timeout_ms = $transitionTimeoutMs
        expectations = [ordered]@{
            two_instances = 'target after first exit, initial after last exit'
            homonym_other_path = 'initial throughout'
            launcher_child = 'initial after child exit while launcher still runs'
            forced_fixture_stop = 'initial after forced target exit'
            manual_external_override = 'external after last target exit'
            crash_restart_recovery = 'initial recovery plan captured before the crash'
            two_rules_priority = 'second rule, first rule, second rule after grace, then initial, never an intermediate plan'
        }
        passed = -not $failure -and $results.Count -eq 7 -and @($results | Where-Object { -not $_.passed }).Count -eq 0
        failure = $failure
        cases = $results
    }
    $report | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $reportPath -Encoding utf8NoBOM
}

if ($failure) { throw $failure }
Write-Output "Vérification comportementale réussie. Preuve : $reportPath"
