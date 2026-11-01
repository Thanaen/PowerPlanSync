#requires -Version 7.5

<#
.SYNOPSIS
Agit sur l'icône de notification d'un processus PowerPlanSync sans souris : `Click` poste le
clic gauche que `tray-icon` reçoit de l'Explorateur ; `Menu` ouvre le vrai menu, y lit
l'identifiant de l'élément dont le texte correspond à `-Item`, ferme le menu, puis envoie la
commande `WM_COMMAND` de cet élément.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][int]$ProcessId,
    [ValidateSet('Click', 'Menu')][string]$Action = 'Menu',
    [string]$Item
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ($Action -eq 'Menu' -and -not $Item) { throw '-Item est requis avec -Action Menu.' }

if (-not ('PowerPlanSyncTray.Native' -as [type])) {
    Add-Type -Namespace PowerPlanSyncTray -Name Native -MemberDefinition @'
public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);
[DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);
[DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassNameW(IntPtr hwnd, System.Text.StringBuilder name, int size);
[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
[DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
[DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
[DllImport("user32.dll")] public static extern IntPtr SendMessageW(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
[DllImport("user32.dll")] public static extern int GetMenuItemCount(IntPtr menu);
[DllImport("user32.dll")] public static extern uint GetMenuItemID(IntPtr menu, int position);
[DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetMenuStringW(IntPtr menu, uint item, System.Text.StringBuilder text, int size, uint flags);
'@
}
$native = [PowerPlanSyncTray.Native]
# Message de rappel de l'icône choisi par `tray-icon` (`WM_USER_TRAYICON`).
$trayCallback = 6002
$WM_LBUTTONUP = 0x0202
$WM_RBUTTONUP = 0x0205
$WM_COMMAND = 0x0111
$WM_KEYDOWN = 0x0100
$VK_ESCAPE = 0x1B
$MN_GETHMENU = 0x01E1
$MF_BYPOSITION = 0x400

function Get-OwnedWindows([string]$ClassName) {
    $found = [Collections.Generic.List[IntPtr]]::new()
    $callback = [PowerPlanSyncTray.Native+EnumWindowsProc] {
        param($hwnd, $lParam)
        $class = [Text.StringBuilder]::new(256)
        [void]$native::GetClassNameW($hwnd, $class, $class.Capacity)
        $owner = [uint32]0
        [void]$native::GetWindowThreadProcessId($hwnd, [ref]$owner)
        if ($owner -eq $ProcessId -and $class.ToString() -eq $ClassName) { $found.Add($hwnd) }
        $true
    }
    [void]$native::EnumWindows($callback, [IntPtr]::Zero)
    @($found)
}

$trays = @(Get-OwnedWindows 'tray_icon_app')
if ($trays.Count -ne 1) { throw "Fenêtres tray_icon_app du PID ${ProcessId} : $($trays.Count), une attendue." }
$tray = $trays[0]

if ($Action -eq 'Click') {
    [void]$native::PostMessageW($tray, $trayCallback, [IntPtr]::Zero, [IntPtr]$WM_LBUTTONUP)
    'Click tray_icon_app'
    return
}

[void]$native::PostMessageW($tray, $trayCallback, [IntPtr]::Zero, [IntPtr]$WM_RBUTTONUP)
$deadline = [DateTime]::UtcNow.AddSeconds(5)
$popup = [IntPtr]::Zero
do {
    Start-Sleep -Milliseconds 100
    $popup = @(Get-OwnedWindows '#32768' | Where-Object { $native::IsWindowVisible($_) }) | Select-Object -First 1
} while (-not $popup -and [DateTime]::UtcNow -lt $deadline)
if (-not $popup) { throw "Le menu de l'icône ne s'est pas ouvert." }

$menu = $native::SendMessageW($popup, $MN_GETHMENU, [IntPtr]::Zero, [IntPtr]::Zero)
$items = for ($position = 0; $position -lt $native::GetMenuItemCount($menu); $position++) {
    $text = [Text.StringBuilder]::new(256)
    [void]$native::GetMenuStringW($menu, [uint32]$position, $text, $text.Capacity, $MF_BYPOSITION)
    [pscustomobject]@{ Id = $native::GetMenuItemID($menu, $position); Text = $text.ToString() }
}
[void]$native::PostMessageW($popup, $WM_KEYDOWN, [IntPtr]$VK_ESCAPE, [IntPtr]::Zero)
$deadline = [DateTime]::UtcNow.AddSeconds(5)
while (@(Get-OwnedWindows '#32768' | Where-Object { $native::IsWindowVisible($_) }).Count -and [DateTime]::UtcNow -lt $deadline) {
    Start-Sleep -Milliseconds 100
}
$chosen = @($items | Where-Object { $_.Text -match $Item })
if ($chosen.Count -ne 1) {
    throw "Éléments du menu correspondant à « $Item » : $($chosen.Count). Menu : $(($items | ForEach-Object { "$($_.Id) « $($_.Text) »" }) -join ' ; ')"
}
[void]$native::PostMessageW($tray, $WM_COMMAND, [IntPtr][int64]$chosen[0].Id, [IntPtr]::Zero)
'Menu {0} « {1} »' -f $chosen[0].Id, $chosen[0].Text
