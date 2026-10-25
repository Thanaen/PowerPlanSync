# PowerPlanSync 0.1.0

PowerPlanSync associe les plans d'alimentation déjà installés dans Windows aux logiciels ouverts. Il fonctionne sur Windows 11 x64 avec les droits de votre session. Il ne crée pas de plan et ne modifie pas le contenu des plans existants.

## Ouvrir l'application

Extrayez entièrement l'archive dans un dossier de votre choix, puis ouvrez `power-plan-sync.exe`. Conservez les fichiers de notices fournis à côté de l'application. Aucun programme d'installation, service Windows ou changement global du PATH n'est nécessaire pour ce paquet.

La page **Logiciels** affiche **Plan actif dans Windows**, puis un statut qui dit ce que fait l'automatisation, par exemple « Automatisation active · Battlefield 6 est ouvert ». Au premier lancement, l'automatisation est désactivée et rien ne change dans Windows. Pour explorer l'interface avec des plans simulés, ouvrez un terminal dans le dossier extrait et exécutez cette commande facultative.

```powershell
.\power-plan-sync.exe --demo
```

La démonstration utilise ses propres données et n'applique pas de plan Windows.

## Associer un logiciel à un plan

Au premier lancement, la page **Logiciels** affiche **Pour commencer**, en trois étapes.

1. Cliquez sur **Ajouter un logiciel** (Ctrl+N). Choisissez **Parcourir…** pour sélectionner le fichier `.exe` du logiciel, ou **Application ouverte** pour choisir une application en cours dont le chemin est vérifiable. Vérifiez le nom affiché et le fichier. Deux fichiers portant le même nom dans des dossiers différents sont des logiciels distincts. Choisissez **Plan pendant l'exécution**, puis cliquez sur **Ajouter**.
2. Vérifiez le plan proposé dans **Confirmer le plan de retour**, puis cliquez sur **Confirmer**. C'est le plan appliqué quand le dernier logiciel suivi se ferme. Si le plan Utilisation normale n'existe pas sur votre PC, aucun plan n'est choisi à votre place.
3. Cliquez sur **Activer l'automatisation**.

Ajouter un logiciel ne suffit pas à modifier le plan : rien ne change avant l'étape 3. La liste **Logiciels associés** remplace ensuite ces étapes.

Chaque logiciel a un interrupteur **Suivre**. Un logiciel qui n'est pas suivi est ignoré. Les boutons **Monter**, **Descendre**, **Modifier** et **Supprimer** organisent la liste. Si plusieurs logiciels suivis sont ouverts, le premier de la liste décide du plan. Un badge indique l'état de chaque logiciel : **Appliqué**, **Ouvert**, **Suspendu** ou **Refusé**. Une suppression peut être annulée avec **Annuler** ou Ctrl+Z tant que son message est affiché.

Si un logiciel ne peut pas être identifié, l'application ne cherche pas à contourner ses protections. Il apparaît comme « Fichier inaccessible · impossible à suivre ». Si un logiciel a été déplacé, **Modifier** permet de choisir son nouveau fichier sans perdre le plan associé.

La palette de commandes (Ctrl+Maj+P ou bouton **Commandes**) donne accès aux actions principales au clavier.

## Mettre en pause ou quitter

**Mettre en pause** (Ctrl+Alt+P) suspend l'automatisation et revient au plan de retour réservé au cycle courant. **Reprendre** relit la situation et relance l'automatisation. Si un logiciel suivi est toujours ouvert, son plan peut alors être appliqué. La pause est conservée après un redémarrage.

Le bouton de fermeture de la fenêtre la masque seulement. L'application continue dans la zone de notification Windows, parfois parmi les icônes masquées (^). Cliquez sur son icône pour rouvrir la fenêtre, ou faites un clic droit pour son menu. La couleur de l'icône résume l'état : bleu quand le plan d'un logiciel est appliqué, gris quand aucun plan de logiciel n'est appliqué, orange quand l'automatisation est suspendue, rouge quand une action est nécessaire.

Pour arrêter l'application, utilisez **Quitter** dans la barre latérale ou **Quitter PowerPlanSync** dans le menu de l'icône. L'application revient d'abord au plan de retour. En cas d'échec, le message « Le retour au plan Utilisation normale a échoué » propose **Réessayer**, **Ouvrir les options d'alimentation** ou **Quitter quand même**. **Quitter quand même** laisse le plan actif tel quel et conserve les informations de récupération : au prochain lancement, PowerPlanSync tentera le retour si le plan n'a pas changé.

## Garder la priorité sur Windows

Si vous changez le plan dans Windows pendant un cycle, votre choix reste prioritaire jusqu'à la fin du cycle. Le statut indique « Automatisation suspendue · plan choisi dans Windows » et PowerPlanSync ne reviendra pas au plan de retour. Cliquez sur **Reprendre l'automatisation** pour réappliquer le plan du logiciel suivi encore ouvert.

Si Windows refuse un changement de plan, le message « Windows a refusé le changement de plan » propose **Réessayer** et **Ouvrir les options d'alimentation**. PowerPlanSync ne réessaie pas seul.

Le plan de retour d'un cycle est choisi au début de ce cycle. Si vous le changez dans **Réglages** pendant qu'un logiciel suivi est ouvert, le cycle en cours revient à l'ancien plan et le nouveau s'applique à partir du cycle suivant.

**Changer de plan sur batterie** est désactivé par défaut : sur batterie, aucun logiciel ne change le plan et un cycle en cours revient au plan de retour. La veille seule ne demande pas un retour. Au réveil, l'application relit la situation avant de décider. Le suivi est limité à une session interactive active et une seule instance de contrôle à la fois.

## Données et démarrage avec Windows

Vos logiciels associés, vos réglages et les informations de récupération sont enregistrés dans `%LOCALAPPDATA%\PowerPlanSync`. Les fichiers principaux sont `config.json`, `config.backup.json`, `recovery.json` et les diagnostics locaux `activity.log` et `activity.previous.log`. Ils ne sont pas inclus dans ce paquet et ne sont pas envoyés à un serveur.

Le mode démo utilise par défaut `%LOCALAPPDATA%\PowerPlanSyncDemo\simulation`. L'option facultative `--data-dir CHEMIN` choisit un autre dossier. En démo, les fichiers sont dans son sous-dossier `simulation`. **Cette option seule ne simule pas Windows** ; il faut aussi `--demo` pour simuler les plans.

**Démarrer avec Windows** est facultatif et désactivé par défaut. Cette option est disponible avec le dossier de données normal, sans mode démo. Elle ouvre l'application dans la zone de notification à votre connexion. Ne déplacez pas le dossier de PowerPlanSync tant que cette option est activée. Désactivez-la avant de déplacer le dossier, puis réactivez-la depuis le nouvel emplacement si vous le souhaitez.

Pour sauvegarder vos réglages, quittez normalement l'application puis copiez son dossier de données. Ne supprimez pas un journal de récupération pour masquer un problème.

## Réglages illisibles

Si `config.json` est endommagé ou incompatible, PowerPlanSync affiche « Les réglages sont illisibles ». Aucun plan n'a été changé et l'automatisation reste arrêtée. Le fichier et sa copie de secours sont conservés sans modification. La copie de secours `config.backup.json` contient la version précédente de vos réglages, enregistrée à chaque modification.

1. Quittez PowerPlanSync, ou laissez-le ouvert sur ce message.
2. Cliquez sur **Ouvrir le dossier des données**.
3. Gardez une copie de `config.json` de côté, par exemple sous le nom `config.illisible.json`.
4. Remplacez `config.json` par une copie de `config.backup.json`.
5. Cliquez sur **Relire les réglages**, ou relancez PowerPlanSync.

Si la copie de secours est elle aussi illisible, renommez `config.json` : PowerPlanSync repartira de **Pour commencer**. Le journal de récupération `recovery.json` n'est pas concerné par cette procédure.

## Après un arrêt forcé

Un arrêt forcé, un crash ou l'arrêt de Windows peut laisser le dernier plan actif. Au prochain lancement, PowerPlanSync examine son journal et vérifie le plan actif avant de récupérer le cycle. Si un autre plan a été choisi entre-temps, il est conservé. Vérifiez le plan affiché et les éventuels messages au redémarrage.

Si vous n'avez plus besoin de l'application, désactivez son démarrage avec Windows puis quittez-la normalement. Vous pouvez ensuite retirer le dossier extrait. Les données locales restent séparées ; leur conservation ou leur suppression vous appartient.

## Limites et notices

Les plans classiques d'alimentation et le mode **Recommandé** des Paramètres Windows peuvent être des réglages distincts. PowerPlanSync sélectionne les plans existants ; il ne garantit pas un changement de ce mode.

Aucun gain de FPS, de température ou de consommation n'est garanti. Battlefield 6 n'a pas été testé pour cette version. Les essais de développement utilisent un petit exécutable témoin.

Les licences des composants sont accessibles dans **Aide et limites**, avec **Voir les licences**. Les composants tiers et leurs textes de licence figurent dans [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) et [notices/THIRD_PARTY_LICENSES.txt](notices/THIRD_PARTY_LICENSES.txt). Ces notices ne définissent pas une licence de redistribution de PowerPlanSync.

Le fichier `manifest.json` donne la taille et le SHA-256 de l'exécutable et des documents livrés. Le fichier `.zip.sha256` fourni à côté de l'archive permet de comparer son empreinte. Ces empreintes vérifient l'identité des fichiers ; elles ne constituent pas une signature d'éditeur.
