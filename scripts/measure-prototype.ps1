[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateRange(1, [int]::MaxValue)][int]$ProcessId,
    [ValidateRange(1, 3600)][int]$DurationSeconds = 60,
    [ValidateRange(100, 60000)][int]$SampleIntervalMilliseconds = 1000,
    [Parameter(Mandatory)][string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'
$outputPath = [IO.Path]::GetFullPath($OutputDirectory)
[IO.Directory]::CreateDirectory($outputPath) | Out-Null
$measuredProcess = Get-Process -Id $ProcessId -ErrorAction Stop
$processStart = $measuredProcess.StartTime.ToUniversalTime()
$logicalProcessors = [Environment]::ProcessorCount
$initialCpu = $measuredProcess.TotalProcessorTime.TotalSeconds
$timer = [Diagnostics.Stopwatch]::StartNew()
$samples = [Collections.Generic.List[object]]::new()

do {
    $measuredProcess.Refresh()
    if ($measuredProcess.HasExited) {
        throw "Le processus $ProcessId s'est arrêté pendant la mesure."
    }
    $samples.Add([pscustomobject]@{
        elapsed_seconds = [Math]::Round($timer.Elapsed.TotalSeconds, 3)
        cpu_seconds = $measuredProcess.TotalProcessorTime.TotalSeconds - $initialCpu
        private_bytes = $measuredProcess.PrivateMemorySize64
        working_set_bytes = $measuredProcess.WorkingSet64
        handles = $measuredProcess.HandleCount
        threads = $measuredProcess.Threads.Count
    })
    if ($timer.Elapsed.TotalSeconds -ge $DurationSeconds) { break }
    Start-Sleep -Milliseconds $SampleIntervalMilliseconds
} while ($true)

$lastSample = $samples[$samples.Count - 1]
$summary = [ordered]@{
    process_id = $ProcessId
    process_name = $measuredProcess.ProcessName
    process_started_utc = $processStart.ToString('o')
    measured_utc = [DateTime]::UtcNow.ToString('o')
    duration_seconds = $lastSample.elapsed_seconds
    logical_processors = $logicalProcessors
    samples = $samples.Count
    cpu_percent_machine = [Math]::Round(100 * $lastSample.cpu_seconds / $lastSample.elapsed_seconds / $logicalProcessors, 5)
    private_mib_max = [Math]::Round(($samples | Measure-Object private_bytes -Maximum).Maximum / 1MB, 3)
    working_set_mib_max = [Math]::Round(($samples | Measure-Object working_set_bytes -Maximum).Maximum / 1MB, 3)
    handles_first = $samples[0].handles
    handles_last = $lastSample.handles
    threads_last = $lastSample.threads
}
$samples | Export-Csv -LiteralPath (Join-Path $outputPath 'samples.csv') -NoTypeInformation -Encoding UTF8
$json = $summary | ConvertTo-Json
[IO.File]::WriteAllText((Join-Path $outputPath 'summary.json'), $json + "`n", [Text.UTF8Encoding]::new($false))
$json
