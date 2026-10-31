#requires -Version 7.5

<#
.SYNOPSIS
Envoie une combinaison de touches virtuelles par SendInput, dans l'ordre donné, puis relâche
les touches dans l'ordre inverse. Sert aux commandes du Narrateur, par exemple
Insert+Alt+X pour son récapitulatif vocal : `-Keys 0x2D,0x12,0x58`.
#>
[CmdletBinding()]
param([Parameter(Mandatory)][int[]]$Keys)

$ErrorActionPreference = 'Stop'
if (-not ('PowerPlanSyncInspect.Keys' -as [type])) {
    Add-Type -Namespace PowerPlanSyncInspect -Name Keys -MemberDefinition @'
[StructLayout(LayoutKind.Sequential)]
public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
[StructLayout(LayoutKind.Explicit, Size = 40)]
public struct INPUT { [FieldOffset(0)] public uint type; [FieldOffset(8)] public KEYBDINPUT ki; }
[DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint count, INPUT[] inputs, int size);
[DllImport("user32.dll")] static extern uint MapVirtualKey(uint code, uint mapType);
static INPUT Key(ushort vk, bool up) {
    var input = new INPUT { type = 1 };
    input.ki.wVk = vk;
    input.ki.wScan = (ushort)MapVirtualKey(vk, 0);
    // Insert et les flèches sont des touches étendues : sans ce drapeau, le Narrateur ne les reconnaît pas.
    bool extended = vk == 0x2D || vk == 0x2E || (vk >= 0x21 && vk <= 0x28);
    input.ki.dwFlags = (up ? 2u : 0u) | (extended ? 1u : 0u);
    return input;
}
public static uint Chord(int[] keys) {
    var inputs = new INPUT[keys.Length * 2];
    for (int i = 0; i < keys.Length; i++) {
        inputs[i] = Key((ushort)keys[i], false);
        inputs[keys.Length * 2 - 1 - i] = Key((ushort)keys[i], true);
    }
    return SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT)));
}
'@
}
$sent = [PowerPlanSyncInspect.Keys]::Chord($Keys)
if ($sent -ne $Keys.Count * 2) { throw "SendInput a envoyé $sent événements sur $($Keys.Count * 2)." }
