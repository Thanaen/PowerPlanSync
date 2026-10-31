#requires -Version 7.5

<#
.SYNOPSIS
Décrit l'élément qui a le focus clavier selon UI Automation, sur une ligne : type, nom,
description et états utiles. Sert à suivre l'ordre de tabulation entre deux touches.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$element = [System.Windows.Automation.AutomationElement]::FocusedElement
if (-not $element) { return 'focus: aucun' }
$current = $element.Current
$parts = @($current.ControlType.ProgrammaticName.Replace('ControlType.', ''), ('"' + $current.Name + '"'))
if ($current.HelpText) { $parts += 'help="' + $current.HelpText + '"' }
if (-not $current.IsEnabled) { $parts += 'disabled' }
$parts += "pid=$($current.ProcessId)"
$rect = $current.BoundingRectangle
if (-not $rect.IsEmpty) { $parts += ('[{0:0},{1:0} {2:0}x{3:0}]' -f $rect.X, $rect.Y, $rect.Width, $rect.Height) }
$parts -join ' '
