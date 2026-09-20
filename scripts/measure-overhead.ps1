[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateRange(1, [int]::MaxValue)][int]$ProcessId,
    [Parameter(Mandatory)][string]$ExecutablePath,
    [Parameter(Mandatory)][ValidateSet('hidden', 'open')][string]$WindowState,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [ValidateRange(1, 86400)][int]$DurationSeconds = 1800,
    [ValidateRange(100, 60000)][int]$SampleIntervalMilliseconds = 1000,
    [ValidateRange(0, 3600)][int]$WarmupSeconds = 10,
    [ValidateRange(-1, [int]::MaxValue)][int]$RuleCount = -1,
    [string]$DataDirectory
)

$ErrorActionPreference = 'Stop'

function Get-Statistics([double[]]$Values) {
    if ($Values.Count -eq 0) { return $null }
    $ordered = @($Values | Sort-Object)
    $total = 0.0
    foreach ($value in $Values) { $total += $value }
    [ordered]@{
        min = $ordered[0]
        max = $ordered[$ordered.Count - 1]
        mean = $total / $Values.Count
        p95 = $ordered[[Math]::Max(0, [Math]::Ceiling(0.95 * $Values.Count) - 1)]
    }
}

function Get-OperatingSystemProcessCount {
    $processes = [Diagnostics.Process]::GetProcesses()
    try { return $processes.Length }
    finally { foreach ($entry in $processes) { $entry.Dispose() } }
}

$expectedPath = [IO.Path]::GetFullPath($ExecutablePath)
$outputPath = [IO.Path]::GetFullPath($OutputDirectory)
$actualRuleCount = if ($RuleCount -ge 0) { $RuleCount } else { $null }
$configurationPath = $null
if ($DataDirectory) {
    $configurationPath = Join-Path ([IO.Path]::GetFullPath($DataDirectory)) 'config.json'
    $configuration = Get-Content -LiteralPath $configurationPath -Raw | ConvertFrom-Json
    if ($null -eq $configuration.rules) {
        throw 'La configuration fournie ne contient pas de liste de règles.'
    }
    $configuredRules = @($configuration.rules).Count
    if ($RuleCount -ge 0 -and $RuleCount -ne $configuredRules) {
        throw "Le nombre déclaré de règles ($RuleCount) diffère de la configuration ($configuredRules)."
    }
    $actualRuleCount = $configuredRules
}

$measuredProcess = Get-Process -Id $ProcessId -ErrorAction Stop
try {
    $observedPath = [IO.Path]::GetFullPath($measuredProcess.Path)
    if (-not [StringComparer]::OrdinalIgnoreCase.Equals($expectedPath, $observedPath)) {
        throw "Le PID $ProcessId ne correspond pas au chemin exact demandé. Aucune mesure commencée."
    }
    $processStart = $measuredProcess.StartTime.ToUniversalTime()
    $processName = $measuredProcess.ProcessName
    $logicalProcessors = [Environment]::ProcessorCount
    $samples = [Collections.Generic.List[object]]::new()
    $failure = $null
    $startedUtc = [DateTime]::UtcNow
    $timer = [Diagnostics.Stopwatch]::new()
    try {
        if ($WarmupSeconds -gt 0) { Start-Sleep -Seconds $WarmupSeconds }
        $measuredProcess.Refresh()
        if ($measuredProcess.HasExited) { throw 'Le processus a quitté pendant la stabilisation.' }
        $initialCpuSeconds = $measuredProcess.TotalProcessorTime.TotalSeconds
        $previousCpuSeconds = $initialCpuSeconds
        $previousElapsed = 0.0
        $startedUtc = [DateTime]::UtcNow
        $timer.Start()
        while ($true) {
            $measuredProcess.Refresh()
            if ($measuredProcess.HasExited -or $measuredProcess.StartTime.ToUniversalTime() -ne $processStart) {
                throw 'Le processus mesuré a quitté ou son identité a changé.'
            }
            $elapsed = $timer.Elapsed.TotalSeconds
            $cpuSeconds = $measuredProcess.TotalProcessorTime.TotalSeconds
            $intervalCpu = $null
            if ($samples.Count -gt 0 -and $elapsed -gt $previousElapsed) {
                $intervalCpu = 100.0 * ($cpuSeconds - $previousCpuSeconds) / ($elapsed - $previousElapsed) / $logicalProcessors
            }
            $samples.Add([pscustomobject][ordered]@{
                elapsed_seconds = $elapsed
                cpu_seconds = $cpuSeconds - $initialCpuSeconds
                cpu_percent_machine_interval = $intervalCpu
                private_mib = $measuredProcess.PrivateMemorySize64 / 1MB
                working_set_mib = $measuredProcess.WorkingSet64 / 1MB
                handles = $measuredProcess.HandleCount
                os_process_count = Get-OperatingSystemProcessCount
            })
            $previousCpuSeconds = $cpuSeconds
            $previousElapsed = $elapsed
            if ($elapsed -ge $DurationSeconds) { break }
            $intervalSeconds = $SampleIntervalMilliseconds / 1000.0
            $nextElapsed = [Math]::Min($DurationSeconds, ([Math]::Floor($timer.Elapsed.TotalSeconds / $intervalSeconds) + 1) * $intervalSeconds)
            $remainingMilliseconds = [Math]::Ceiling(($nextElapsed - $timer.Elapsed.TotalSeconds) * 1000)
            if ($remainingMilliseconds -gt 0) { Start-Sleep -Milliseconds $remainingMilliseconds }
        }
    }
    catch { $failure = $_.Exception.Message }
    finally { $timer.Stop() }

    $complete = $null -eq $failure -and $samples.Count -ge 2
    $last = if ($samples.Count -gt 0) { $samples[$samples.Count - 1] } else { $null }
    $measuredSeconds = if ($null -ne $last) { $last.elapsed_seconds } else { 0.0 }
    $cpuPercent = if ($measuredSeconds -gt 0) { 100.0 * $last.cpu_seconds / $measuredSeconds / $logicalProcessors } else { $null }
    $privateStatistics = Get-Statistics @($samples | ForEach-Object { $_.private_mib })
    $privateLimit = if ($WindowState -eq 'hidden') { 40 } else { 100 }
    $report = [ordered]@{
        schema_version = 1
        process = [ordered]@{
            id = $ProcessId
            name = $processName
            executable_path = $observedPath
            started_utc = $processStart.ToString('o')
        }
        environment = [ordered]@{
            computer_name = [Environment]::MachineName
            os_version = [Runtime.InteropServices.RuntimeInformation]::OSDescription
            logical_processors = $logicalProcessors
            logical_processor_source = 'System.Environment.ProcessorCount'
            powershell_version = $PSVersionTable.PSVersion.ToString()
        }
        operator_context = [ordered]@{
            window_state = $WindowState
            window_state_source = 'operator declaration; not inspected by this script'
            rule_count = $actualRuleCount
            rule_count_source = if ($configurationPath) { $configurationPath } elseif ($RuleCount -ge 0) { 'operator declaration' } else { 'not supplied' }
        }
        measurement = [ordered]@{
            started_utc = $startedUtc.ToString('o')
            completed_utc = [DateTime]::UtcNow.ToString('o')
            complete = $complete
            failure = $failure
            requested_duration_seconds = $DurationSeconds
            duration_seconds = $measuredSeconds
            warmup_seconds = $WarmupSeconds
            sample_interval_milliseconds = $SampleIntervalMilliseconds
            sample_count = $samples.Count
            clock = 'System.Diagnostics.Stopwatch; intervals scheduled from measurement start'
            cpu_method = 'delta TotalProcessorTime / monotonic elapsed / logical processors * 100'
            percentile_method = 'nearest rank, ceil(0.95 * sample count)'
            scope = 'specified process only; no child-process aggregation or ETW; sampler cost excluded from target CPU'
        }
        summary = [ordered]@{
            cpu_percent_machine = $cpuPercent
            cpu_percent_machine_interval = Get-Statistics @($samples | Where-Object { $null -ne $_.cpu_percent_machine_interval } | ForEach-Object { $_.cpu_percent_machine_interval })
            private_mib = $privateStatistics
            working_set_mib = Get-Statistics @($samples | ForEach-Object { $_.working_set_mib })
            handles = Get-Statistics @($samples | ForEach-Object { [double]$_.handles })
            os_process_count = Get-Statistics @($samples | ForEach-Object { [double]$_.os_process_count })
        }
        thresholds = [ordered]@{
            cpu_percent_machine_max = 0.1
            private_mib_max = $privateLimit
            observed_cpu_within_limit = $complete -and $cpuPercent -le 0.1
            observed_private_within_limit = $complete -and $privateStatistics.max -le $privateLimit
            qualifies_as_30_minute_run = $complete -and $measuredSeconds -ge 1800
        }
        samples = $samples.ToArray()
    }
    [IO.Directory]::CreateDirectory($outputPath) | Out-Null
    $reportPath = Join-Path $outputPath ("overhead-{0}-{1}.json" -f $ProcessId, [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ'))
    [IO.File]::WriteAllText($reportPath, ($report | ConvertTo-Json -Depth 8) + "`n", [Text.UTF8Encoding]::new($false))
    Write-Output $reportPath
    if (-not $complete) { throw "Mesure incomplète. $failure Rapport conservé dans $reportPath" }
}
finally { $measuredProcess.Dispose() }
