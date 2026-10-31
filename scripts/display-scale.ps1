#requires -Version 7.5

<#
.SYNOPSIS
Lit ou change l'échelle de l'écran principal, comme le réglage « Échelle » des Paramètres.
Utilise DisplayConfigGetDeviceInfo et DisplayConfigSetDeviceInfo avec les types -3 et -4,
non documentés mais employés par les outils d'essai de Windows. L'appelant note l'échelle lue
avant l'essai et la rétablit dans un bloc `finally`.

.EXAMPLE
$before = & scripts/display-scale.ps1
try { & scripts/display-scale.ps1 -Percent 200 } finally { & scripts/display-scale.ps1 -Percent $before.percent }
#>
[CmdletBinding()]
param([ValidateSet(100, 125, 150, 175, 200, 225, 250, 300, 350)][int]$Percent)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not ('PowerPlanSyncInspect.DisplayScale' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

namespace PowerPlanSyncInspect {
    public static class DisplayScale {
        static readonly int[] Steps = { 100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500 };

        [StructLayout(LayoutKind.Sequential)] struct LUID { public uint Low; public int High; }
        [StructLayout(LayoutKind.Sequential)] struct PathSource { public LUID Adapter; public uint Id; public uint ModeIndex; public uint Flags; }
        [StructLayout(LayoutKind.Sequential)] struct PathTarget {
            public LUID Adapter; public uint Id; public uint ModeIndex; public uint Technology; public uint Rotation;
            public uint Scaling; public uint RefreshNumerator; public uint RefreshDenominator; public uint ScanLine; public int Available; public uint Flags;
        }
        [StructLayout(LayoutKind.Sequential)] struct PathInfo { public PathSource Source; public PathTarget Target; public uint Flags; }
        [StructLayout(LayoutKind.Sequential, Size = 64)] struct ModeInfo { public uint InfoType; }
        [StructLayout(LayoutKind.Sequential)] struct Header { public int Type; public uint Size; public LUID Adapter; public uint Id; }
        [StructLayout(LayoutKind.Sequential)] struct GetScale { public Header Header; public int Minimum; public int Current; public int Maximum; }
        [StructLayout(LayoutKind.Sequential)] struct SetScale { public Header Header; public int Relative; }

        [DllImport("user32.dll")] static extern int GetDisplayConfigBufferSizes(uint flags, out uint paths, out uint modes);
        [DllImport("user32.dll")] static extern int QueryDisplayConfig(uint flags, ref uint pathCount, [Out] PathInfo[] paths, ref uint modeCount, [Out] ModeInfo[] modes, IntPtr topology);
        [DllImport("user32.dll")] static extern int DisplayConfigGetDeviceInfo(ref GetScale request);
        [DllImport("user32.dll")] static extern int DisplayConfigSetDeviceInfo(ref SetScale request);

        static PathSource Primary() {
            const uint QDC_ONLY_ACTIVE_PATHS = 2;
            uint pathCount, modeCount;
            Check(GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, out pathCount, out modeCount));
            var paths = new PathInfo[pathCount];
            var modes = new ModeInfo[modeCount];
            Check(QueryDisplayConfig(QDC_ONLY_ACTIVE_PATHS, ref pathCount, paths, ref modeCount, modes, IntPtr.Zero));
            if (pathCount != 1) throw new InvalidOperationException("Essai prévu pour un seul écran actif, " + pathCount + " trouvés.");
            return paths[0].Source;
        }

        static void Check(int status) { if (status != 0) throw new System.ComponentModel.Win32Exception(status); }

        static GetScale Read(PathSource source) {
            var request = new GetScale();
            request.Header.Type = -3;
            request.Header.Size = (uint)Marshal.SizeOf(typeof(GetScale));
            request.Header.Adapter = source.Adapter;
            request.Header.Id = source.Id;
            Check(DisplayConfigGetDeviceInfo(ref request));
            return request;
        }

        // Les valeurs sont relatives à l'échelle recommandée, dont l'indice est -Minimum.
        public static int Current() {
            var scale = Read(Primary());
            int recommended = -scale.Minimum;
            return Steps[Math.Max(0, Math.Min(Steps.Length - 1, recommended + scale.Current))];
        }

        public static void Set(int percent) {
            var source = Primary();
            var scale = Read(source);
            int index = Array.IndexOf(Steps, percent);
            if (index < 0) throw new ArgumentOutOfRangeException("percent");
            int relative = index - (-scale.Minimum);
            if (relative < scale.Minimum || relative > scale.Maximum) throw new ArgumentOutOfRangeException("percent", percent + " % n'est pas proposé pour cet écran.");
            var request = new SetScale();
            request.Header.Type = -4;
            request.Header.Size = (uint)Marshal.SizeOf(typeof(SetScale));
            request.Header.Adapter = source.Adapter;
            request.Header.Id = source.Id;
            request.Relative = relative;
            Check(DisplayConfigSetDeviceInfo(ref request));
        }
    }
}
'@
}

if ($PSBoundParameters.ContainsKey('Percent')) {
    [PowerPlanSyncInspect.DisplayScale]::Set($Percent)
}
[pscustomobject]@{ percent = [PowerPlanSyncInspect.DisplayScale]::Current() }
