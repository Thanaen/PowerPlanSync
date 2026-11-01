---
name: verify
description: Vérifier PowerPlanSync sur son interface Windows native, ses cycles de plans et son coût mesuré après une modification produit, avec données isolées et preuves conservées.
---

# Vérifier PowerPlanSync

Lire d'abord `AGENTS.md`, `.specify/memory/constitution.md`, `.specify/feature.json` et [la carte des fonctionnalités](features/README.md). La conception reste dans `specs/001-app-power-profiles`. Ce skill décrit comment vérifier le produit, sans remplacer sa spécification.

## Launch

Utiliser le binaire produit `target/release/power-plan-sync.exe`. Il lance lui-même sa fenêtre dans un processus enfant `power-plan-sync.exe --window` (D18 de `research.md`). Ne jamais lancer `--window` à la main. Dans le dépôt principal de cette machine, son chemin est `D:\Dev\PowerPlanSync\target\release\power-plan-sync.exe`. Exécuter les commandes depuis la racine du checkout à vérifier. Ne pas piloter une instance utilisée par une autre campagne.

Si nécessaire, compiler une fois avant les essais avec `cargo build --locked --release --bins --examples`. Rust est installé par rustup : `cargo` est dans le PATH et `rust-toolchain.toml` sélectionne la chaîne épinglée. Ne définir ni `CARGO_HOME` ni `RUSTUP_HOME`. Ne pas reconstruire pendant une mesure de consommation.

Lancer la démonstration isolée depuis l'outil shell, sans automatiser un terminal Windows avec sky.

```powershell
$ppsRepo = (Get-Location).Path
$ppsRun = (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0,8)
$ppsExe = Join-Path $ppsRepo 'target/release/power-plan-sync.exe'
$ppsFixture = Join-Path $ppsRepo 'target/release/examples/demo_target.exe'
$ppsData = Join-Path $ppsRepo ".audit/verify-data/$ppsRun"
$ppsEvidence = Join-Path $ppsRepo "specs/001-app-power-profiles/evidence/verification/$ppsRun"
[IO.Directory]::CreateDirectory($ppsEvidence) | Out-Null
$ppsPlanBefore = (& powercfg.exe /getactivescheme) -join "`n"
if ($LASTEXITCODE -ne 0) { throw 'Plan initial illisible.' }
$ppsHash = (Get-FileHash -LiteralPath $ppsExe -Algorithm SHA256).Hash
$ppsProcess = Start-Process -FilePath $ppsExe -ArgumentList @('--demo','--data-dir',('"' + $ppsData + '"')) -WindowStyle Hidden -PassThru
$ppsStarted = $ppsProcess.StartTime.ToUniversalTime()
Write-Output $ppsEvidence
```

Le mode démo conserve ses fichiers dans `$ppsData/simulation`. `--data-dir` seul ne simule pas les plans. Aucun essai ne lance Battlefield. La fenêtre prête porte le badge **Démo**, le message **Démonstration** et, sur des données neuves, les étapes **Pour commencer**. **Lancer le logiciel simulé** n'apparaît qu'après **Activer l’automatisation**. Confirmer cette disponibilité par une capture native, pas par un délai fixe.

## Doctor

Avant de piloter, vérifier en lecture seule le PID retourné au lancement, son heure de création, son chemin complet et le hash du binaire. Ce PID est le processus de suivi. Il porte l'icône de notification, le sélecteur de fichier et les cycles. La fenêtre appartient à un processus enfant `--window` : son parent est ce PID et son chemin est le même exécutable. Ne jamais retrouver un PID par son seul nom. Garder ces variables et enregistrer leur valeur dans `doctor.json`.

```powershell
function Get-PpsWindowProcess {
    $found = @(Get-CimInstance Win32_Process -Filter "ParentProcessId = $($ppsProcess.Id) AND Name = 'power-plan-sync.exe'" |
        Where-Object { [StringComparer]::OrdinalIgnoreCase.Equals($_.ExecutablePath, $ppsExe) -and $_.CreationDate.ToUniversalTime() -ge $ppsStarted })
    if ($found.Count -ne 1) { throw "Processus de fenêtre : $($found.Count), un attendu." }
    Get-Process -Id $found[0].ProcessId
}
$ppsProcess.Refresh()
if ($ppsProcess.HasExited) { throw 'Instance arrêtée.' }
if ($ppsProcess.StartTime.ToUniversalTime() -ne $ppsStarted) { throw 'Identité différente.' }
if (-not [StringComparer]::OrdinalIgnoreCase.Equals($ppsProcess.Path, $ppsExe)) { throw 'Mauvais exécutable.' }
if ((Get-FileHash -LiteralPath $ppsExe -Algorithm SHA256).Hash -ne $ppsHash) { throw 'Binaire remplacé.' }
$ppsWindowProcess = Get-PpsWindowProcess
[ordered]@{
    pid = $ppsProcess.Id
    started_utc = $ppsStarted.ToString('o')
    window_pid = $ppsWindowProcess.Id
    window_started_utc = $ppsWindowProcess.StartTime.ToUniversalTime().ToString('o')
    executable = $ppsExe
    sha256 = $ppsHash
    revision = (git rev-parse HEAD)
    data_directory = $ppsData
    mode = 'demo'
    active_plan_before = $ppsPlanBefore
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $ppsEvidence 'doctor.json')
```

Le processus de fenêtre change à chaque réouverture. Rappeler `Get-PpsWindowProcess` après chaque fermeture ou réouverture de la fenêtre et ne jamais réutiliser un ancien PID de fenêtre.

Comparer aussi le dossier du lancement avec les fichiers effectivement créés. Une autre fenêtre PowerPlanSync, un second processus `--window`, un journal inattendu, un hash différent ou un verrou occupé invalide la cible. Réobserver avant toute reprise après une interruption.

## Drive

Utiliser le plugin Computer Use natif Windows. Lire son `SKILL.md`, `docs/guidance.md`, `docs/api.md` et `docs/confirmations.md` installés avant le premier pilotage. Charger `@oai/sky` dans `node_repl`. Ne pas employer Playwright, des sélecteurs DOM, un client UI Automation maison ou un protocole inventé.

```javascript
if (!globalThis.sky) {
  const { sky } = await import('@oai/sky');
  globalThis.sky = sky;
}
globalThis.ppsApps = await sky.list_apps();
nodeRepl.write(JSON.stringify(ppsApps, null, 2));
```

Dans les objets retournés, sélectionner l'application du processus de fenêtre relevé par Doctor et son unique fenêtre PowerPlanSync. Le processus de suivi a le même chemin mais aucune fenêtre visible. Si les objets portent un PID, il doit être `$ppsWindowProcess.Id`. Affecter cet objet retourné à `globalThis.ppsWindow`. Ne pas construire de handle. Si plusieurs candidats subsistent, inspecter leurs identités avant d'agir.

```javascript
globalThis.ppsState = await sky.get_window_state({window: ppsWindow, include_text: true, include_screenshot: true});
globalThis.ppsWindow = ppsState.window;
nodeRepl.write(ppsState.accessibility?.tree || 'Arbre indisponible');
```

Arrêter la cellule et lire le résultat. Choisir ensuite `globalThis.ppsIndex` dans cet arbre pour le nom accessible demandé par la fiche. Ne jamais conserver un index dans une fiche ou réutiliser un index après une action.

```javascript
await sky.click({window: ppsState.window, element_index: ppsIndex});
globalThis.ppsState = await sky.get_window_state({window: ppsWindow, include_text: true, include_screenshot: true});
globalThis.ppsWindow = ppsState.window;
nodeRepl.write(ppsState.accessibility?.tree || 'Arbre indisponible');
```

Pour un champ éditable observé, utiliser `sky.set_value({window: ppsWindow, element_index: ppsIndex, value: ppsText})`, puis capturer immédiatement l'état. Pour le clavier, utiliser `sky.press_key({window: ppsWindow, key: 'Tab'})`, puis observer le focus. `Return`, `Escape` et `Control_L+a` sont des touches prises en charge. Avec `type_text`, vérifier le focus dans l'observation précédente.

Quatre transitions seulement sont animées, et seulement à la souris : le message de retrait annulable, l'échange de deux lignes après **Monter** ou **Descendre**, l'ouverture de **Ajouter un logiciel** et la coche d'une étape de **Pour commencer** (section Mouvement de `contracts/user-interface.md`). Elles durent de l'ordre de 250 ms (`src/interface/motion.rs`). Une action au clavier n'est jamais animée, et les fermetures sont instantanées. Si la première capture montre un état transitoire observé, attendre 300 ms puis recapturer. Ce délai ne prouve pas la disponibilité. La capture suivante doit montrer le contrôle attendu avant son activation.

Sans le plugin Computer Use, par exemple sous Claude Code, piloter avec `agent-desktop` (lire `agent-desktop skills get desktop` puis `platform`) et les scripts du dépôt, sans autre client UI Automation :

- `scripts/uia-inspect.ps1 -ProcessId $ppsWindowProcess.Id -OutputPrefix …` écrit l'arbre (rôles, noms, états, position dans l'ensemble) et une capture ;
- `scripts/uia-focus.ps1` décrit le focus entre deux touches ;
- `scripts/uia-act.ps1 -ProcessId $ppsWindowProcess.Id -Name '<regex>' -Action Invoke|Toggle|Select|Focus` agit sur un élément nommé ;
- `scripts/tray-act.ps1 -ProcessId $ppsProcess.Id -Action Click` clique sur l'icône de notification, qui appartient au processus de suivi, et `-Item '<regex>'` choisit un élément de son menu par son texte, sans souris ; le script échoue si le menu ne se ferme pas avant l'envoi de la commande ;
- `scripts/send-keys.ps1 -Keys 0x09` envoie des touches virtuelles par `SendInput` à la fenêtre au premier plan ;
- `scripts/uia-events.ps1 -ProcessId $ppsWindowProcess.Id` enregistre régions actives, notifications, noms et focus pendant une action ;
- `scripts/windows-appearance.ps1` et `scripts/display-scale.ps1` changent thème, contraste et échelle ; rétablir la valeur lue avant l'essai dans un bloc `finally`.

Ce que le Narrateur prononce se lit dans **Récapitulatif de la reconnaissance vocale** : `send-keys.ps1 -Keys 0x2D,0x12,0x58` (Insert + Alt + X), puis une capture de cette fenêtre. Fermer ce récapitulatif avant de reprendre le pilotage : il garde sinon le premier plan.

Dans **Ajouter un logiciel**, **Parcourir…** ouvre la fenêtre native **Choisir un logiciel**. Elle appartient au processus de suivi : avec les scripts, l'inspecter par `-ProcessId $ppsProcess.Id`. Réexécuter `sky.list_windows()`, sélectionner la fenêtre retournée de ce dialogue, puis lire son arbre. Saisir uniquement le chemin de `$ppsFixture` dans son champ de fichier observé. Ne pas y saisir une commande shell. Revenir à une nouvelle observation de la fenêtre principale après validation.

## Evidence

Chaque étape prouvée conserve l'action, le résultat visuel, l'arbre accessible et les effets persistés dans `$ppsEvidence`. Nommer les fichiers avec l'ID de fonctionnalité et un compteur, par exemple `configuration-01-before.png`, `configuration-02-after.jpg`, `configuration-02-tree.txt` et `configuration-02-config.json`.

Les captures retournées par sky s'affichent automatiquement. Les sauvegarder uniquement pour l'archive de preuve, sans les réémettre pour les inspecter. Affecter à `globalThis.ppsEvidence` le chemin absolu imprimé par la préparation et à `globalThis.ppsStep` le nom de l'étape courante.

```javascript
{
  const fs = await import('node:fs/promises');
  const path = await import('node:path');
  await fs.writeFile(path.join(ppsEvidence, ppsStep + '-tree.txt'), ppsState.accessibility?.tree || 'Arbre indisponible', 'utf8');
  for (const [index, shot] of ppsState.screenshots.entries()) {
    const match = /^data:image\/(png|jpeg);base64,(.+)$/s.exec(shot.url);
    if (!match) throw new Error('Format de capture inconnu; ne pas déclarer la preuve visuelle complète.');
    const extension = match[1] === 'png' ? '.png' : '.jpg';
    await fs.writeFile(path.join(ppsEvidence, ppsStep + '-' + index + extension), Buffer.from(match[2], 'base64'));
  }
}
```

Lire et copier `simulation/config.json`, `simulation/recovery.json` s'il existe, et le diagnostic local pertinent après les actions. Ne pas injecter la configuration pour prétendre avoir vérifié sa création par l'interface. Les fichiers de fixture sont admis pour une campagne de charge distincte et doivent être annoncés.

Comparer le plan Windows avant et après la démonstration. Une valeur identique ne prouve pas seule l'absence de toute écriture intermédiaire. Associer cette observation aux tests de la frontière simulée et ne pas revendiquer de trace ETW non exécutée.

Le rapport `result.md` liste les fonctionnalités et leurs entrées réellement exercées, le verdict, les limites et les liens vers les preuves. Un contrôle non effectué porte « non exécuté ». Un arbre accessible ne prouve pas les annonces vocales du Narrateur. Une valeur cible n'est pas une mesure.

## Cleanup

Utiliser **Quitter** en bas de la barre latérale (Ctrl+Q) ou **Quitter PowerPlanSync** dans le menu de l'icône. **Fermer** dans la barre de titre arrête seulement le processus de fenêtre et laisse le suivi actif. Vérifier la sortie des deux processus, suivi et fenêtre, puis le plan Windows après sortie. Sur retour échoué, conserver le journal et suivre le parcours de reprise. Ne pas assimiler **Quitter quand même** à une restauration réussie.

Si la démo créée par cet essai est bloquée, relever les preuves, revérifier PID, parent, chemin et heure de création, puis arrêter seulement ces PID : le processus de fenêtre, puis le processus de suivi. Ne pas utiliser un arrêt par nom. Un processus réel ayant un cycle n'est pas un processus démo à tuer pour gagner du temps.

Copier les fichiers de données utiles dans la preuve avant tout nettoyage. Supprimer uniquement le dossier de scratch `$ppsData` créé par la séance, après vérification de son chemin absolu sous `.audit/verify-data/` et de l'arrêt de son propriétaire. Ne pas supprimer `$ppsEvidence`. Confirmer que les captures PNG ou JPEG, l'arbre, la configuration et le rapport existent encore après le nettoyage.

## Helpers

La vérification des plans réels est une campagne séparée et sérialisée. Ne pas l'exécuter pendant une autre instance ou une mesure. L'option d'écriture explicite correspond à l'autorisation de test déjà donnée dans la session. Sinon, conserver la campagne en lecture seule et demander seulement l'autorisation réellement manquante.

```powershell
.\scripts\verify-windows.ps1 -AllowPowerSwitch -Cycles 100 -EvidenceDirectory (Join-Path $ppsRepo 'specs/001-app-power-profiles/evidence/windows-cycles')
```

Ce script requiert PowerShell 7.5 ou plus récent. Il utilise le témoin, prépare des données isolées, attend les plans observés et restaure le plan initial dans son nettoyage. Lire son rapport et vérifier la restauration, même si une assertion échoue. Cent cycles API/processus ne prouvent pas cent parcours UI.

Le lancement et le Doctor précédents utilisent `--demo` et `$ppsData/simulation`. Ils servent au smoke de l'interface et ne qualifient jamais SC-004.

Pour SC-004, suivre les quatre états de **Mesurer la consommation et la stabilité** dans `specs/001-app-power-profiles/quickstart.md`. Préparer une instance réelle sans `--demo`, avec vingt règles, puis refaire les contrôles d'identité Doctor sous des variables distinctes `$ppsRealProcess`, `$ppsRealExe`, `$ppsRealData` et `$ppsRealEvidence`. Utiliser le PID exact du processus de suivi et le chemin vérifié de cette instance. L'état de fenêtre est une déclaration de l'opérateur. La commande ne lance ni n'arrête aucun processus et ne change aucun plan.

```powershell
.\scripts\measure-overhead.ps1 -ProcessId $ppsRealProcess.Id -ExecutablePath $ppsRealExe -WindowState hidden -OutputDirectory $ppsRealEvidence -DataDirectory $ppsRealData -RuleCount 20 -DurationSeconds 1800 -WarmupSeconds 60 -SampleIntervalMilliseconds 5000
```

Répéter selon les quatre états du quickstart, dont `-WindowState open` après ouverture effectivement constatée. Utiliser un dossier de preuve distinct par état. Les valeurs par défaut du script restent adaptées à un usage général. La campagne SC-004 utilise explicitement 60 s de stabilisation, un intervalle de 5 s et une mesure de 1800 s. Lire les échantillons et les statistiques JSON. Les limites sont 40 MiB de mémoire privée masquée, 100 MiB ouverte et 0,1 % CPU de la machine. Une mesure raccourcie reste un essai du script, pas une preuve de trente minutes. Le script ne relève que le PID donné. Fenêtre ouverte, le processus `--window` consomme à part : `scripts/measure-window-states.ps1` relève la somme des deux processus.

Lors de la création ou d'une révision du skill, exécuter Launch, Doctor, une fonctionnalité, Evidence et Cleanup une fois avant de déclarer ce skill validé. En délégation documentaire, laisser cette exécution au coordinateur et signaler explicitement qu'elle reste à faire. Utiliser `/maintain-verification-skill` quand les parcours ou les noms changent.
