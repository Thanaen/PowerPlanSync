#requires -Version 7.5

<#
.SYNOPSIS
Lit ou change, pour la session courante, le thème des applications (clair ou sombre) et le
contraste élevé, puis prévient les fenêtres ouvertes. Sert aux essais d'interface : l'appelant
note l'état lu avant l'essai et le rétablit dans un bloc `finally`.

.EXAMPLE
$before = & scripts/windows-appearance.ps1
try { & scripts/windows-appearance.ps1 -AppsLight $true } finally { & scripts/windows-appearance.ps1 -AppsLight $before.apps_light -HighContrast $before.high_contrast }
#>
[CmdletBinding()]
param(
    [Nullable[bool]]$AppsLight,
    [Nullable[bool]]$HighContrast
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not ('PowerPlanSyncInspect.Appearance' -as [type])) {
    Add-Type -Namespace PowerPlanSyncInspect -Name Appearance -MemberDefinition @'
[StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
public struct HIGHCONTRAST { public uint cbSize; public uint dwFlags; public IntPtr lpszDefaultScheme; }
[DllImport("user32.dll", SetLastError = true)]
static extern bool SystemParametersInfo(uint action, uint param, ref HIGHCONTRAST value, uint winIni);
[DllImport("user32.dll", CharSet = CharSet.Unicode)]
static extern IntPtr SendMessageTimeout(IntPtr hwnd, uint msg, IntPtr wParam, string lParam, uint flags, uint timeout, out IntPtr result);
const uint HCF_HIGHCONTRASTON = 1;
public static bool HighContrastOn() {
    var value = new HIGHCONTRAST { cbSize = (uint)Marshal.SizeOf(typeof(HIGHCONTRAST)) };
    if (!SystemParametersInfo(0x0042, value.cbSize, ref value, 0)) throw new System.ComponentModel.Win32Exception();
    return (value.dwFlags & HCF_HIGHCONTRASTON) != 0;
}
public static void SetHighContrast(bool on) {
    var value = new HIGHCONTRAST { cbSize = (uint)Marshal.SizeOf(typeof(HIGHCONTRAST)) };
    if (!SystemParametersInfo(0x0042, value.cbSize, ref value, 0)) throw new System.ComponentModel.Win32Exception();
    value.dwFlags = on ? (value.dwFlags | HCF_HIGHCONTRASTON) : (value.dwFlags & ~HCF_HIGHCONTRASTON);
    // SPI_SETHIGHCONTRAST, SPIF_UPDATEINIFILE | SPIF_SENDCHANGE
    if (!SystemParametersInfo(0x0043, value.cbSize, ref value, 3)) throw new System.ComponentModel.Win32Exception();
}
public static void BroadcastThemeChange() {
    IntPtr result;
    SendMessageTimeout((IntPtr)0xffff, 0x001A, IntPtr.Zero, "ImmersiveColorSet", 2, 5000, out result);
}
'@
}

$personalize = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'
if ($null -ne $AppsLight) {
    Set-ItemProperty -Path $personalize -Name AppsUseLightTheme -Value ([int]$AppsLight) -Type DWord
    [PowerPlanSyncInspect.Appearance]::BroadcastThemeChange()
}
if ($null -ne $HighContrast -and $HighContrast -ne [PowerPlanSyncInspect.Appearance]::HighContrastOn()) {
    [PowerPlanSyncInspect.Appearance]::SetHighContrast($HighContrast)
}
[pscustomobject]@{
    apps_light = [bool](Get-ItemPropertyValue -Path $personalize -Name AppsUseLightTheme)
    high_contrast = [PowerPlanSyncInspect.Appearance]::HighContrastOn()
}
