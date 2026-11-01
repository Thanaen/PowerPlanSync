# PowerPlanSync 0.1.0

PowerPlanSync associe les plans d'alimentation déjà installés dans Windows aux logiciels ouverts. Il fonctionne sur Windows 11 x64 avec les droits de votre session. Il ne crée pas de plan et ne modifie pas le contenu des plans existants.

## Ouvrir l'application

Extrayez entièrement l'archive dans un dossier de votre choix, puis ouvrez `power-plan-sync.exe`. Conservez les fichiers de notices fournis à côté de l'application. Aucun programme d'installation, service Windows ou changement global du PATH n'est nécessaire pour ce paquet.

La fenêtre affiche le plan actif observé et la raison de son état. Au premier lancement, l'automatisation est désactivée. Pour explorer l'interface avec des plans simulés, ouvrez un terminal dans le dossier extrait et exécutez cette commande facultative.

```powershell
.\power-plan-sync.exe --demo
```

La démonstration utilise ses propres données et n'applique pas de plan Windows.

## Associer un logiciel à un plan

1. Cliquez sur **Ajouter un logiciel**.
2. Cliquez sur **Parcourir…** et sélectionnez le fichier `.exe` du logiciel. Vous pouvez aussi choisir **Application ouverte…** pour sélectionner une application dont le chemin est vérifiable.
3. Vérifiez le nom affiché et le chemin complet. Deux fichiers portant le même nom dans des dossiers différents sont des logiciels distincts.
4. Choisissez **Plan pendant l'exécution**, puis cliquez sur **Enregistrer**.
5. Dans **Réglages**, choisissez **Après fermeture, revenir à**. Si un plan est proposé sans être encore enregistré, cliquez sur **Confirmer ce plan de retour**.
6. Cochez **Activer l'automatisation**.

Le suivi commence lorsque la règle et le retour sont valides et que vous activez l'automatisation. Enregistrer une règle ne suffit pas à modifier le plan. Les boutons **Modifier**, **Supprimer**, **Monter** et **Descendre** permettent d'organiser les règles. Le premier logiciel actif de la liste a priorité. Une règle décochée est ignorée.

Si un logiciel ne peut pas être identifié, l'application ne cherche pas à contourner ses protections. Vérifiez le fichier sélectionné et le message affiché. Un chemin invalide ou déjà présent doit être corrigé avant l'enregistrement.

## Faire une pause ou quitter

**Pause** suspend le suivi et tente le retour réservé au cycle courant. **Reprendre** relit la situation et relance le suivi. Si le logiciel est toujours ouvert, son plan peut alors être appliqué.

**Fermer la fenêtre** et le bouton de fermeture standard masquent la fenêtre. L'application continue dans la zone de notification Windows. Ouvrez son icône, éventuellement dans les icônes masquées, pour afficher la fenêtre ou son menu.

Pour arrêter l'application, utilisez **Quitter PowerPlanSync** dans la fenêtre ou **Quitter** dans le menu de notification. L'application tente d'abord le retour protégé. En cas d'échec, lisez le plan encore observé et utilisez **Réessayer** ou ouvrez les réglages Windows. **Quitter quand même** conserve les informations de récupération ; ce choix ne signifie pas que le retour a réussi.

## Garder la priorité sur Windows

Si vous changez le plan dans Windows et que PowerPlanSync observe un plan différent de celui qu'il a confirmé, votre choix reste prioritaire pour le cycle courant. Le statut indique que le réglage Windows est conservé. Cliquez sur **Reprendre** pour autoriser une nouvelle décision.

Le retour d'un cycle est réservé au début de ce cycle. Si vous changez le plan de retour pendant qu'un logiciel suivi est actif, la modification s'applique au cycle suivant.

**Autoriser sur batterie** est désactivé par défaut. La veille seule ne demande pas un retour. Au réveil, l'application relit la situation avant de décider. Le suivi est limité à une session interactive active et une seule instance de contrôle à la fois.

## Données et démarrage avec Windows

Vos règles et les informations de récupération sont enregistrées dans `%LOCALAPPDATA%\PowerPlanSync`. Les fichiers principaux sont `config.json`, `config.backup.json`, `recovery.json` et les diagnostics locaux `activity.log` et `activity.previous.log`. Ils ne sont pas inclus dans ce paquet et ne sont pas envoyés à un serveur.

Le mode démo utilise par défaut `%LOCALAPPDATA%\PowerPlanSyncDemo\simulation`. L'option facultative `--data-dir CHEMIN` choisit un autre dossier. En démo, les fichiers sont dans son sous-dossier `simulation`. **Cette option seule ne simule pas Windows** ; il faut aussi `--demo` pour simuler les plans.

**Démarrer avec Windows** est facultatif et désactivé par défaut. Cette option est disponible avec le dossier de données normal, sans mode démo. Elle ouvre l'application dans la zone de notification à votre connexion. Conservez l'exécutable au même emplacement tant que cette option est active. Désactivez-la avant de déplacer le dossier, puis réactivez-la depuis le nouvel emplacement si vous le souhaitez.

Pour sauvegarder vos réglages, quittez normalement l'application puis copiez son dossier de données. Ne supprimez pas un journal de récupération pour masquer un problème. Si la configuration est illisible ou incompatible, l'application conserve les fichiers et affiche une erreur.

## Après un arrêt forcé

Un arrêt forcé, un crash ou l'arrêt de Windows peut laisser le dernier plan actif. Au prochain lancement, PowerPlanSync examine son journal et le plan réellement observé avant de récupérer le cycle. Un autre réglage Windows observé reste prioritaire. Vérifiez le plan affiché et les éventuels messages au redémarrage.

Si vous n'avez plus besoin de l'application, désactivez son démarrage avec Windows puis quittez-la normalement. Vous pouvez ensuite retirer le dossier extrait. Les données locales restent séparées ; leur conservation ou leur suppression vous appartient.

## Limites et notices

Les plans classiques d'alimentation et le mode **Recommandé** des Paramètres Windows peuvent être des réglages distincts. PowerPlanSync sélectionne les plans existants ; il ne garantit pas un changement de ce mode.

Aucun gain de FPS, de température ou de consommation n'est garanti. Battlefield 6 n'a pas été testé pour cette version. Les essais de développement utilisent un petit exécutable témoin.

Les licences des composants sont accessibles dans **Aide et limites**, avec **Voir les licences**. Les composants tiers et leurs textes de licence figurent dans [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) et [notices/THIRD_PARTY_LICENSES.txt](notices/THIRD_PARTY_LICENSES.txt). Ces notices ne définissent pas une licence de redistribution de PowerPlanSync.

Le fichier `manifest.json` donne la taille et le SHA-256 de l'exécutable et des documents livrés. Le fichier `.zip.sha256` fourni à côté de l'archive permet de comparer son empreinte. Ces empreintes vérifient l'identité des fichiers ; elles ne constituent pas une signature d'éditeur.
