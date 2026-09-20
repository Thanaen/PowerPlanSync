---
description: "Tâches de réalisation de PowerPlanSync issues de la conception spec-kit"
---

# Tasks: Associer un plan d'alimentation à un logiciel

**Input**: documents de `specs/001-app-power-profiles/`.

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md` et `contracts/`.

**Tests**: exigés par FR-026 et la constitution pour les décisions sensibles.
Les tests de comportement précèdent la logique correspondante. Les contrôles UI et ressources
portent sur Windows réel, pas uniquement sur les tests automatisés.

**Organization**: fondations communes puis parcours indépendamment démontrables, dans l'ordre P1 puis P2.
Aucune case de réalisation n'est cochée pendant la conception.

## Format: `[ID] [P?] [Story] Description`

Chaque tâche porte un identifiant, un chemin cible et les exigences couvertes.
`[P]` indique des fichiers disjoints après les prérequis précisés plus bas.
Les mesures et essais qui changent le plan Windows ne s'exécutent jamais simultanément.

## Path Conventions

Les chemins sont relatifs à la racine du projet. `src/`, `ui/`, `tests/`, `examples/` et
`scripts/` seront créés pendant l'implémentation. Les preuves restent sous le dossier de cette spec.

## Phase 1: Setup et faisabilité

**Purpose**: vérifier les contraintes qui peuvent invalider le choix technique avant le produit complet.

- [x] T001 Vérifier Rust MSVC et le SDK Windows, épingler outils et dépendances stables compatibles dans `rust-toolchain.toml`, `Cargo.toml` et `Cargo.lock`, créer `src/main.rs`, `src/lib.rs` et `build.rs`, et consigner les versions dans `specs/001-app-power-profiles/evidence/toolchain.md`. Couvre FR-026.
- [x] T002 Construire `examples/windows_probe.rs` pour énumérer les plans, lire les noms, tester une bascule explicite avec restauration finale, lire les chemins et vérifier le verrou global sous compte standard. Mesurer aussi sortie réelle, observation, grâce et confirmation Windows avec sorties réparties entre les observations. Consigner les résultats dans `specs/001-app-power-profiles/evidence/windows-probe.md`. Couvre FR-001, FR-004, FR-010, FR-016, SC-002.
- [x] T003 [P] Construire `examples/ui_probe.rs` avec les contrôles Slint standards, une notification et une fenêtre créée puis détruite à la demande. Documenter la version fournissant les API choisies et les obligations de distribution dans `specs/001-app-power-profiles/evidence/ui-probe.md`. Couvre FR-019, SC-007.
- [ ] T004 Vérifier le prototype avec clavier, Narrateur, arbre accessible, thèmes et échelles 100 à 200 %, puis enregistrer les premiers coûts CPU et mémoire dans `specs/001-app-power-profiles/evidence/prototype-results.md`. Couvre FR-019, SC-004, SC-005, SC-007.
- [x] T005 Créer `examples/demo_target.rs` et vérifier depuis `examples/windows_probe.rs` son chemin, ses instances multiples, un homonyme ailleurs et un lanceur témoin restant ouvert. Enregistrer les preuves dans `specs/001-app-power-profiles/evidence/fixture-feasibility.md`. Ne pas lancer Battlefield 6. Couvre FR-004, FR-023, SC-006.
- [ ] T006 Conclure les essais dans `specs/001-app-power-profiles/research.md` et ajuster `specs/001-app-power-profiles/plan.md` avant la suite. Comparer egui ou Win32 si Slint échoue. La compatibilité Battlefield reste non annoncée, sans bloquer la validation générique demandée. Couvre FR-026, SC-004, SC-005, SC-006.

**Checkpoint**: les choix Windows et UI sont prouvés ou corrigés. Les résultats inconnus restent marqués.

## Phase 2: Foundational

**Purpose**: établir les invariants et les adaptateurs communs.

- [ ] T007 Définir les identités, états, observations et événements de `data-model.md` dans `src/domain.rs`, y compris `automation_enabled` distinct de `paused`. Couvre FR-002, FR-004, FR-005, FR-006, FR-024, FR-025.
- [ ] T008 [P] Écrire les scénarios de fichier interrompu, invalide, version future et erreur de journal dans `tests/storage_failures.rs`. Couvre FR-011, FR-017, FR-026.
- [ ] T009 Implémenter stockage versionné, remplacement durable, sauvegarde et journal récupérable dans `src/storage.rs`. Couvre FR-011, FR-017.
- [ ] T010 [P] Implémenter le catalogue et les appels Win32 bornés dans `src/windows/mod.rs` et `src/windows/power.rs`, avec gestion des allocations et codes d'erreur. Couvre FR-001, FR-010, FR-013, FR-022.
- [ ] T011 [P] Implémenter snapshots cohérents, filtrage de session, chemins Windows normalisés, instances et erreurs d'accès dans `src/windows/processes.rs`. Couvre FR-004, FR-005, FR-023.
- [ ] T012 [P] Implémenter le contrôle unique global et l'ouverture de fenêtre de la même session dans `src/windows/session.rs`, avec refus sûr si le verrou est inaccessible. Couvre FR-016.
- [ ] T013 Intégrer le démarrage, le mode `--demo`, le dossier isolé, l'exécutable témoin et les diagnostics limités à 5 Mio dans `src/main.rs`, `src/demo.rs`, `src/diagnostics.rs` et `examples/demo_target.rs`. Couvre FR-020, FR-021, FR-026.

**Checkpoint**: le mode demo ne modifie pas Windows. Les défauts d'adaptateur sont injectables.

## Phase 3: User Story 1 - Automatiser une session de jeu (Priority: P1)

**Goal**: prouver un cycle logiciel, plan cible et retour explicite depuis une interface minimale.

**Independent Test**: deux instances de `demo_target`, un homonyme ailleurs, et un retour différent du plan initial.

- [ ] T014 [US1] Écrire les scénarios de début, dernière fermeture, réapparition, PID réutilisé et cible déjà active dans `tests/engine_scenarios.rs`. Couvre FR-004, FR-005, FR-007, FR-008, SC-003.
- [ ] T015 [US1] Implémenter le décideur pur et la grâce monotone de trois secondes dans `src/engine.rs`. Couvre FR-005, FR-007, FR-008, FR-024.
- [ ] T016 [US1] Implémenter dans `src/coordinator.rs` la boucle d'une seconde, l'acquisition journalisée, une seule transaction en vol, la relecture avant et après écriture et l'arrêt du timer quand inutile. Couvre FR-008, FR-010, FR-011, FR-025, SC-002, SC-004.
- [ ] T017 [P] [US1] Créer l'éditeur minimal de fichier et plan dans `ui/rule-editor.slint`, avec validation et annulation. Couvre FR-002, FR-003, FR-025.
- [ ] T018 [US1] Relier retour confirmé, activation explicite, état réel, fenêtre et notification dans `src/ui.rs`, `ui/main.slint` et `ui/tray.slint`. Couvre FR-002, FR-008, FR-018, FR-024, FR-025.
- [ ] T019 [US1] Vérifier le cycle sur Windows avec le témoin, le nom homonyme et plusieurs instances dans `tests/windows_integration.rs`, mesurer les délais depuis création et sortie réelles jusqu'au GUID confirmé, puis conserver les résultats dans `specs/001-app-power-profiles/evidence/us1.md`. Couvre FR-004, FR-005, FR-008, FR-026, SC-002, SC-003.

**Checkpoint**: démonstration du besoin principal. Cette étape seule n'est pas une version distribuable.

## Phase 4: User Story 2 - Garder la maîtrise de Windows (Priority: P1)

**Goal**: respecter les modifications externes et récupérer les interruptions sans boucle de bascule.

**Independent Test**: modifier le plan hors application en jeu, fermer le jeu puis relancer le suivi.

- [ ] T020 [US2] Écrire les tests de révocation externe, Reprendre, pause persistante et coupures de chaque phase du journal dans `tests/recovery_scenarios.rs`. Couvre FR-009, FR-010, FR-011, FR-012, FR-026, SC-003.
- [ ] T021 [US2] Implémenter récupération et notifications de plan dans `src/coordinator.rs` et `src/windows/power.rs`, y compris le journal préparé sans confirmation. Couvre FR-009, FR-010, FR-011.
- [ ] T022 [US2] Implémenter Pause, Reprendre, fermeture de fenêtre et sortie avec retour protégé dans `src/coordinator.rs` et `src/ui.rs`. Couvre FR-009, FR-012.
- [ ] T023 [US2] Implémenter états bloqués, disparition de plan, observations incertaines dix secondes et reprise explicite dans `src/engine.rs` et `src/coordinator.rs`. Couvre FR-013, FR-023.
- [ ] T024 [US2] Intégrer source secteur, batterie et reprise de veille dans `src/windows/power.rs` et `src/coordinator.rs`. Couvre FR-014, FR-015.
- [ ] T025 [US2] Intégrer déconnexion, changement de session et arrêt de Windows dans `src/windows/session.rs` et `src/coordinator.rs`, en distinguant verrouillage d'écran et perte de session active. Couvre FR-015, FR-016.
- [ ] T026 [US2] Présenter erreurs, choix de sortie malgré échec et explication des limites de récupération dans `ui/main.slint`, `ui/tray.slint` et `ui/strings-fr.slint`. Couvre FR-009, FR-012, FR-013, FR-018.
- [ ] T027 [US2] Exécuter la matrice de récupération, batterie, veille, sessions et erreurs du guide, puis consigner chaque résultat dans `specs/001-app-power-profiles/evidence/us2.md`. Couvre FR-009, FR-010, FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-023, FR-026, SC-003.

**Checkpoint**: les erreurs restent visibles et les écritures respectent la révocation observée.

## Phase 5: User Story 4 - Configurer et consulter sans expertise (Priority: P1)

**Goal**: rendre le parcours principal utilisable sans souris ni vocabulaire d'administration.

**Independent Test**: parcourir toute la configuration et les actions globales au clavier puis au Narrateur.

- [ ] T028 [US4] Construire l'état vide, le premier parcours et les libellés français dans `ui/main.slint`, `ui/rule-editor.slint` et `ui/strings-fr.slint`. Couvre FR-002, FR-018, FR-025, SC-001.
- [ ] T029 [US4] Ajouter le sélecteur d'application ouverte avec chemin vérifiable dans `src/ui.rs` et `ui/rule-editor.slint`. Couvre FR-003, FR-004, FR-023.
- [ ] T030 [US4] Implémenter noms accessibles, rôles, états, focus, raccourcis et annonces de statut dans `ui/main.slint`, `ui/rule-editor.slint` et `ui/tray.slint`. Couvre FR-019, SC-005.
- [ ] T031 [US4] Adapter thèmes Windows, contraste, mise à l'échelle, texte agrandi et défilement dans `ui/main.slint` et `ui/rule-editor.slint`. Couvre FR-019, SC-005.
- [ ] T032 [US4] Finaliser le cycle créer, fermer et rouvrir la fenêtre depuis la notification dans `src/ui.rs` et `ui/tray.slint`, avec explication du premier masquage. Couvre FR-012, FR-018, SC-007.
- [ ] T033 [US4] Ajouter le démarrage facultatif à la connexion et son retrait exact dans `src/windows/startup.rs`, `src/windows/mod.rs`, `src/ui.rs` et `ui/main.slint`. Couvre FR-020.
- [ ] T034 [US4] Vérifier clavier, Narrateur, contrastes, DPI, texte agrandi et activation puis désactivation du démarrage, et enregistrer captures et arbre accessible dans `specs/001-app-power-profiles/evidence/accessibility/`. Couvre FR-019, FR-020, FR-026, SC-005.

**Checkpoint**: le parcours principal est accessible sur la machine cible, sans se limiter à une capture visuelle.

## Phase 6: User Story 3 - Gérer plusieurs logiciels (Priority: P2)

**Goal**: rendre la priorité et les modifications de règles prévisibles.

**Independent Test**: deux logiciels avec des plans différents, ordre inversé puis fin du premier.

- [ ] T035 [US3] Ajouter les scénarios de priorité, réordonnancement, désactivation et suppression active dans `tests/engine_scenarios.rs`. Couvre FR-003, FR-006, FR-007, FR-024.
- [ ] T036 [US3] Compléter l'arbitrage ordonné et le recalcul immédiat des actions explicites dans `src/engine.rs`. Couvre FR-003, FR-006, FR-007.
- [ ] T037 [US3] Ajouter activation, modification, suppression, Monter et Descendre dans `ui/main.slint` et `src/ui.rs`, sans exiger de glisser-déposer. Couvre FR-003, FR-006, FR-019.
- [ ] T038 [US3] Vérifier les transitions entre gagnants sans plan intermédiaire et les logiciels déplacés, puis conserver les preuves dans `specs/001-app-power-profiles/evidence/us3.md`. Couvre FR-003, FR-004, FR-006, FR-007, SC-003.

**Checkpoint**: toutes les règles sont testables séparément et ensemble.

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: produire les preuves qui autorisent une première livraison.

- [ ] T039 [P] Créer `scripts/measure-overhead.ps1`, exécuter le protocole CPU, mémoire et cent cycles de fenêtre, puis conserver données brutes et traces dans `specs/001-app-power-profiles/evidence/performance/`. Couvre FR-026, SC-004, SC-007.
- [ ] T040 Créer `scripts/verify-windows.ps1` avec sauvegarde et restauration finale du plan de test, mesurer 100 cycles depuis création et sortie réelles avec offsets variés entre observations, puis reprendre tous les cas du guide dans `specs/001-app-power-profiles/evidence/validation/`. Couvre FR-010, FR-026, SC-002, SC-003.
- [ ] T041 Vérifier le cycle de l'exécutable témoin avec son lanceur restant ouvert, plusieurs instances et un arrêt forcé, puis enregistrer activation, maintien, retour et coût dans `specs/001-app-power-profiles/evidence/fixture-validation.md`. Aucun lancement de Battlefield 6. Couvre FR-004, FR-022, FR-026, SC-006.
- [ ] T042 Faire essayer le parcours initial à cinq personnes non spécialistes et consigner temps et réussite anonymisés dans `specs/001-app-power-profiles/evidence/usability.md`. Couvre FR-018, SC-001.
- [ ] T043 Auditer droits demandés, réseau absent, données collectées et rotation des diagnostics dans `src/windows/`, `src/diagnostics.rs` et `specs/001-app-power-profiles/evidence/privacy.md`. Couvre FR-020, FR-021, FR-022.
- [ ] T044 Préparer le binaire release portable, les notices requises et le guide d'installation et retrait dans `README.md` et `THIRD_PARTY_NOTICES.md`, en résolvant la licence de distribution avant publication. Couvre FR-020, FR-022.
- [ ] T045 Exécuter les contrôles Cargo et le guide complet sur les Windows cibles, mettre à jour `specs/001-app-power-profiles/checklists/release.md`, puis relancer `speckit-analyze` et `speckit-converge`. Couvre FR-026 et SC-001 à SC-007.

## Dependencies & Execution Order

### Phase Dependencies

T001 précède tous les prototypes. T002 et T003 peuvent avancer sur des fichiers distincts après
épinglage commun des dépendances. T004 dépend de T003. T005 dépend de T002. T006 examine leurs preuves.
Les preuves Windows de T002 et T005 autorisent T007 à T013 et le moteur T014 à T016.
T004 et T006 précèdent toujours le choix de l'interface définitive et restent nécessaires
à la livraison. Les tests T020 et la récupération T021 accompagnent la transaction de T016,
avant son intégration UI. T008 précède T009. T010 à T012 partagent les types de T007, mais leurs
fichiers sont disjoints. T013 les assemble. Toute modification de Cargo reste sérialisée.

### User Story Dependencies

US1 suit les fondations. US2 durcit le cycle US1 et le valide indépendamment avec erreurs injectées.
US4 finalise l'interface déjà minimale de US1. US3 vient ensuite car P2 et partage le moteur et l'UI.
Les phases ne sont pas parallélisées sur les mêmes fichiers. T039 à T045 suivent les parcours.
Les mesures T039 et essais T040/T041 utilisent le même PC et restent sérialisés.

### Within Each User Story

Les scénarios de décision sont écrits avant le comportement correspondant et constatés en échec
pour la bonne raison. Les vérifications graphiques portent sur le binaire optimisé. Une correction
de comportement met à jour la spec avant le code si elle change le contrat.

### Parallel Opportunities

L'annotation `[P]` signifie une possibilité d'écriture disjointe, pas une autorisation de tests
Windows simultanés. Après T001, T002 et T003 sont séparables. Après T007, T008, T010, T011 et T012
peuvent avancer séparément. Un responsable unique assemble ces fichiers et exécute les essais.

## Parallel Example: User Story 1

T017 peut préparer le dialogue à partir du contrat pendant T015 et T016.
T018 attend les trois. US2 reste séquentielle car moteur, coordinateur et intégration se recouvrent.
Pour US4, une revue d'accessibilité en lecture seule peut accompagner T033. Pour US3, la collecte
des scénarios de démonstration peut accompagner T037, puis T038 attend l'intégration.

## Implementation Strategy

Le premier livrable de développement est la preuve de faisabilité, puis la démonstration US1.
Une version utilisable au quotidien exige toutes les protections US2, l'accessibilité US4 et
les validations finales. Une case n'est cochée qu'avec son fichier et sa preuve disponibles.
Un résultat inaccessible ou non exécuté demeure un échec de validation pour la promesse concernée.

Ne pas publier le binaire, créer une tâche persistante ni changer les réglages personnels au titre
de cette conception. Ces tâches décrivent le futur produit. La livraison sera décidée séparément.

## Notes

Le plan suppose une UI française et Slint comme candidat initial. La première mesure peut changer
ce choix, avec une décision écrite. L'absence de Rust dans l'environnement actuel est traitée par T001.
L'outil ne garantit ni une hausse des FPS, ni une restauration immédiate après sa fermeture forcée.
