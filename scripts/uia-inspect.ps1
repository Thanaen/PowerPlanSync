#requires -Version 7.5

<#
.SYNOPSIS
Lit l'arbre UI Automation des fenêtres d'un processus, identifié par son PID, et capture
chaque fenêtre. Lecture seule : le script n'active aucun contrôle.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][int]$ProcessId,
    [Parameter(Mandatory)][string]$OutputPrefix,
    [ValidateRange(1, 40)][int]$MaxDepth = 25,
    [switch]$NoScreenshot
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Drawing

Add-Type -Namespace PowerPlanSyncInspect -Name Native -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
[DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr value);
'@
[void][PowerPlanSyncInspect.Native]::SetProcessDpiAwarenessContext([IntPtr]::new(-4))

$automation = [System.Windows.Automation.AutomationElement]
$walker = [System.Windows.Automation.TreeWalker]::RawViewWalker

function Get-Pattern($element, $pattern) {
    $value = $null
    if ($element.TryGetCurrentPattern($pattern, [ref]$value)) { $value } else { $null }
}

function Format-Element($element) {
    $current = $element.Current
    $parts = [Collections.Generic.List[string]]::new()
    $parts.Add($current.ControlType.ProgrammaticName.Replace('ControlType.', ''))
    $parts.Add('"' + $current.Name + '"')
    if ($current.AutomationId) { $parts.Add('id=' + $current.AutomationId) }
    if ($current.HelpText) { $parts.Add('help="' + $current.HelpText + '"') }
    if ($current.ItemStatus) { $parts.Add('status="' + $current.ItemStatus + '"') }
    if ($current.IsKeyboardFocusable) { $parts.Add('focusable') }
    if ($current.HasKeyboardFocus) { $parts.Add('FOCUS') }
    if (-not $current.IsEnabled) { $parts.Add('disabled') }
    if ($current.IsOffscreen) { $parts.Add('offscreen') }
    $toggle = Get-Pattern $element ([System.Windows.Automation.TogglePattern]::Pattern)
    if ($toggle) { $parts.Add('toggle=' + $toggle.Current.ToggleState) }
    $selection = Get-Pattern $element ([System.Windows.Automation.SelectionItemPattern]::Pattern)
    if ($selection) { $parts.Add('selected=' + $selection.Current.IsSelected) }
    $valuePattern = Get-Pattern $element ([System.Windows.Automation.ValuePattern]::Pattern)
    if ($valuePattern) { $parts.Add('value="' + $valuePattern.Current.Value + '"') }
    $expand = Get-Pattern $element ([System.Windows.Automation.ExpandCollapsePattern]::Pattern)
    if ($expand) { $parts.Add('expand=' + $expand.Current.ExpandCollapseState) }
    $position = $element.GetCurrentPropertyValue([System.Windows.Automation.AutomationElement]::PositionInSetProperty, $true)
    $size = $element.GetCurrentPropertyValue([System.Windows.Automation.AutomationElement]::SizeOfSetProperty, $true)
    if ($position -is [int] -and $position -gt 0) { $parts.Add("pos=$position/$size") }
    $rect = $current.BoundingRectangle
    if (-not $rect.IsEmpty) { $parts.Add(('[{0:0},{1:0} {2:0}x{3:0}]' -f $rect.X, $rect.Y, $rect.Width, $rect.Height)) }
    $parts -join ' '
}

function Write-Tree($element, [int]$depth, [Text.StringBuilder]$builder) {
    [void]$builder.AppendLine(('  ' * $depth) + (Format-Element $element))
    if ($depth -ge $MaxDepth) { return }
    $child = $walker.GetFirstChild($element)
    while ($child) {
        Write-Tree $child ($depth + 1) $builder
        $child = $walker.GetNextSibling($child)
    }
}

function Save-WindowImage([IntPtr]$hwnd, $rect, [string]$path) {
    $width = [int][math]::Ceiling($rect.Width)
    $height = [int][math]::Ceiling($rect.Height)
    if ($width -le 0 -or $height -le 0) { return $false }
    $bitmap = [Drawing.Bitmap]::new($width, $height)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $hdc = $graphics.GetHdc()
        $printed = [PowerPlanSyncInspect.Native]::PrintWindow($hwnd, $hdc, 2)
        $graphics.ReleaseHdc($hdc)
        if (-not $printed) {
            $graphics.CopyFromScreen([int]$rect.X, [int]$rect.Y, 0, 0, $bitmap.Size)
        }
        $bitmap.Save($path, [Drawing.Imaging.ImageFormat]::Png)
        $true
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

$condition = [System.Windows.Automation.PropertyCondition]::new($automation::ProcessIdProperty, $ProcessId)
$windows = $automation::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $condition)
if ($windows.Count -eq 0) { throw "Aucune fenêtre UI Automation pour le PID $ProcessId." }

$focused = $automation::FocusedElement
$builder = [Text.StringBuilder]::new()
[void]$builder.AppendLine("pid=$ProcessId captured_utc=$([DateTime]::UtcNow.ToString('o'))")
if ($focused -and $focused.Current.ProcessId -eq $ProcessId) {
    [void]$builder.AppendLine('focus: ' + (Format-Element $focused))
} else {
    [void]$builder.AppendLine('focus: hors du processus')
}
$index = 0
foreach ($window in $windows) {
    [void]$builder.AppendLine("--- window $index")
    Write-Tree $window 0 $builder
    if (-not $NoScreenshot) {
        $image = "$OutputPrefix-$index.png"
        $saved = Save-WindowImage ([IntPtr]$window.Current.NativeWindowHandle) $window.Current.BoundingRectangle $image
        if ($saved) { [void]$builder.AppendLine("capture: $(Split-Path -Leaf $image)") }
    }
    $index++
}
$tree = $builder.ToString()
Set-Content -LiteralPath "$OutputPrefix-tree.txt" -Value $tree -Encoding utf8
$tree
