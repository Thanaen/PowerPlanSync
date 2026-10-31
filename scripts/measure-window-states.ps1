#requires -Version 7.5

<#
.SYNOPSIS
Mesure le coût de PowerPlanSync en démonstration isolée, fenêtre ouverte puis fenêtre fermée.
La fenêtre est fermée par UI Automation (WindowPattern.Close), comme le bouton de fermeture :
l'application reste dans la zone de notification. Aucun plan Windows n'est modifié en démonstration.
Avec `--gpui`, la fenêtre vit dans un processus `--window` enfant (D18) : fenêtre ouverte, les
deux processus sont mesurés en même temps et leur somme est relevée ; fenêtre fermée, le script
vérifie qu'aucun processus de fenêtre ne reste.
#>
[CmdletBinding()]
param(
    [string]$ExecutablePath = (Join-Path $PSScriptRoot '../target/release/power-plan-sync.exe'),
    [string[]]$Arguments = @('--gpui'),
    [Parameter(Mandatory)][string]$OutputDirectory,
    [ValidateRange(1, 3600)][int]$DurationSeconds = 60,
    [ValidateRange(0, 600)][int]$StabilizationSeconds = 10
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$automation = [System.Windows.Automation.AutomationElement]

$executable = [IO.Path]::GetFullPath($ExecutablePath)
$output = [IO.Path]::GetFullPath($OutputDirectory)
$data = Join-Path $output 'data'
[IO.Directory]::CreateDirectory($output) | Out-Null
$planBefore = (& powercfg.exe /getactivescheme) -join "`n"
if ($LASTEXITCODE -ne 0) { throw 'Plan initial illisible.' }

function Get-WindowProcessIds([int]$TrackerId) {
    @($TrackerId) + @(Get-CimInstance Win32_Process -Filter "ParentProcessId = $TrackerId AND Name = 'power-plan-sync.exe'" |
        ForEach-Object { [int]$_.ProcessId })
}

function Find-Window([int]$TrackerId) {
    foreach ($id in Get-WindowProcessIds $TrackerId) {
        $condition = [System.Windows.Automation.PropertyCondition]::new($automation::ProcessIdProperty, $id)
        $found = $automation::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $condition)
        if ($found) { return $found }
    }
    $null
}

$argumentList = @($Arguments) + @('--demo', '--data-dir', ('"' + $data + '"'))
$process = Start-Process -FilePath $executable -ArgumentList $argumentList -PassThru
$started = $process.StartTime.ToUniversalTime()
try {
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    do {
        Start-Sleep -Milliseconds 250
        $window = Find-Window $process.Id
    } while (-not $window -and [DateTime]::UtcNow -lt $deadline)
    if (-not $window) { throw 'Fenêtre PowerPlanSync introuvable après 30 secondes.' }
    $windowProcessId = $window.Current.ProcessId

    Start-Sleep -Seconds $StabilizationSeconds
    $measure = Join-Path $PSScriptRoot 'measure-prototype.ps1'
    $windowJob = $null
    if ($windowProcessId -ne $process.Id) {
        $windowJob = Start-ThreadJob -ScriptBlock {
            & $using:measure -ProcessId $using:windowProcessId -DurationSeconds $using:DurationSeconds `
                -OutputDirectory (Join-Path $using:output 'window-open-window-process')
        }
    }
    & $measure -ProcessId $process.Id -DurationSeconds $DurationSeconds `
        -OutputDirectory (Join-Path $output 'window-open') | Out-Null
    if ($windowJob) { $windowJob | Wait-Job | Receive-Job -ErrorAction Stop | Out-Null }

    $window.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern).Close()
    Start-Sleep -Seconds 2
    $process.Refresh()
    if ($process.HasExited) { throw "L'application a quitté à la fermeture de la fenêtre." }
    if (Find-Window $process.Id) { throw 'La fenêtre est encore présente après sa fermeture.' }
    $remaining = @(Get-WindowProcessIds $process.Id).Count - 1
    if ($remaining -ne 0) { throw "$remaining processus de fenêtre encore présents après la fermeture." }
    Start-Sleep -Seconds $StabilizationSeconds
    & (Join-Path $PSScriptRoot 'measure-prototype.ps1') -ProcessId $process.Id -DurationSeconds $DurationSeconds `
        -OutputDirectory (Join-Path $output 'window-closed') | Out-Null
} finally {
    $process.Refresh()
    if (-not $process.HasExited -and $process.StartTime.ToUniversalTime() -eq $started) {
        $process.Kill()
        [void]$process.WaitForExit(5000)
    }
    $planAfter = (& powercfg.exe /getactivescheme) -join "`n"
    if ($planAfter -ne $planBefore) { Write-Warning "Plan Windows différent après la mesure : $planAfter" }
}

$open = Get-Content (Join-Path $output 'window-open/summary.json') -Raw | ConvertFrom-Json
$openWindowProcess = $null
$openSum = $open.private_mib_max
$windowProcessSummary = Join-Path $output 'window-open-window-process/summary.json'
if (Test-Path -LiteralPath $windowProcessSummary) {
    $openWindowProcess = Get-Content $windowProcessSummary -Raw | ConvertFrom-Json
    $openSum = [Math]::Round($open.private_mib_max + $openWindowProcess.private_mib_max, 3)
}
$summary = [ordered]@{
    executable = $executable
    sha256 = (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash
    arguments = $argumentList
    stabilization_seconds = $StabilizationSeconds
    plan_unchanged = ($planAfter -eq $planBefore)
    window_open = $open
    window_open_window_process = $openWindowProcess
    # Somme des maxima des deux processus : borne supérieure du maximum de la somme.
    window_open_private_mib_sum = $openSum
    window_closed = Get-Content (Join-Path $output 'window-closed/summary.json') -Raw | ConvertFrom-Json
}
$json = $summary | ConvertTo-Json -Depth 4
[IO.File]::WriteAllText((Join-Path $output 'result.json'), $json + "`n", [Text.UTF8Encoding]::new($false))
Remove-Item -LiteralPath $data -Recurse -Force -ErrorAction SilentlyContinue
$json
