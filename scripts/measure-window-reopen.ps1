#requires -Version 7.5

<#
.SYNOPSIS
Ouvre puis ferme la fenêtre de PowerPlanSync plusieurs fois dans le même processus et relève,
fenêtre fermée, la mémoire privée, les handles et les threads. La fenêtre est rouverte comme
par un second lancement, qui demande à l'instance en cours de s'afficher. Le dossier de données
est neuf : l'automatisation reste désactivée et aucun plan Windows ne change.
La fenêtre vit dans un processus `--window` enfant (D18) : chaque ligne relève le processus de
suivi et compte les processus de fenêtre restants, qui doivent être zéro fenêtre fermée.
#>
[CmdletBinding()]
param(
    [string]$ExecutablePath = (Join-Path $PSScriptRoot '../target/release/power-plan-sync.exe'),
    [Parameter(Mandatory)][string]$OutputDirectory,
    [ValidateRange(2, 100)][int]$Opens = 6,
    [ValidateRange(1, 60)][int]$SettleSeconds = 8
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes

$executable = [IO.Path]::GetFullPath($ExecutablePath)
$output = [IO.Path]::GetFullPath($OutputDirectory)
$data = Join-Path $output 'data'
if (Test-Path -LiteralPath $data) { throw "Le dossier de données existe déjà : $data" }
[IO.Directory]::CreateDirectory($output) | Out-Null
$arguments = @('--gpui', '--data-dir', ('"' + $data + '"'))
$planBefore = (& powercfg.exe /getactivescheme) -join "`n"
$automation = [System.Windows.Automation.AutomationElement]

function Get-WindowProcessIds([int]$TrackerId) {
    @(Get-CimInstance Win32_Process -Filter "ParentProcessId = $TrackerId AND Name = 'power-plan-sync.exe'" |
        ForEach-Object { [int]$_.ProcessId })
}

function Find-Window([int]$TrackerId) {
    foreach ($id in @($TrackerId) + @(Get-WindowProcessIds $TrackerId)) {
        $condition = [System.Windows.Automation.PropertyCondition]::new($automation::ProcessIdProperty, $id)
        $found = $automation::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $condition)
        if ($found) { return $found }
    }
    $null
}

function Wait-Window([int]$ProcessId, [bool]$Present) {
    $deadline = [DateTime]::UtcNow.AddSeconds(15)
    do {
        $window = Find-Window $ProcessId
        if ([bool]$window -eq $Present) { return $window }
        Start-Sleep -Milliseconds 250
    } while ([DateTime]::UtcNow -lt $deadline)
    throw "Fenêtre $(if ($Present) { 'absente' } else { 'toujours présente' }) après 15 secondes."
}

$process = Start-Process -FilePath $executable -ArgumentList $arguments -PassThru
$started = $process.StartTime.ToUniversalTime()
$rows = [Collections.Generic.List[object]]::new()
try {
    $window = Wait-Window $process.Id $true
    for ($open = 1; $open -le $Opens; $open++) {
        Start-Sleep -Seconds 2
        $window.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern).Close()
        $null = Wait-Window $process.Id $false
        Start-Sleep -Seconds $SettleSeconds
        $process.Refresh()
        if ($process.HasExited) { throw "L'application a quitté après la fermeture $open." }
        $rows.Add([pscustomobject]@{
            opens = $open
            closed_private_mib = [math]::Round($process.PrivateMemorySize64 / 1MB, 1)
            handles = $process.HandleCount
            threads = $process.Threads.Count
            window_processes = @(Get-WindowProcessIds $process.Id).Count
        })
        if ($open -lt $Opens) {
            Start-Process -FilePath $executable -ArgumentList $arguments -Wait -WindowStyle Hidden
            $window = Wait-Window $process.Id $true
        }
    }
} finally {
    $process.Refresh()
    if (-not $process.HasExited -and $process.StartTime.ToUniversalTime() -eq $started) {
        $process.Kill()
        [void]$process.WaitForExit(5000)
    }
}
$planAfter = (& powercfg.exe /getactivescheme) -join "`n"
if ($planAfter -ne $planBefore) { throw "Le plan Windows a changé pendant la mesure : $planAfter" }
$rows | Export-Csv -LiteralPath (Join-Path $output 'reopen.csv') -NoTypeInformation -Encoding utf8NoBOM
[ordered]@{
    executable = $executable
    sha256 = (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash
    settle_seconds = $SettleSeconds
    per_open_mib = [math]::Round(($rows[-1].closed_private_mib - $rows[0].closed_private_mib) / ($rows.Count - 1), 1)
    rows = $rows
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $output 'result.json') -Encoding utf8NoBOM
Remove-Item -LiteralPath $data -Recurse -Force -ErrorAction SilentlyContinue
$rows | Format-Table -AutoSize | Out-String
