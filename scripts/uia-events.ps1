#requires -Version 7.5

<#
.SYNOPSIS
Enregistre les événements UI Automation qu'un lecteur d'écran utilise pour annoncer un
changement sans déplacer le focus : région active, notification, changement de nom et de focus.
L'écoute porte sur les fenêtres du PID donné. `-During` reçoit la fenêtre et agit pendant l'écoute.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][int]$ProcessId,
    [Parameter(Mandatory)][string]$OutputPath,
    [ValidateRange(1, 600)][int]$Seconds = 10,
    [scriptblock]$During
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes

if (-not ('PowerPlanSyncInspect.EventRecorder' -as [type])) {
    $references = @(
        [System.Windows.Automation.AutomationElement].Assembly.Location,
        [System.Windows.Automation.ControlType].Assembly.Location,
        [System.Windows.Threading.Dispatcher].Assembly.Location,
        'System.Runtime', 'System.Collections.Concurrent', 'System.ComponentModel.Primitives',
        'System.Linq.Expressions', 'netstandard'
    )
    Add-Type -ReferencedAssemblies $references -TypeDefinition @'
using System;
using System.Collections.Concurrent;
using System.Windows.Automation;

namespace PowerPlanSyncInspect {
    public sealed class EventRecorder {
        // Identifiants UIA de UIAutomationClient.h, absents de l'API managée.
        const int LiveRegionChanged = 20024;
        const int Notification = 20035;
        readonly int processId;
        readonly AutomationElement window;
        public readonly ConcurrentQueue<string> Lines = new ConcurrentQueue<string>();

        public EventRecorder(AutomationElement window, int processId) {
            this.window = window;
            this.processId = processId;
        }

        static string Describe(object sender) {
            try {
                var element = (AutomationElement)sender;
                return element.Current.ControlType.ProgrammaticName.Replace("ControlType.", "") + " \"" + element.Current.Name + "\"";
            } catch (Exception) {
                return "élément disparu";
            }
        }

        bool FromProcess(object sender) {
            try { return ((AutomationElement)sender).Current.ProcessId == processId; } catch (Exception) { return false; }
        }

        static string Details(AutomationEventArgs args) {
            var notification = args as NotificationEventArgs;
            if (notification == null) return "";
            return " kind=" + notification.NotificationKind + " processing=" + notification.NotificationProcessing
                + " text=\"" + notification.DisplayString + "\" activity=" + notification.ActivityId;
        }

        void Add(string text) {
            Lines.Enqueue(DateTime.UtcNow.ToString("o") + " " + text);
        }

        public void Start() {
            Add("écoute pid=" + processId);
            foreach (var id in new[] { LiveRegionChanged, Notification }) {
                var automationEvent = AutomationEvent.LookupById(id);
                if (automationEvent == null) { Add("événement " + id + " inconnu du client"); continue; }
                var label = id == LiveRegionChanged ? "live-region " : "notification ";
                // Une notification levée sur le fournisseur hôte d'un HWND n'est pas toujours
                // rattachée au sous-arbre de la fenêtre : l'écoute part du bureau, filtrée par PID.
                Automation.AddAutomationEventHandler(automationEvent, AutomationElement.RootElement, TreeScope.Subtree,
                    (sender, args) => { if (FromProcess(sender)) Add(label + Describe(sender) + Details(args)); });
            }
            Automation.AddAutomationPropertyChangedEventHandler(window, TreeScope.Subtree,
                (sender, args) => Add("name-changed " + Describe(sender) + " -> \"" + args.NewValue + "\""),
                AutomationElement.NameProperty);
            Automation.AddAutomationFocusChangedEventHandler((sender, args) => {
                try {
                    if (((AutomationElement)sender).Current.ProcessId == processId) Add("focus " + Describe(sender));
                } catch (Exception) { }
            });
        }

        public void Stop() {
            Automation.RemoveAllEventHandlers();
            Add("fin");
        }
    }
}
'@
}

$automation = [System.Windows.Automation.AutomationElement]
$condition = [System.Windows.Automation.PropertyCondition]::new($automation::ProcessIdProperty, $ProcessId)
$window = $automation::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $condition)
if (-not $window) { throw "Aucune fenêtre UI Automation pour le PID $ProcessId." }

$recorder = [PowerPlanSyncInspect.EventRecorder]::new($window, $ProcessId)
$recorder.Start()
try {
    $started = [DateTime]::UtcNow
    if ($During) { & $During $window }
    $remaining = $started.AddSeconds($Seconds) - [DateTime]::UtcNow
    if ($remaining -gt [TimeSpan]::Zero) { Start-Sleep -Milliseconds ([int]$remaining.TotalMilliseconds) }
} finally {
    $recorder.Stop()
}
$lines = @($recorder.Lines.ToArray())
Set-Content -LiteralPath $OutputPath -Value $lines -Encoding utf8
$lines
