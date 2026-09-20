# Carte de vérification de PowerPlanSync

Lire [le skill principal](../SKILL.md) avant une fiche. La carte décrit les parcours à vérifier. Elle ne certifie pas qu'ils ont tous été exécutés.

## Préconditions communes

Utiliser le binaire produit identifié par PID, chemin, heure de création et SHA-256. La séance possède son dossier `.audit/verify-data/` et son dossier de preuve sous `specs/001-app-power-profiles/evidence/verification/`. Les parcours UI commencent en `--demo` et lisent les fichiers du sous-dossier `simulation`. Le témoin est `target/release/examples/demo_target.exe`. Aucune entrée ne requiert Battlefield.

## Convention de pilotage

`sky` dans `node_repl` pilote la vraie fenêtre Windows. À chaque étape, observer l'arbre et la capture, choisir le contrôle par son nom observé, effectuer une seule action, puis observer de nouveau. Les indices, PID et identifiants de fenêtre sont propres à la séance. Ne pas les figer dans cette carte. Après une animation observée, attendre 200 ms et recapturer avant l'action suivante.

Chaque action des fiches utilise les appels exacts décrits dans Drive du skill. « Cliquer » signifie `sky.click` avec l'index de la dernière observation. « Saisir » signifie `sky.set_value` sur le champ observé ou `sky.type_text` après vérification du focus. « Touche » signifie `sky.press_key` suivi d'une capture.

## Preuves et entrées

Associer à chaque ID exercé l'action, un PNG, l'arbre et l'effet constaté dans les fichiers ou Windows. Couvrir chaque entrée listée ou la marquer « non exécutée ». La configuration injectée pour vingt règles ne remplace pas le parcours Ajouter. Les tests Rust et le harnais de cycles complètent la preuve UI sans la remplacer. L'exécution initiale de cette carte reste à consigner par le coordinateur dans son rapport de vérification.

## Fonctionnalités

- [Configuration des logiciels](configuration.md). Ajout par fichier et application ouverte, édition, priorité et confirmation du retour.
- [Cycles et priorité du réglage Windows](cycles-windows.md). Activation, fermeture, pause, reprise et retour protégé.
- [Fenêtre et zone de notification](window-tray.md). Masquage, réouverture et sortie de l'instance propriétaire.
- [Clavier et accessibilité](accessibility.md). Focus, erreurs, défilement, échelle, contraste et Narrateur.
