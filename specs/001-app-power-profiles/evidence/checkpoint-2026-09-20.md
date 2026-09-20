# Reprise après la pause du 20 septembre 2026

L'utilisateur a demandé d'arrêter le travail, de finaliser les changements en cours et
de publier le projet sur GitHub en privé. L'objectif « Travailler jusqu'à obtenir la v1
fonctionnelle » est en pause. Aucune campagne ne doit continuer automatiquement.
Le dépôt est https://github.com/Thanaen/PowerPlanSync, branche `main`.

## État livré

Le binaire principal relie le moteur Rust, les API Windows, le stockage durable et
l'interface française Slint 1.18.0. Les règles utilisent le chemin complet, comptent
les instances et choisissent la première règle active. Le retour est explicite et
réservé au début du cycle. Une modification Windows observée reste prioritaire.
Le mode démo est isolé. La fenêtre peut être fermée en laissant la notification active.

Les changements en cours sont intégrés, dont la confirmation du plan proposé,
le focus de l'éditeur, l'état toujours visible et son annonce par le Narrateur,
les couleurs des textes en contraste élevé et les notices Slint.
Le guide est dans `docs/guide-utilisateur.md`. Le script `scripts/package-portable.ps1`
assemble un paquet local et son manifeste. Aucune version publique n'est annoncée.

## Résultats vérifiés

- 66 tests passent, dont 37 tests de bibliothèque, 6 scénarios moteur, 17 de récupération
  et 6 de stockage. Formatage et Clippy sur toutes les cibles passent.
- 100 cycles réels avec le témoin de dix secondes passent. Activation p95 1 007,464 ms,
  retour p95 4 056,666 ms. Le rapport et le hash du binaire testé sont dans
  `evidence/windows-cycles/`.
- Les six scénarios réels passent dans `evidence/behaviors/20260920-220229-c86b8ec6/`.
  Ils couvrent deux instances, un homonyme ailleurs, un lanceur encore actif, l'arrêt
  forcé du témoin, le choix manuel conservé et le retour gelé récupéré après crash.
- Le parcours natif d'une démo neuve passe dans `evidence/verification/v1-20260920-215730/`.
  La configuration, le cycle, le retour, À propos et Quitter sont exercés. Le Narrateur
  annonce réellement le plan et le logiciel responsable. Les captures sont conservées.
- Le sélecteur de fichiers natif et le focus clavier ont été exercés dans les séances
  précédentes. Les preuves se trouvent dans `evidence/accessibility/`.

Le binaire produit testé le plus récent a pour SHA-256
`43644EC204BEE65D4207A92071B3EF46765E315147291DE2AF0A610091547E8E`.
Les corrections ultérieures concernent les scripts et la sonde de fenêtres, pas ce binaire.

## Mesures interrompues et défaut de sonde

La mesure longue a été arrêtée à la demande de pause, après 155 secondes mesurées et
60 secondes de stabilisation. Vingt règles, 216 à 222 processus Windows, CPU moyen
0,0277 % de la machine et mémoire privée maximale 3,09 Mio ont été observés.
Ce résultat partiel ne valide pas SC-004. Le rapport porte `complete: false` dans
`evidence/performance/v1-20260920-220700/`. L'arrêt du PID explique son message d'échec.

La sonde de fenêtres a terminé les cent cycles avec une dérive observée de 1,62 Mio
et deux handles, puis a échoué lors de son arrêt. Le callback conservait fortement
son propre timer et l'interface jusqu'à la destruction du contexte Slint.
La trace est dans `evidence/window-lifecycle/20260920-220806-6744ffd4/result.json`.
Le correctif intégré remplace cette rétention par une référence faible et détruit
explicitement les objets avant la fin du contexte. Il compile, mais **n'a pas été
réexécuté sur le bureau après correction**. Les résultats précédents restent en échec.
Ce cycle de références concerne la sonde, pas le chemin d'arrêt du binaire principal.

## Première action à la reprise

Relire `tasks.md`, ce checkpoint et le skill `.agents/skills/verify/SKILL.md`.
Puis compiler et réexécuter uniquement la sonde corrigée avant une campagne longue.

```powershell
cargo build --locked --release --example window_lifecycle
.\scripts\verify-window-lifecycle.ps1
```

Sur ce poste, l'installation Rust est locale au dépôt. Définir `CARGO_HOME=.tools/cargo`,
`RUSTUP_HOME=.tools/rustup` et ajouter `.tools/cargo/bin` au PATH de la commande.
Le binaire de la sonde dans `target/release/examples/` doit être reconstruit ; il peut
encore correspondre à la version qui échouait. Aucun essai ne doit lancer Battlefield 6.

## Travail restant

- Terminer les mesures du protocole de `quickstart.md`, trente minutes par état avec
  vingt règles, puis le diagnostic de rendu masqué. Ne pas lancer de compilation pendant
  les mesures ni deux campagnes qui écrivent des plans Windows simultanément.
- Vérifier toute la matrice clavier, Narrateur, clair, sombre, contraste élevé,
  DPI 100/150/200 %, texte agrandi et petites fenêtres. Le correctif de couleurs est
  compilé mais le contraste élevé réel n'a pas été vérifié visuellement.
- Vérifier les contrôles Slint standards en contraste élevé. Ils suivent une palette
  Fluent claire/sombre ; seules les couleurs applicatives suivent exactement Windows.
- Terminer veille, batterie, sessions, démarrage à la connexion, plans manquants,
  transitions de priorité et réouverture depuis la zone de notification selon le guide.
- Faire le parcours avec cinq personnes non spécialistes et compléter Windows 11 24H2.
  La machine testée est Windows 11 25H2, build 26200, Ryzen 7 9800X3D, 32 Go, 16 threads.
- Finaliser les rapports manquants, la checklist de livraison, `speckit-analyze` et
  `speckit-converge`. Résoudre la licence du projet avant une publication publique.

Une remarque d'audit sur les PID inaccessibles reste à qualifier. Le moteur conserve
au plus dix secondes une instance reconnue devenue incertaine, conformément à FR-023.
Sans accès à sa date de création, la distinction avec un PID réutilisé est indémontrée.
Ce point n'est pas un défaut reproduit et ne justifie pas seul de supprimer cette grâce.

## État du poste

Les témoins, les démos et la mesure ont été arrêtés. Le Narrateur lancé pour les essais
a été arrêté. Aucun démarrage automatique ni automatisation récurrente n'a été activé.
Le dernier plan Windows vérifié est Haute performance,
`8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c`, identique au plan initial des essais.

Les outils locaux, les copies de travail, les données temporaires et les exécutables
reconstructibles sont exclus de Git. Les sources, la conception spec-kit, les scripts,
les captures et les rapports JSON sont conservés. Les branches de travail sous `.audit`
restent locales ; `main` contient les changements retenus.
