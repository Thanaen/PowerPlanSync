#requires -Version 7.5

<#
.SYNOPSIS
Agit sur le premier élément UI Automation d'un processus dont le nom correspond à une
expression régulière : Invoke, Toggle, Select, Expand, Collapse ou Focus. Échoue si aucun
élément ne correspond ou si plusieurs correspondent sans `-First`.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][int]$ProcessId,
    [Parameter(Mandatory)][string]$Name,
    [ValidateSet('Invoke', 'Toggle', 'Select', 'Expand', 'Collapse', 'Focus')][string]$Action = 'Invoke',
    [string]$ControlType,
    [switch]$First
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes

$automation = [System.Windows.Automation.AutomationElement]
$condition = [System.Windows.Automation.PropertyCondition]::new($automation::ProcessIdProperty, $ProcessId)
$windows = $automation::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $condition)
$matches = foreach ($window in $windows) {
    foreach ($element in $window.FindAll([System.Windows.Automation.TreeScope]::Subtree, [System.Windows.Automation.Condition]::TrueCondition)) {
        $current = $element.Current
        if ($current.Name -notmatch $Name) { continue }
        if ($ControlType -and $current.ControlType.ProgrammaticName -ne "ControlType.$ControlType") { continue }
        $element
    }
}
$matches = @($matches)
if ($matches.Count -eq 0) { throw "Aucun élément ne correspond à « $Name »." }
if ($matches.Count -gt 1 -and -not $First) {
    $names = ($matches | ForEach-Object { $_.Current.ControlType.ProgrammaticName + ' "' + $_.Current.Name + '"' }) -join '; '
    throw "Plusieurs éléments correspondent à « $Name » : $names"
}
$element = $matches[0]
switch ($Action) {
    'Invoke' { $element.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() }
    'Toggle' { $element.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle() }
    'Select' { $element.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern).Select() }
    'Expand' { $element.GetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern).Expand() }
    'Collapse' { $element.GetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern).Collapse() }
    'Focus' { $element.SetFocus() }
}
'{0} {1} "{2}"' -f $Action, $element.Current.ControlType.ProgrammaticName.Replace('ControlType.', ''), $element.Current.Name
