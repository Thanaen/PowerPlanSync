---
name: verify
description: Vérifier PowerPlanSync sur son interface Windows native, ses cycles de plans et son coût mesuré après une modification produit, avec données isolées et preuves conservées.
---

# Vérifier PowerPlanSync

Lire d'abord `AGENTS.md`, `.specify/memory/constitution.md`, `.specify/feature.json` et [la carte des fonctionnalités](features/README.md). La conception reste dans `specs/001-app-power-profiles`. Ce skill décrit comment vérifier le produit, sans remplacer sa spécification.

## Launch

Utiliser le binaire produit `target/release/power-plan-sync.exe`, pas `ui_probe.exe` ni `ui_candidate.exe`. Dans le dépôt principal de cette machine, son chemin est `C:\Users\Utilisateur\Documents\Dev\PowerPlanSync\target\release\power-plan-sync.exe`. Exécuter les commandes depuis la racine du checkout à vérifier. Ne pas piloter une instance utilisée par une autre campagne.

Si nécessaire, compiler une fois avant les essais avec `cargo build --locked --release --bins --examples`. L'installation locale requiert `CARGO_HOME=.tools/cargo`, `RUSTUP_HOME=.tools/rustup` et `.tools/cargo/bin` dans le PATH de cette commande. Ne pas reconstruire pendant une mesure de consommation.

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

Le mode démo conserve ses fichiers dans `$ppsData/simulation`. `--data-dir` seul ne simule pas les plans. Aucun essai ne lance Battlefield. La fenêtre prête présente PowerPlanSync, un plan simulé et les commandes de démonstration. Confirmer cette disponibilité par une capture native, pas par un délai fixe.

## Doctor

Avant de piloter, vérifier en lecture seule le PID retourné au lancement, son heure de création, son chemin complet et le hash du binaire. Ne jamais retrouver le PID par son seul nom. Garder ces variables et enregistrer leur valeur dans `doctor.json`.

```powershell
$ppsProcess.Refresh()
if ($ppsProcess.HasExited) { throw 'Instance arrêtée.' }
if ($ppsProcess.StartTime.ToUniversalTime() -ne $ppsStarted) { throw 'Identité différente.' }
if (-not [StringComparer]::OrdinalIgnoreCase.Equals($ppsProcess.Path, $ppsExe)) { throw 'Mauvais exécutable.' }
if ((Get-FileHash -LiteralPath $ppsExe -Algorithm SHA256).Hash -ne $ppsHash) { throw 'Binaire remplacé.' }
[ordered]@{
    pid = $ppsProcess.Id
    started_utc = $ppsStarted.ToString('o')
    executable = $ppsExe
    sha256 = $ppsHash
    revision = (git rev-parse HEAD)
    data_directory = $ppsData
    mode = 'demo'
    active_plan_before = $ppsPlanBefore
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $ppsEvidence 'doctor.json')
```

Comparer aussi le dossier du lancement avec les fichiers effectivement créés. Une autre fenêtre PowerPlanSync, un journal inattendu, un hash différent ou un verrou occupé invalide la cible. Réobserver avant toute reprise après une interruption.

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

Dans les objets retournés, sélectionner l'application correspondant au chemin exact vérifié par Doctor et son unique fenêtre PowerPlanSync. Affecter cet objet retourné à `globalThis.ppsWindow`. Ne pas construire de handle. Si plusieurs candidats subsistent, inspecter leurs identités avant d'agir.

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

Les panneaux et listes Slint peuvent finir leur animation après l'action. Si la première capture montre cet état transitoire observé, attendre 200 ms puis recapturer. Ce délai ne prouve pas la disponibilité. La capture suivante doit montrer le contrôle attendu avant son activation.

**Parcourir les fichiers** ouvre la fenêtre native **Choisir un logiciel**. Réexécuter `sky.list_windows()`, sélectionner la fenêtre retournée de ce dialogue, puis lire son arbre. Saisir uniquement le chemin de `$ppsFixture` dans son champ de fichier observé. Ne pas y saisir une commande shell. Revenir à une nouvelle observation de la fenêtre principale après validation.

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

Utiliser **Quitter PowerPlanSync** dans la fenêtre ou **Quitter** dans son menu de notification. **Fermer la fenêtre** laisse l'application active. Vérifier la sortie du PID propriétaire et le plan Windows après sortie. Sur retour échoué, conserver le journal et suivre le parcours de reprise. Ne pas assimiler **Quitter quand même** à une restauration réussie.

Si la démo créée par cet essai est bloquée, relever les preuves, revérifier PID, chemin et heure de création, puis arrêter seulement ce PID. Ne pas utiliser un arrêt par nom. Un processus réel ayant un cycle n'est pas un processus démo à tuer pour gagner du temps.

Copier les fichiers de données utiles dans la preuve avant tout nettoyage. Supprimer uniquement le dossier de scratch `$ppsData` créé par la séance, après vérification de son chemin absolu sous `.audit/verify-data/` et de l'arrêt de son propriétaire. Ne pas supprimer `$ppsEvidence`. Confirmer que les captures PNG ou JPEG, l'arbre, la configuration et le rapport existent encore après le nettoyage.

## Helpers

La vérification des plans réels est une campagne séparée et sérialisée. Ne pas l'exécuter pendant une autre instance ou une mesure. L'option d'écriture explicite correspond à l'autorisation de test déjà donnée dans la session. Sinon, conserver la campagne en lecture seule et demander seulement l'autorisation réellement manquante.

```powershell
.\scripts\verify-windows.ps1 -AllowPowerSwitch -Cycles 100 -EvidenceDirectory (Join-Path $ppsRepo 'specs/001-app-power-profiles/evidence/windows-cycles')
```

Ce script requiert PowerShell 7.5 ou plus récent. Il utilise le témoin, prépare des données isolées, attend les plans observés et restaure le plan initial dans son nettoyage. Lire son rapport et vérifier la restauration, même si une assertion échoue. Cent cycles API/processus ne prouvent pas cent parcours UI.

Pour mesurer une instance déjà préparée avec vingt règles, utiliser son PID exact et son chemin Doctor. L'état de fenêtre est une déclaration de l'opérateur. La commande ne lance ni n'arrête aucun processus et ne change aucun plan.

```powershell
.\scripts\measure-overhead.ps1 -ProcessId $ppsProcess.Id -ExecutablePath $ppsExe -WindowState hidden -OutputDirectory $ppsEvidence -DataDirectory (Join-Path $ppsData 'simulation') -RuleCount 20 -DurationSeconds 1800 -WarmupSeconds 60 -SampleIntervalMilliseconds 5000
```

Répéter avec `-WindowState open` après ouverture effectivement constatée. Les valeurs par défaut du script restent adaptées à un usage général. La campagne SC-004 utilise explicitement 60 s de stabilisation, un intervalle de 5 s et une mesure de 1800 s. Lire les échantillons et les statistiques JSON. Les limites sont 40 MiB de mémoire privée masquée, 100 MiB ouverte et 0,1 % CPU de la machine. Une mesure raccourcie reste un essai du script, pas une preuve de trente minutes.

Lors de la création ou d'une révision du skill, exécuter Launch, Doctor, une fonctionnalité, Evidence et Cleanup une fois avant de déclarer ce skill validé. En délégation documentaire, laisser cette exécution au coordinateur et signaler explicitement qu'elle reste à faire. Utiliser `/maintain-verification-skill` quand les parcours ou les noms changent.
