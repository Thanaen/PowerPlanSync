# Valider PowerPlanSync sur Windows

Ce guide décrit la validation à réaliser pendant l'implémentation. La version Slint et ses
scripts existent ; leurs preuves sont dans `evidence/`. La migration GPUI (T046 à T072) reprend
les mêmes vérifications sur la nouvelle interface. Un contrôle non exécuté porte la mention
« non exécuté ».

## Préparer l'environnement

1. Utiliser Windows 11 x64 avec un compte standard et deux plans installés.
2. Installer la chaîne Rust MSVC retenue en T001, les outils C++ et le SDK Windows.
3. Vérifier les versions avec `rustc --version`, `cargo --version` et la version du SDK.
4. Exécuter `cargo build --locked --release` après création du package.
5. Installer `rustfmt` et `clippy` avec `rustup component add rustfmt clippy` tant que T046 n'est pas faite, puis exécuter `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` et `cargo test --locked`.
6. Conserver versions, sorties et code de retour dans `evidence/validation/`.

Dans un conteneur Linux, le crate ne compile pas pour Linux car il dépend de `windows`. Seuls ces
contrôles s'y exécutent :

```bash
rustup component add rustfmt clippy
cargo fmt --check
cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings
```

Après T046, ce contrôle demande aussi `llvm-rc` dans le `PATH` : la fonctionnalité
`windows-manifest` de GPUI intègre un manifeste à la compilation.

Les tests restent à exécuter sous Windows, puis dans la CI Windows prévue (issue #7). Si T057
aboutit, `cargo run --locked --features screenshots --bin screenshots` produit aussi les captures
hors écran sous Linux, avec un pilote Vulkan logiciel comme Mesa lavapipe.

Les essais de changement de plan utilisent un PC de test ou une séance explicitement dédiée.
Capturer le plan actif avec `powercfg /getactivescheme` avant de commencer. Restaurer ce plan en
fin de séance, même si l'application échoue. Aucun script de validation n'active l'option de
démarrage automatique sans étape explicite et réversible.

## Vérifier l'interface isolée

Lancer le mode de démonstration. Pendant la migration, ajouter `--gpui` après T056 pour ouvrir
l'interface GPUI. Après T071, GPUI est l'interface par défaut et `--gpui` n'existe plus.

```powershell
cargo run --locked --release -- --demo --data-dir .audit/demo-data
```

Ajouter une règle, choisir le retour, activer le suivi, simuler un lancement et une fermeture.
Vérifier les noms et les explications affichés. Le mode demo ne doit écrire aucun plan réel.
Fermer la fenêtre puis la rouvrir depuis la notification.

Parcourir tous les contrôles au clavier. Répéter avec le Narrateur et inspecter l'arbre accessible.
Tester les thèmes clair, sombre et contrasté, les échelles 100 %, 150 % et 200 %, le texte agrandi,
une fenêtre de faible hauteur et un déplacement entre écrans de DPI différents.
Conserver les captures, l'arbre inspecté et les résultats dans `evidence/accessibility/`, et ceux
de l'interface GPUI dans `evidence/accessibility/gpui/`.

Pour l'interface GPUI, comparer chaque écran aux captures de [design/ecrans](design/ecrans) et
vérifier le [contrat d'interface](contracts/user-interface.md). Vérifier aussi le mouvement :

1. Les quatre transitions du contrat s'animent à la souris, jamais au clavier.
2. En build debug, `POWERPLANSYNC_MOTION_SLOWDOWN=4` ralentit chaque animation quatre fois pour
   la juger au ressenti face aux images de [design/mouvement](design/mouvement), par exemple avec
   `$env:POWERPLANSYNC_MOTION_SLOWDOWN=4; cargo run --locked -- --demo --gpui` (sans `--release`).
3. Avec « Effets d'animation » désactivé dans Windows, l'état final s'affiche directement, y
   compris après un changement du réglage pendant que l'application est ouverte.

## Prouver le cycle complet

Après T019, lancer l'application avec un répertoire isolé, sans `--demo`.
Créer deux copies du petit exécutable `demo_target` dans deux répertoires différents.
Configurer seulement le chemin de la première copie.

1. Choisir un plan cible différent du plan de retour.
2. Lancer le logiciel configuré et constater le plan actif avec `powercfg /getactivescheme`.
3. Lancer une seconde instance, puis fermer la première. Le plan cible doit rester actif.
4. Fermer la dernière instance. Le retour doit se produire après la grâce de trois secondes.
5. Lancer la copie homonyme non configurée. Aucun cycle ne doit démarrer.
6. Refaire l'essai alors que le plan cible est déjà actif. Le retour explicite doit tout de même s'appliquer.
7. Répéter 100 cycles. Chaque processus vit au moins dix secondes.

Le futur `scripts/verify-windows.ps1` écrit l'instant du lancement, de la sortie et des changements
réellement relus dans Windows. Il calcule les p95. Exclure les snapshots incomplets du résultat
normal et les compter séparément. Échouer si le p95 activation dépasse 2 s ou le p95 retour 5 s.
L'horloge part de la création ou de la sortie réelle du processus témoin, jamais de sa détection.
Journaliser séparément observation, fin de grâce, appel Windows et confirmation. Distribuer les
lancements et sorties sur l'intervalle de sondage pour éviter un essai artificiellement favorable.
Le prototype T002 mesure déjà cette chaîne. Une cible dépassée exige une correction ou un
amendement explicite de SC-002, sans changer silencieusement le point de départ du chronomètre.

## Vérifier les protections

| Scénario | Résultat attendu | Preuve prévue |
| --- | --- | --- |
| Deux règles actives | Première règle gagnante, puis suivante après grâce | Journal et GUID relus |
| Plan changé manuellement en jeu | Suspension, aucun retour automatique à la fin | Trace des écritures |
| Reprendre pendant cette suspension | Recalcul immédiat, nouveau cycle | Transition et GUID observé |
| Pause puis relance | Pause conservée, aucun suivi actif | Configuration relue et trace |
| Changer le retour pendant un cycle | Retour du cycle inchangé, nouveau retour au cycle suivant | Deux cycles |
| Plan cible ou retour manquant | Erreur visible, aucun plan substitué | Erreur et absence d'écriture |
| Lecture de processus indéterminée | Pas de faux nouveau match, délai puis blocage pour instance connue | Horloge et erreurs injectées |
| Journal non inscriptible | Pas de bascule | Absence d'appel Windows |
| Coupure à chaque phase du journal | Issue conforme au modèle de données après relance | Matrice de coupures |
| Journal corrompu ou version future | Fichier conservé, suivi bloqué | Fichier et état |
| Batterie non autorisée | Retour protégé, règles inéligibles | Source et GUID observés |
| Veille et reprise | Relecture avant toute écriture | Trace d'ordonnancement |
| Changement de session | Un seul contrôleur écrit, retour au mieux | Deux sessions standard |
| Quitter avec retour refusé | Choix explicites et journal conservé | Fenêtre et fichier |

Les erreurs difficiles à reproduire utilisent un adaptateur simulé pour une preuve déterministe.
Les droits standards, la veille, l'accessibilité et le verrou entre sessions exigent aussi un essai réel.
Un test simulé ne remplace pas ces vérifications.

## Mesurer la consommation et la stabilité

Après T039, utiliser `scripts/measure-overhead.ps1` sur le binaire optimisé.
Consigner processeur, nombre de processeurs logiques, RAM, build Windows, versions des dépendances,
nombre de processus et nombre de règles. Le scénario nominal a 20 règles et environ 200 processus.
Répéter avec l'exécutable témoin ouvert pour mesurer séparément les deux applications.

Mesurer les états suivants après 60 secondes de stabilisation, pendant 30 minutes chacun.

1. Notification seule avec règles actives et aucun logiciel correspondant.
2. Fenêtre ouverte au repos.
3. Notification seule après cent ouvertures et fermetures de la fenêtre.
4. Pause persistante, pour vérifier l'arrêt du timer de suivi.

Calculer le CPU du processus depuis le delta de temps processeur, divisé par le temps écoulé et
le nombre de processeurs logiques, multiplié par 100. Collecter la mémoire privée engagée,
le working set, les handles et les threads toutes les cinq secondes. La mémoire privée maximale
après stabilisation doit respecter 40 Mio en notification seule et 100 Mio fenêtre ouverte.
Fenêtre ouverte, additionner le processus de suivi et le processus de fenêtre `--window` (D18).
En notification seule, vérifier aussi qu'aucun processus de fenêtre ne reste.
La moyenne CPU du suivi doit rester ≤ 0,1 % de la capacité totale.

Après cent cycles de fenêtre, comparer au premier état stabilisé. Échouer au-delà de 5 Mio
supplémentaires ou de dix handles supplémentaires, même si le plafond absolu reste respecté.
Vérifier dans une trace ETW de diagnostic l'absence de rendu périodique fenêtre fermée.
En pause, aucune observation de processus ne doit être programmée.
Le snapshot par seconde implique des réveils pendant le suivi. L'objectif n'est pas zéro réveil.

Conserver le CSV brut et le rapport dans `evidence/performance/`.
Il n'existe pas de version antérieure du produit. Ne pas fabriquer un ratio avant/après sur du code absent.
Comparer les futurs changements à la première version mesurée avec des essais intercalés identiques.

## Vérifier l'exécutable témoin et l'usage réel

À la demande de l'utilisateur, ne pas lancer Battlefield 6. Sélectionner `demo_target.exe`
et vérifier lancement, maintien et fermeture avec un lanceur témoin distinct encore ouvert.
Répéter après un redémarrage court, avec plusieurs instances et après un arrêt forcé du témoin.
Une copie homonyme dans un autre répertoire ne doit pas déclencher la règle.
Tester les erreurs de processus protégé par injection, sans prétendre valider un anti-triche.

Une éventuelle vérification ultérieure sur un jeu fera l'objet d'une séance distincte.
Les mesures du témoin ne justifient aucune promesse de gain de FPS ou de compatibilité Battlefield.

Enfin, faire configurer l'outil par cinq personnes non spécialistes sans aide verbale.
Noter durée, réussite et hésitations. Au moins quatre doivent terminer en moins de deux minutes.
Conserver les résultats anonymisés dans `evidence/usability.md`.

## Reprendre le processus spec-kit

Les commandes disponibles dans Codex sont des skills, pas des exécutables PowerShell.
Utiliser `$speckit-implement` pour commencer les tâches après la conception.
Le CLI local est `.tools/bin/specify.exe`. Sa version se vérifie avec la commande suivante.

```powershell
.\.tools\bin\specify.exe version
.\.specify\scripts\powershell\check-prerequisites.ps1 -Json -RequireSpec -RequireTasks -IncludeTasks
```

Pour reproduire l'installation depuis un nouveau clone, installer `specify-cli` depuis
`git+https://github.com/github/spec-kit.git@v1.1.3` avec uv. Les skills et modèles sont déjà
versionnables dans le dépôt. L'initialisation ne doit pas écraser des modifications existantes.
