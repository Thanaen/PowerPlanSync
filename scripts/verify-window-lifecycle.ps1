#requires -Version 7.5

[CmdletBinding()]
param(
    [string]$ExecutablePath = (Join-Path $PSScriptRoot '../target/release/examples/window_lifecycle.exe'),
    [string]$EvidenceDirectory = (Join-Path $PSScriptRoot '../.audit/window-lifecycle'),
    [ValidateRange(1, 600)][int]$MarkerTimeoutSeconds = 60
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$executable = [IO.Path]::GetFullPath($ExecutablePath)
$runId = '{0}-{1}' -f (Get-Date -Format 'yyyyMMdd-HHmmss'), ([guid]::NewGuid().ToString('N').Substring(0, 8))
$evidence = Join-Path ([IO.Path]::GetFullPath($EvidenceDirectory)) $runId
$reportPath = Join-Path $evidence 'result.json'
$stdoutPath = Join-Path $evidence 'stdout.log'
$stderrPath = Join-Path $evidence 'stderr.log'
$expectedCycles = 100
$privateLimitBytes = 5MB
$handleLimit = 10

function Get-ProcessIdentity([Diagnostics.Process]$Process) {
    $Process.Refresh()
    [ordered]@{
        id = $Process.Id
        executable_path = [IO.Path]::GetFullPath($Process.MainModule.FileName)
        started_utc = $Process.StartTime.ToUniversalTime().ToString('o')
    }
}

function Assert-ProcessIdentity(
    [Diagnostics.Process]$Process,
    [string]$ExpectedPath,
    [datetime]$ExpectedStart
) {
    $Process.Refresh()
    if ($Process.HasExited) { throw 'La sonde a quitté avant la mesure demandée.' }
    $actualPath = [IO.Path]::GetFullPath($Process.MainModule.FileName)
    if (-not [string]::Equals($actualPath, $ExpectedPath, [StringComparison]::OrdinalIgnoreCase) -or
        $Process.StartTime.ToUniversalTime() -ne $ExpectedStart) {
        throw "L'identité du PID $($Process.Id) a changé."
    }
}

function Get-Sample(
    [Diagnostics.Process]$Process,
    [string]$ExpectedPath,
    [datetime]$ExpectedStart,
    [string]$Phase
) {
    Assert-ProcessIdentity $Process $ExpectedPath $ExpectedStart
    [ordered]@{
        phase = $Phase
        captured_utc = [DateTime]::UtcNow.ToString('o')
        private_bytes = $Process.PrivateMemorySize64
        private_mib = [math]::Round($Process.PrivateMemorySize64 / 1MB, 3)
        handles = $Process.HandleCount
    }
}

function Stop-OwnedProcess(
    [Diagnostics.Process]$Process,
    [string]$ExpectedPath,
    [datetime]$ExpectedStart
) {
    if ($null -eq $Process -or $Process.HasExited) { return }
    Assert-ProcessIdentity $Process $ExpectedPath $ExpectedStart
    $Process.Kill()
    if (-not $Process.WaitForExit(5000)) { throw "Le PID $($Process.Id) ne s'est pas arrêté." }
}

if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw "Exécutable release manquant : $executable"
}
[IO.Directory]::CreateDirectory($evidence) | Out-Null
[IO.File]::WriteAllText($stdoutPath, '', [Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllText($stderrPath, '', [Text.UTF8Encoding]::new($false))

$startInfo = [Diagnostics.ProcessStartInfo]::new()
$startInfo.FileName = $executable
$startInfo.UseShellExecute = $false
$startInfo.CreateNoWindow = $true
$startInfo.WindowStyle = [Diagnostics.ProcessWindowStyle]::Hidden
$startInfo.RedirectStandardOutput = $true
$startInfo.RedirectStandardError = $true
$process = [Diagnostics.Process]::new()
$process.StartInfo = $startInfo
$process.EnableRaisingEvents = $true
$baseline = $null
$afterCycles = $null
$identity = $null
$processStart = [datetime]::MinValue
$lines = [Collections.Generic.List[string]]::new()
$phases = [Collections.Generic.List[string]]::new()
$reportedCycles = $null
$reportedOpens = $null
$reportedHides = $null
$failure = $null
$exitCode = $null
$stderr = ''
$stderrTask = $null
$startedUtc = [DateTime]::UtcNow

try {
    if (-not $process.Start()) { throw 'Le démarrage de la sonde a échoué.' }
    $processStart = $process.StartTime.ToUniversalTime()
    $identity = Get-ProcessIdentity $process
    $stderrTask = $process.StandardError.ReadToEndAsync()

    while ($true) {
        $readTask = $process.StandardOutput.ReadLineAsync()
        if (-not $readTask.Wait([TimeSpan]::FromSeconds($MarkerTimeoutSeconds))) {
            throw "Aucune sortie de phase reçue pendant $MarkerTimeoutSeconds secondes."
        }
        $line = $readTask.Result
        if ($null -eq $line) { break }
        $lines.Add($line)
        $line | Add-Content -LiteralPath $stdoutPath -Encoding utf8NoBOM
        if ($line -match 'phase=([^ ]+)') { $phases.Add($Matches[1]) }
        if ($line -match '^phase=baseline_begin\b') {
            $baseline = Get-Sample $process $executable $processStart 'baseline_begin'
        } elseif ($line -match '^phase=baseline_end\b') {
            $afterCycles = Get-Sample $process $executable $processStart 'baseline_end'
            if ($line -match '\bcycles=(\d+)') { $reportedCycles = [int]$Matches[1] }
            if ($line -match '\bsuccessful_opens=(\d+)') { $reportedOpens = [int]$Matches[1] }
            if ($line -match '\bhides=(\d+)') { $reportedHides = [int]$Matches[1] }
        }
    }
    $process.WaitForExit()
    $exitCode = $process.ExitCode
    $stderr = $stderrTask.GetAwaiter().GetResult()
    [IO.File]::WriteAllText($stderrPath, $stderr, [Text.UTF8Encoding]::new($false))
    if ($exitCode -ne 0) { throw "La sonde a quitté avec le code $exitCode. $stderr" }
    if ($null -eq $baseline -or $null -eq $afterCycles) { throw 'Les deux marqueurs de mesure sont incomplets.' }
    foreach ($required in @('baseline_begin', 'baseline_end', 'hidden_begin', 'hidden_end', 'complete')) {
        if ($required -notin $phases) { throw "Le marqueur $required est absent." }
    }
    if ($reportedCycles -ne $expectedCycles -or $reportedOpens -ne 101 -or $reportedHides -ne 101) {
        throw "Comptes inattendus : cycles=$reportedCycles, ouvertures=$reportedOpens, fermetures=$reportedHides."
    }
} catch {
    $failure = $_.Exception.Message
} finally {
    try {
        if ($processStart -ne [datetime]::MinValue) {
            Stop-OwnedProcess $process $executable $processStart
        }
    } catch {
        $failure = if ($failure) { "$failure Nettoyage : $($_.Exception.Message)" } else { $_.Exception.Message }
    }
    if ($stderr -eq '' -and $null -ne $stderrTask) {
        try {
            $stderr = $stderrTask.GetAwaiter().GetResult()
            [IO.File]::WriteAllText($stderrPath, $stderr, [Text.UTF8Encoding]::new($false))
        } catch {}
    }
    $privateDrift = if ($null -ne $baseline -and $null -ne $afterCycles) { $afterCycles.private_bytes - $baseline.private_bytes } else { $null }
    $handleDrift = if ($null -ne $baseline -and $null -ne $afterCycles) { $afterCycles.handles - $baseline.handles } else { $null }
    $passed = -not $failure -and $reportedCycles -eq $expectedCycles -and
        $privateDrift -le $privateLimitBytes -and $handleDrift -le $handleLimit
    $report = [ordered]@{
        schema_version = 1
        started_utc = $startedUtc.ToString('o')
        finished_utc = [DateTime]::UtcNow.ToString('o')
        process = $identity
        executable_path = $executable
        expected_cycles = $expectedCycles
        reported_cycles = $reportedCycles
        reported_successful_opens = $reportedOpens
        reported_hides = $reportedHides
        phases = $phases
        samples = [ordered]@{ baseline_begin = $baseline; baseline_end = $afterCycles }
        drift = [ordered]@{
            private_bytes = $privateDrift
            private_mib = if ($null -ne $privateDrift) { [math]::Round($privateDrift / 1MB, 3) } else { $null }
            handles = $handleDrift
        }
        thresholds = [ordered]@{
            private_bytes_max = $privateLimitBytes
            private_mib_max = 5
            handles_max = $handleLimit
        }
        stdout_path = $stdoutPath
        stderr_path = $stderrPath
        exit_code = $exitCode
        measurement_scope = 'PrivateMemorySize64 and HandleCount for the launched process at probe phase markers; no ETW.'
        passed = $passed
        failure = $failure
    }
    [IO.File]::WriteAllText($reportPath, ($report | ConvertTo-Json -Depth 8) + "`n", [Text.UTF8Encoding]::new($false))
    $process.Dispose()
}

if ($failure) { throw "Mesure incomplète. $failure Rapport : $reportPath" }
if (-not $passed) {
    throw "Dérive hors seuil : $([math]::Round($privateDrift / 1MB, 3)) Mio, $handleDrift handles. Rapport : $reportPath"
}
Write-Output "100 cycles vérifiés. Dérive : $([math]::Round($privateDrift / 1MB, 3)) Mio, $handleDrift handles."
Write-Output "Preuve : $reportPath"
