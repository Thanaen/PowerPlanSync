#requires -Version 7.5

<#
.SYNOPSIS
Mesure le coût de PowerPlanSync en démonstration isolée, fenêtre ouverte puis fenêtre fermée.
La fenêtre est fermée par UI Automation (WindowPattern.Close), comme le bouton de fermeture :
l'application reste dans la zone de notification. Aucun plan Windows n'est modifié en démonstration.
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

$executable = [IO.Path]::GetFullPath($ExecutablePath)
$output = [IO.Path]::GetFullPath($OutputDirectory)
$data = Join-Path $output 'data'
[IO.Directory]::CreateDirectory($output) | Out-Null
$planBefore = (& powercfg.exe /getactivescheme) -join "`n"
if ($LASTEXITCODE -ne 0) { throw 'Plan initial illisible.' }

$argumentList = @($Arguments) + @('--demo', '--data-dir', ('"' + $data + '"'))
$process = Start-Process -FilePath $executable -ArgumentList $argumentList -PassThru
$started = $process.StartTime.ToUniversalTime()
try {
    $automation = [System.Windows.Automation.AutomationElement]
    $condition = [System.Windows.Automation.PropertyCondition]::new($automation::ProcessIdProperty, $process.Id)
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    do {
        Start-Sleep -Milliseconds 250
        $window = $automation::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $condition)
    } while (-not $window -and [DateTime]::UtcNow -lt $deadline)
    if (-not $window) { throw 'Fenêtre PowerPlanSync introuvable après 30 secondes.' }

    Start-Sleep -Seconds $StabilizationSeconds
    & (Join-Path $PSScriptRoot 'measure-prototype.ps1') -ProcessId $process.Id -DurationSeconds $DurationSeconds `
        -OutputDirectory (Join-Path $output 'window-open') | Out-Null

    $window.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern).Close()
    Start-Sleep -Seconds 2
    $process.Refresh()
    if ($process.HasExited) { throw "L'application a quitté à la fermeture de la fenêtre." }
    if ($automation::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $condition)) {
        throw 'La fenêtre est encore présente après sa fermeture.'
    }
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

$summary = [ordered]@{
    executable = $executable
    sha256 = (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash
    arguments = $argumentList
    stabilization_seconds = $StabilizationSeconds
    plan_unchanged = ($planAfter -eq $planBefore)
    window_open = Get-Content (Join-Path $output 'window-open/summary.json') -Raw | ConvertFrom-Json
    window_closed = Get-Content (Join-Path $output 'window-closed/summary.json') -Raw | ConvertFrom-Json
}
$json = $summary | ConvertTo-Json -Depth 4
[IO.File]::WriteAllText((Join-Path $output 'result.json'), $json + "`n", [Text.UTF8Encoding]::new($false))
Remove-Item -LiteralPath $data -Recurse -Force -ErrorAction SilentlyContinue
$json
