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
Les phases 1 à 6 décrivent la version Slint ; leurs cases cochées reflètent ce code et ses preuves.
Les phases 7 à 13 migrent l'interface vers GPUI (D12 à D16 de `research.md`). La phase 14 valide
la livraison sur l'interface GPUI.

## Format: `[ID] [P?] [Story] Description`

Chaque tâche porte un identifiant, un chemin cible et les exigences couvertes.
`[P]` indique des fichiers disjoints après les prérequis précisés plus bas.
Les mesures et essais qui changent le plan Windows ne s'exécutent jamais simultanément.

Une tâche remplacée garde son identifiant et sa case d'origine. Sa description est barrée et
suivie de « → remplacée par » et des tâches GPUI qui la reprennent. Une case cochée remplacée
reste la preuve historique de l'interface Slint. Une case vide remplacée ne s'exécute plus.

## Path Conventions

Les chemins sont relatifs à la racine du projet. La couche GPUI vit dans `src/interface/`, ses
ressources dans `assets/`. `ui/` et `src/ui.rs` disparaissent en T071. Les preuves restent sous le
dossier de cette spec.

## Phase 1: Setup et faisabilité

**Purpose**: vérifier les contraintes qui peuvent invalider le choix technique avant le produit complet.

- [x] T001 Vérifier Rust MSVC et le SDK Windows, épingler outils et dépendances stables compatibles dans `rust-toolchain.toml`, `Cargo.toml` et `Cargo.lock`, créer `src/main.rs`, `src/lib.rs` et `build.rs`, et consigner les versions dans `specs/001-app-power-profiles/evidence/toolchain.md`. Couvre FR-026.
- [x] T002 Construire `examples/windows_probe.rs` pour énumérer les plans, lire les noms, tester une bascule explicite avec restauration finale, lire les chemins et vérifier le verrou global sous compte standard. Mesurer aussi sortie réelle, observation, grâce et confirmation Windows avec sorties réparties entre les observations. Consigner les résultats dans `specs/001-app-power-profiles/evidence/windows-probe.md`. Couvre FR-001, FR-004, FR-010, FR-016, SC-002.
- [x] T003 [P] ~~Construire `examples/ui_probe.rs` avec les contrôles Slint standards, une notification et une fenêtre créée puis détruite à la demande. Documenter la version fournissant les API choisies et les obligations de distribution dans `specs/001-app-power-profiles/evidence/ui-probe.md`. Couvre FR-019, SC-007.~~ → remplacée par T047.
- [ ] T004 ~~Vérifier le prototype avec clavier, Narrateur, arbre accessible, thèmes et échelles 100 à 200 %, puis enregistrer les premiers coûts CPU et mémoire dans `specs/001-app-power-profiles/evidence/prototype-results.md`. Couvre FR-019, SC-004, SC-005, SC-007.~~ → remplacée par T048.
- [x] T005 Créer `examples/demo_target.rs` et vérifier depuis `examples/windows_probe.rs` son chemin, ses instances multiples, un homonyme ailleurs et un lanceur témoin restant ouvert. Enregistrer les preuves dans `specs/001-app-power-profiles/evidence/fixture-feasibility.md`. Ne pas lancer Battlefield 6. Couvre FR-004, FR-023, SC-006.
- [ ] T006 ~~Conclure les essais dans `specs/001-app-power-profiles/research.md` et ajuster `specs/001-app-power-profiles/plan.md` avant la suite. Comparer egui ou Win32 si Slint échoue. La compatibilité Battlefield reste non annoncée, sans bloquer la validation générique demandée. Couvre FR-026, SC-004, SC-005, SC-006.~~ → remplacée par T049.

**Checkpoint**: les choix Windows sont prouvés. Le choix d'interface est repris en Phase 7.
Les résultats inconnus restent marqués.

## Phase 2: Foundational

**Purpose**: établir les invariants et les adaptateurs communs.

- [x] T007 Définir les identités, états, observations et événements de `data-model.md` dans `src/domain.rs`, y compris `automation_enabled` distinct de `paused`. Couvre FR-002, FR-004, FR-005, FR-006, FR-024, FR-025.
- [x] T008 [P] Écrire les scénarios de fichier interrompu, invalide, version future et erreur de journal dans `tests/storage_failures.rs`. Couvre FR-011, FR-017, FR-026.
- [x] T009 Implémenter stockage versionné, remplacement durable, sauvegarde et journal récupérable dans `src/storage.rs`. Couvre FR-011, FR-017.
- [x] T010 [P] Implémenter le catalogue et les appels Win32 bornés dans `src/windows/mod.rs` et `src/windows/power.rs`, avec gestion des allocations et codes d'erreur. Couvre FR-001, FR-010, FR-013, FR-022.
- [x] T011 [P] Implémenter snapshots cohérents, filtrage de session, chemins Windows normalisés, instances et erreurs d'accès dans `src/windows/processes.rs`. Couvre FR-004, FR-005, FR-023.
- [ ] T012 [P] Implémenter le contrôle unique global et l'ouverture de fenêtre de la même session dans `src/windows/session.rs`, avec refus sûr si le verrou est inaccessible. Couvre FR-016.
- [x] T013 Intégrer le démarrage, le mode `--demo`, le dossier isolé, l'exécutable témoin et les diagnostics limités à 5 Mio dans `src/main.rs`, `src/runtime_power.rs`, `src/diagnostics.rs` et `examples/demo_target.rs`. Couvre FR-020, FR-021, FR-026.

**Checkpoint**: le mode demo ne modifie pas Windows. Les défauts d'adaptateur sont injectables.

## Phase 3: User Story 1 - Automatiser une session de jeu (Priority: P1)

**Goal**: prouver un cycle logiciel, plan cible et retour explicite depuis une interface minimale.

**Independent Test**: deux instances de `demo_target`, un homonyme ailleurs, et un retour différent du plan initial.

- [x] T014 [US1] Écrire les scénarios de début, dernière fermeture, réapparition, PID réutilisé et cible déjà active dans `tests/engine_scenarios.rs`. Couvre FR-004, FR-005, FR-007, FR-008, SC-003.
- [x] T015 [US1] Implémenter le décideur pur et la grâce monotone de trois secondes dans `src/engine.rs`. Couvre FR-005, FR-007, FR-008, FR-024.
- [x] T016 [US1] Implémenter dans `src/runtime.rs` et `src/coordinator.rs` la boucle d'une seconde, l'acquisition journalisée, une seule transaction en vol, la relecture avant et après écriture et l'arrêt du timer quand inutile. Couvre FR-008, FR-010, FR-011, FR-025, SC-002, SC-004.
- [x] T017 [P] [US1] ~~Créer l'éditeur minimal de fichier et plan dans `ui/rule-editor.slint`, avec validation et annulation. Couvre FR-002, FR-003, FR-025.~~ → remplacée par T059.
- [x] T018 [US1] ~~Relier retour confirmé, activation explicite, état réel, fenêtre et notification dans `src/ui.rs`, `ui/main.slint` et `ui/tray.slint`. Couvre FR-002, FR-008, FR-018, FR-024, FR-025.~~ → remplacée par T054, T055, T056, T058 et T060.
- [ ] T019 [US1] Vérifier le cycle sur Windows avec le témoin, le nom homonyme et plusieurs instances dans `tests/windows_integration.rs`, mesurer les délais depuis création et sortie réelles jusqu'au GUID confirmé, puis conserver les résultats dans `specs/001-app-power-profiles/evidence/us1.md`. Couvre FR-004, FR-005, FR-008, FR-026, SC-002, SC-003.

**Checkpoint**: démonstration du besoin principal. Cette étape seule n'est pas une version distribuable.

## Phase 4: User Story 2 - Garder la maîtrise de Windows (Priority: P1)

**Goal**: respecter les modifications externes et récupérer les interruptions sans boucle de bascule.

**Independent Test**: modifier le plan hors application en jeu, fermer le jeu puis relancer le suivi.

- [x] T020 [US2] Écrire les tests de révocation externe, Reprendre, pause persistante et coupures de chaque phase du journal dans `tests/recovery_scenarios.rs`. Couvre FR-009, FR-010, FR-011, FR-012, FR-026, SC-003.
- [x] T021 [US2] Implémenter récupération et notifications de plan dans `src/coordinator.rs` et `src/windows/power.rs`, y compris le journal préparé sans confirmation. Couvre FR-009, FR-010, FR-011.
- [x] T022 [US2] Implémenter Pause, Reprendre, fermeture de fenêtre et sortie avec retour protégé dans `src/coordinator.rs` et `src/ui.rs`. Couvre FR-009, FR-012. Le coordinateur reste valable ; la partie `src/ui.rs` est reprise par T063.
- [x] T023 [US2] Implémenter états bloqués, disparition de plan, observations incertaines dix secondes et reprise explicite dans `src/engine.rs` et `src/coordinator.rs`. Couvre FR-013, FR-023.
- [x] T024 [US2] Intégrer source secteur, batterie et reprise de veille dans `src/windows/power.rs` et `src/coordinator.rs`. Couvre FR-014, FR-015.
- [x] T025 [US2] Intégrer déconnexion, changement de session et arrêt de Windows dans `src/windows/session.rs` et `src/coordinator.rs`, en distinguant verrouillage d'écran et perte de session active. Couvre FR-015, FR-016.
- [x] T026 [US2] ~~Présenter erreurs, choix de sortie malgré échec et explication des limites de récupération dans `ui/main.slint`, `ui/tray.slint` et `ui/strings-fr.slint`. Couvre FR-009, FR-012, FR-013, FR-018.~~ → remplacée par T062.
- [ ] T027 [US2] Exécuter la matrice de récupération, batterie, veille, sessions et erreurs du guide, puis consigner chaque résultat dans `specs/001-app-power-profiles/evidence/us2.md`. Couvre FR-009, FR-010, FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-023, FR-026, SC-003.

**Checkpoint**: les erreurs restent visibles et les écritures respectent la révocation observée.

## Phase 5: User Story 4 - Configurer et consulter sans expertise (Priority: P1)

**Goal**: rendre le parcours principal utilisable sans souris ni vocabulaire d'administration.

**Independent Test**: parcourir toute la configuration et les actions globales au clavier puis au Narrateur.

- [x] T028 [US4] ~~Construire l'état vide, le premier parcours et les libellés français dans `ui/main.slint`, `ui/rule-editor.slint` et `ui/strings-fr.slint`. Couvre FR-002, FR-018, FR-025, SC-001.~~ → remplacée par T060.
- [x] T029 [US4] ~~Ajouter le sélecteur d'application ouverte avec chemin vérifiable dans `src/ui.rs` et `ui/rule-editor.slint`. Couvre FR-003, FR-004, FR-023.~~ → remplacée par T059.
- [x] T030 [US4] ~~Implémenter noms accessibles, rôles, états, focus, raccourcis et annonces de statut dans `ui/main.slint`, `ui/rule-editor.slint` et `ui/tray.slint`. Couvre FR-019, SC-005.~~ → remplacée par T067.
- [ ] T031 [US4] ~~Adapter thèmes Windows, contraste, mise à l'échelle, texte agrandi et défilement dans `ui/main.slint` et `ui/rule-editor.slint`. Couvre FR-019, SC-005.~~ → remplacée par T068.
- [ ] T032 [US4] ~~Finaliser le cycle créer, fermer et rouvrir la fenêtre depuis la notification dans `src/ui.rs` et `ui/tray.slint`, avec explication du premier masquage. Couvre FR-012, FR-018, SC-007.~~ → remplacée par T054 et T055.
- [x] T033 [US4] Ajouter le démarrage facultatif à la connexion et son retrait exact dans `src/windows/startup.rs`, `src/windows/mod.rs`, `src/ui.rs` et `ui/main.slint`. Couvre FR-020. Les fichiers `src/windows/` restent valables ; l'interrupteur est repris par T065.
- [ ] T034 [US4] ~~Vérifier clavier, Narrateur, contrastes, DPI, texte agrandi et activation puis désactivation du démarrage, et enregistrer captures et arbre accessible dans `specs/001-app-power-profiles/evidence/accessibility/`. Couvre FR-019, FR-020, FR-026, SC-005.~~ → remplacée par T070.

**Checkpoint**: le parcours principal est accessible sur la machine cible, sans se limiter à une capture visuelle.

## Phase 6: User Story 3 - Gérer plusieurs logiciels (Priority: P2)

**Goal**: rendre la priorité et les modifications de règles prévisibles.

**Independent Test**: deux logiciels avec des plans différents, ordre inversé puis fin du premier.

- [x] T035 [US3] Ajouter les scénarios de priorité, réordonnancement, désactivation et suppression active dans `tests/engine_scenarios.rs`. Couvre FR-003, FR-006, FR-007, FR-024.
- [x] T036 [US3] Compléter l'arbitrage ordonné et le recalcul immédiat des actions explicites dans `src/engine.rs`. Couvre FR-003, FR-006, FR-007.
- [x] T037 [US3] ~~Ajouter activation, modification, suppression, Monter et Descendre dans `ui/main.slint` et `src/ui.rs`, sans exiger de glisser-déposer. Couvre FR-003, FR-006, FR-019.~~ → remplacée par T064.
- [ ] T038 [US3] Vérifier les transitions entre gagnants sans plan intermédiaire et les logiciels déplacés, puis conserver les preuves dans `specs/001-app-power-profiles/evidence/us3.md`. Couvre FR-003, FR-004, FR-006, FR-007, SC-003.

**Checkpoint**: toutes les règles sont testables séparément et ensemble.

## Phase 7: Faisabilité GPUI

**Purpose**: prouver GPUI sur Windows avant l'interface définitive, comme la constitution l'exige.
Voir D12 à D16 dans `research.md`. Ces tâches remplacent T003, T004 et T006 pour l'interface.

- [x] T046 Épingler dans `Cargo.toml` et `Cargo.lock` `gpui` `=0.3.9` (paquet `gpui-pre`), `gpui_platform` `=0.3.9` (paquet `gpui-pre-platform`), `tray-icon` 0.26 sans fonctionnalités par défaut, `image` 0.25 sans fonctionnalités par défaut (`ico`, `png`), `futures` 0.3, `raw-window-handle` 0.6 (`tray-icon`, `futures` et `raw-window-handle` sous `cfg(windows)`) et la fonctionnalité `screenshots` (`gpui/test-support`, `gpui_platform/test-support`, `gpui_wgpu` `=0.3.9` sous Linux), sans retirer Slint. Ajouter `components = ["rustfmt", "clippy"]` à `rust-toolchain.toml`. Vérifier `cargo fmt --check` et `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings` avec Rust 1.98.1, puis consigner versions et résultats dans `specs/001-app-power-profiles/evidence/toolchain.md`. Couvre FR-026.
- [x] T047 Construire `examples/gpui_probe.rs` : fenêtre à barre de titre dessinée, ouverte par `cx.open_window` et détruite par `remove_window`, bouton, interrupteur, liste et statut avec `Role` et `aria_label`, icône `tray-icon` avec menu `muda`, `cx.set_reduce_motion` relié à `SPI_GETCLIENTAREAANIMATION`, une animation `with_spring`, et sélection d'un `.exe` par `cx.prompt_for_paths` puis par `src/windows/file_dialog.rs`. Consigner les API confirmées, les manques et les licences publiées dans `specs/001-app-power-profiles/evidence/gpui-probe.md`. Couvre FR-019, SC-007.
- [ ] T048 Vérifier la sonde sur Windows réel : clavier, Narrateur, arbre UI Automation, annonce d'un statut sans déplacer le focus, thèmes clair, sombre et contrastés, échelles 100 %, 150 % et 200 %, texte agrandi, changement d'écran et « Effets d'animation » désactivé. Mesurer mémoire privée, handles et CPU fenêtre ouverte, fenêtre fermée et après cent cycles de fenêtre. Ajouter une section GPUI à `specs/001-app-power-profiles/evidence/prototype-results.md` et les captures dans `specs/001-app-power-profiles/evidence/accessibility/gpui/`. Couvre FR-019, SC-004, SC-005, SC-007.
- [ ] T049 Conclure les essais GPUI dans `specs/001-app-power-profiles/research.md` (D12 à D16) et ajuster `specs/001-app-power-profiles/plan.md` et ce fichier : risques levés, contournements retenus, ou proposition explicite d'amendement de la spec si SC-004 ou SC-005 échoue. Couvre FR-026, SC-004, SC-005.

**Checkpoint**: GPUI est prouvé sur Windows, ou la décision revient au responsable produit avec les mesures.

## Phase 8: Fondations GPUI

**Purpose**: poser la couche `src/interface/` sur le coordinateur existant, sans changer le moteur.
L'interface GPUI s'ouvre avec l'option de développement `--gpui` jusqu'à T071.

- [x] T050 Écrire d'abord `tests/presentation.rs`, puis déplacer `UiDisplayModel`, `UiAction` et leurs types de `src/ui.rs` vers `src/presentation.rs` sans type graphique. Aligner statuts, badges, compteur « 2 logiciels suivis sur 3 », source d'alimentation et retour réservé sur `contracts/user-interface.md`, et créer `src/interface/mod.rs` avec la table de libellés `src/interface/strings.rs`. Couvre FR-018, FR-024.
- [ ] T051 [P] Créer `src/interface/theme.rs` avec les jetons Bientôt GPUI (WorkTime One clair et sombre, IBM Plex Sans et Mono dans `assets/fonts/`), les écarts listés dans `design/README.md` « Écarts avec le design system », les couleurs `tray-*` et le remplacement par les couleurs système en contraste élevé, lu par `src/windows/preferences.rs`. Ne pas modifier le design system. Couvre FR-019.
- [ ] T052 [P] Porter `theme::motion` de Bientôt dans `src/interface/motion.rs` (PRESS 140 ms, TOOLTIP 150 ms, MENU 200 ms, DIALOG 250 ms, `ease_out`, `ease_in_out`, `slide_in`, `slide_over`, `Entrance`, `OPAQUE_AT`, `POWERPLANSYNC_MOTION_SLOWDOWN` en debug). Lire « Effets d'animation » par `SPI_GETCLIENTAREAANIMATION` dans `src/windows/preferences.rs` et le relire à chaque `WM_SETTINGCHANGE` reçu par `src/windows/notifications.rs`. Une lecture échouée garde le mouvement. Couvre FR-019.
- [ ] T053 Créer `src/interface/widgets.rs` et `src/interface/text_input.rs` : boutons principal, Tinted et désactivé, interrupteur, badge, liste déroulante de plans, message intégré, touches et anneau de focus de 2 px, chacun avec rôle, nom et état accessibles. Couvre FR-019.
- [ ] T054 Compléter `src/interface/mod.rs` et créer `src/interface/window.rs` : fenêtre créée à la demande et détruite à sa fermeture, barre de titre avec **Commandes**, barre latérale (**Logiciels**, **Réglages**, **Aide et limites**, **Quitter**), barre d'état, réduction aux icônes en fenêtre étroite et défilement. Couvre FR-012, FR-018, FR-019, SC-007.
- [ ] T055 [P] Créer `src/interface/tray.rs` avec `tray-icon` et `muda` : icônes de `assets/tray/` selon l'état, infobulle avec plan et raison, menu (état, **Ouvrir PowerPlanSync**, **Mettre en pause** ou **Reprendre**, **Quitter PowerPlanSync**), clic qui ouvre la fenêtre et notification unique « PowerPlanSync reste ouvert » au premier masquage. Couvre FR-012, FR-018.
- [ ] T056 Brancher l'application GPUI dans `src/runtime.rs`, `src/main.rs` et `src/app_args.rs` derrière `--gpui` : thread principal GPUI, coordinateur sur son thread, `ViewState` reçu par canal, commandes typées vers `Controller`, contrôle unique et ouverture dans la même session par `src/windows/session.rs`, vue « Contrôle indisponible » quand une autre session détient le contrôle (FR-016), options `--demo`, `--data-dir` et `--start-minimized` inchangées. Couvre FR-012, FR-016, FR-018.
- [ ] T057 [P] Ajouter `src/bin/screenshots.rs` (fonctionnalité `screenshots`), qui rend les états de démonstration hors écran sous Linux avec `gpui_wgpu`, en compilant les modules Windows sous `cfg(windows)`. Comparer aux captures de `design/ecrans/` dans `specs/001-app-power-profiles/evidence/gpui-screenshots/`. Si l'isolement coûte trop, consigner l'abandon dans `research.md` D16. Couvre FR-019.

**Checkpoint**: `--gpui` ouvre la fenêtre vide du thème, l'icône et son menu fonctionnent, `--demo` n'écrit aucun plan.

## Phase 9: User Story 1 · interface GPUI (Priority: P1)

**Goal**: refaire en GPUI le premier parcours de la Phase 3.

**Independent Test**: avec `--demo --gpui`, suivre « Pour commencer », ajouter un logiciel, confirmer le retour, activer, simuler un lancement puis une fermeture.

- [ ] T058 [US1] Construire la page **Logiciels** dans `src/interface/software_page.rs` : actions **Ajouter un logiciel** (Ctrl+N) et **Mettre en pause** ou **Reprendre**, **Plan actif dans Windows**, statut avec icône, mot-clé et phrase, ligne du plan de retour, liste **Logiciels associés** avec rang, nom, chemin abrégé au milieu avec le chemin complet en infobulle, plan, interrupteur **Suivre**, badge **Appliqué**, **Ouvert**, **Suspendu** ou **Refusé**, et la phrase sur le premier de la liste. Couvre FR-001, FR-006, FR-018, FR-024.
- [ ] T059 [P] [US1] Construire les dialogues **Ajouter un logiciel** et **Modifier** dans `src/interface/rule_dialog.rs` : **Parcourir…** ou **Application ouverte** avec filtre, fichier complet, **Nom affiché**, **Plan pendant l'exécution**, **Ajouter** ou **Enregistrer** (Entrée), **Annuler** (Échap), processus inaccessible désactivé avec « Fichier inaccessible · impossible à suivre », dialogue invalide qui explique le champ à corriger et conserve la saisie, message sur l'homonyme, plan disparu et « Fichier introuvable ». Couvre FR-002, FR-003, FR-004, FR-023.
- [ ] T060 [P] [US1] Construire « Pour commencer » dans `src/interface/first_launch.rs` : trois étapes, bouton **Confirmer** du plan de retour, aucun plan présélectionné si le plan normal manque, **Activer l'automatisation** indisponible avant les étapes 1 et 2, aucun changement Windows avant l'étape 3. Couvre FR-002, FR-025.

**Checkpoint**: le parcours US1 fonctionne en `--gpui` comme en Slint.

## Phase 10: User Story 2 · interface GPUI (Priority: P1)

**Goal**: rendre visibles en GPUI les protections de la Phase 4.

**Independent Test**: avec `--demo --gpui`, provoquer chaque état signalé et vérifier ses actions.

- [ ] T061 [US2] Écrire d'abord dans `tests/storage_failures.rs` la relecture des réglages (fichier encore illisible, fichier réparé, aucun plan changé), puis ajouter cette commande dans `src/domain.rs` et `src/coordinator.rs`. Couvre FR-017, FR-026.
- [ ] T062 [US2] Afficher dans `src/interface/messages.rs` les messages intégrés du contrat, sans dialogue bloquant ni animation : plan modifié dans Windows (**Reprendre l'automatisation**), refus de Windows (**Réessayer**, **Ouvrir les options d'alimentation**), retour échoué en quittant (**Réessayer**, **Ouvrir les options d'alimentation**, **Quitter quand même**), batterie (« Automatisation en attente · sur batterie, aucun plan ne sera changé ») et réglages illisibles (**Ouvrir le dossier des données**, **Relire les réglages**, **Ouvrir le guide**). Couvre FR-009, FR-012, FR-013, FR-014, FR-017.
- [ ] T063 [US2] Relier **Mettre en pause** ou **Reprendre** (Ctrl+Alt+P, **Reprendre** en premier en pause), **Quitter** de la barre latérale et de l'icône avec retour protégé, et la fermeture de fenêtre qui garde le suivi, dans `src/interface/software_page.rs`, `src/interface/window.rs` et `src/interface/tray.rs`. Couvre FR-009, FR-012, FR-025.

**Checkpoint**: chaque état de la page « États signalés » du contrat a sa capture GPUI.

## Phase 11: User Story 3 · interface GPUI (Priority: P2)

**Goal**: refaire en GPUI l'édition de la liste de la Phase 6.

**Independent Test**: avec `--demo --gpui`, réordonner deux logiciels, retirer puis annuler le retrait.

- [ ] T064 [US3] Ajouter dans `src/interface/software_page.rs` **Suivre**, **Monter**, **Descendre**, **Modifier** et **Supprimer** sans confirmation, le message de retrait avec **Annuler** (Ctrl+Z) qui réenregistre la configuration précédente à la même position, le texte changé au second retrait, l'annonce « Blender déplacé en position 1 sur 3. », suivie de « Plan actif : … » seulement si le plan actif change, et le focus passé au bouton opposé quand le bouton utilisé devient indisponible. Couvre FR-003, FR-006, FR-007, FR-019.

## Phase 12: User Story 4 · interface GPUI (Priority: P1)

**Goal**: finir le parcours accessible en GPUI et le prouver sur Windows réel.

**Independent Test**: tout le parcours au clavier, puis au Narrateur, en thèmes contrastés et à 200 %.

- [ ] T065 [US4] Construire **Réglages** dans `src/interface/settings_page.rs` (**Plan de retour**, **Changer de plan sur batterie**, **Démarrer avec Windows**, **Ouvrir le dossier**, message du retour réservé au cycle en cours) et **Aide et limites** dans `src/interface/help_page.rs`, avec **Voir les licences** et **Ouvrir le guide**. Couvre FR-002, FR-014, FR-020, FR-024.
- [ ] T066 [US4] Construire la palette de commandes dans `src/interface/palette.rs` (Ctrl+Maj+P, bouton **Commandes**, entrées « Domaine : verbe ») sur le modèle de la palette de Bientôt. Couvre FR-019.
- [ ] T067 [US4] Compléter dans `src/interface/` noms, rôles, états, descriptions, ordre de tabulation, raccourcis Ctrl+N, Ctrl+Z, Ctrl+Alt+P, Ctrl+Maj+P, Entrée et Échap, retour du focus au bouton d'ouverture d'un dialogue, annonce d'une ligne « 1. Battlefield 6, plan Haute performance, appliqué » et annonce d'un statut sans déplacer le focus, avec des tests de vues `HeadlessAppContext`. Couvre FR-019, SC-005.
- [ ] T068 [US4] Adapter dans `src/interface/theme.rs` et `src/interface/window.rs` les thèmes Windows, le contraste avec couleurs système, l'échelle de 100 à 200 %, le texte agrandi et la fenêtre étroite. Couvre FR-019, SC-005.
- [ ] T069 [US4] Animer seulement les quatre transitions du contrat dans `src/interface/software_page.rs`, `src/interface/rule_dialog.rs` et `src/interface/first_launch.rs`, jamais au clavier, avec l'état final direct quand `reduce_motion` est actif, vérifié par un test de vue avec `set_reduce_motion` et `advance_clock`. Les juger dans l'application avec `POWERPLANSYNC_MOTION_SLOWDOWN=4`. Couvre FR-019.
- [ ] T070 [US4] Vérifier sur Windows réel clavier, Narrateur, contrastes, échelles 100 %, 150 % et 200 %, texte agrandi, « Effets d'animation » désactivé, réouverture depuis l'icône, refus « Contrôle indisponible » depuis une autre session et démarrage activé puis désactivé. Enregistrer captures et arbre UI Automation dans `specs/001-app-power-profiles/evidence/accessibility/gpui/`. Couvre FR-019, FR-020, FR-026, SC-005.

**Checkpoint**: le parcours GPUI est accessible sur la machine cible, sans se limiter à une capture.

## Phase 13: Retrait de Slint et documentation

**Purpose**: faire de GPUI l'unique interface, puis préparer la documentation livrée.

- [ ] T071 Rendre GPUI l'interface par défaut et retirer Slint : supprimer `ui/`, `src/ui.rs`, `build.rs`, `slint` et `slint-build` de `Cargo.toml`, l'option `--gpui`, `examples/ui_probe.rs` et `examples/ui_candidate.rs`. Porter `examples/window_lifecycle.rs` vers GPUI pour `scripts/verify-window-lifecycle.ps1`, retirer toute mention de Slint de `THIRD_PARTY_NOTICES.md` (section Slint et paragraphe des ressources intégrées) et de `docs/guide-utilisateur.md` (mention de **À propos**), régénérer `notices/THIRD_PARTY_LICENSES.txt`, mettre à jour la section prototype de `README.md` et retirer la boucle d'événements et l'icône de notification Slint de `src/runtime.rs`, mettre à jour les mentions Slint de `.agents/skills/verify/SKILL.md` et les fiches `.agents/skills/verify/features/*.md` avec les libellés de `contracts/user-interface.md`. Couvre FR-019, SC-007.
- [ ] T072 À la livraison de l'interface GPUI, avec T044, remplacer `docs/guide-utilisateur.md` par le texte de `specs/001-app-power-profiles/design/guide-gpui.md` : retirer l'encadré de brouillon, reprendre les chemins des notices du paquet et le numéro de version dans le titre, puis laisser dans `design/guide-gpui.md` un renvoi vers `docs/guide-utilisateur.md`. Couvre FR-018, FR-020.

**Checkpoint**: aucun fichier ni dépendance Slint ne reste, et les contrôles Cargo passent.

## Phase 14: Polish & Cross-Cutting Concerns

**Purpose**: produire les preuves qui autorisent une première livraison, sur le binaire GPUI
issu de T071.

- [ ] T039 [P] Créer `scripts/measure-overhead.ps1`, exécuter le protocole CPU, mémoire et cent cycles de fenêtre, puis conserver données brutes et traces dans `specs/001-app-power-profiles/evidence/performance/`. Couvre FR-026, SC-004, SC-007.
- [ ] T040 Créer `scripts/verify-windows.ps1` avec sauvegarde et restauration finale du plan de test, mesurer 100 cycles depuis création et sortie réelles avec offsets variés entre observations, puis reprendre tous les cas du guide dans `specs/001-app-power-profiles/evidence/validation/`. Couvre FR-010, FR-026, SC-002, SC-003.
- [ ] T041 Vérifier le cycle de l'exécutable témoin avec son lanceur restant ouvert, plusieurs instances et un arrêt forcé, puis enregistrer activation, maintien, retour et coût dans `specs/001-app-power-profiles/evidence/fixture-validation.md`. Aucun lancement de Battlefield 6. Couvre FR-004, FR-022, FR-026, SC-006.
- [ ] T042 Faire essayer le parcours initial à cinq personnes non spécialistes et consigner temps et réussite anonymisés dans `specs/001-app-power-profiles/evidence/usability.md`. Couvre FR-018, SC-001.
- [ ] T043 Auditer droits demandés, réseau absent (y compris par les dépendances GPUI), données collectées et rotation des diagnostics dans `src/windows/`, `src/interface/`, `src/diagnostics.rs` et `specs/001-app-power-profiles/evidence/privacy.md`. Couvre FR-020, FR-021, FR-022.
- [ ] T044 Préparer le binaire release portable, les notices requises et le guide d'installation et retrait dans `README.md` et `THIRD_PARTY_NOTICES.md`, en vérifiant les licences publiées de `gpui-pre`, `tray-icon` et des polices IBM Plex, et en résolvant la licence de distribution avant publication. Couvre FR-020, FR-022.
- [ ] T045 Exécuter les contrôles Cargo et le guide complet sur les Windows cibles, ou dans la CI Windows prévue (issue #7) pour ce qu'elle couvre, mettre à jour `specs/001-app-power-profiles/checklists/release.md`, puis relancer `speckit-analyze` et `speckit-converge`. Couvre FR-026 et SC-001 à SC-007.

## Dependencies & Execution Order

### Phase Dependencies

T001 précède tous les prototypes. T002 et T003 peuvent avancer sur des fichiers distincts après
épinglage commun des dépendances. T004 dépend de T003. T005 dépend de T002. T006 examine leurs preuves.
Les preuves Windows de T002 et T005 autorisent T007 à T013 et le moteur T014 à T016.
T004 et T006 sont remplacées par T048 et T049, qui précèdent l'interface GPUI définitive.
Les tests T020 et la récupération T021 accompagnent la transaction de T016,
avant son intégration UI. T008 précède T009. T010 à T012 partagent les types de T007, mais leurs
fichiers sont disjoints. T013 les assemble. Toute modification de Cargo reste sérialisée.

Migration GPUI : T046 précède T047. T048 dépend de T047, T049 de T048. T049 autorise T051 à T057. T050 ne dépend pas du cadre graphique et a précédé T048.
T050 précède les pages. T051, T052 et T055 sont disjoints. T053 attend T051, T054 attend T053,
T056 attend T054, T055 et T012, encore ouverte, qui fournit l'ouverture de la fenêtre dans la même session. T057 attend T056. Les phases 9 à 12 attendent T056. T061 précède T062.
T064 précède T069, qui anime aussi l'échange de lignes. T070 attend T058 à T069. T071 attend T070.
T039 à T045 portent sur le binaire de T071. T072 accompagne T044 à la livraison.

### User Story Dependencies

US1 suit les fondations. US2 durcit le cycle US1 et le valide indépendamment avec erreurs injectées.
US4 finalise l'interface déjà minimale de US1. US3 vient ensuite car P2 et partage le moteur et l'UI.
Les phases ne sont pas parallélisées sur les mêmes fichiers. T039 à T045 suivent les parcours.
Dans la vague GPUI, US3 (T064) passe avant US4 (T065 à T070), car la vérification finale d'US4
couvre aussi la liste réordonnable. T019, T027 et T038 restent valables : ils vérifient le moteur
et Windows, pas l'interface, et peuvent s'exécuter avant ou pendant la migration.
Les mesures T039 et essais T040/T041 utilisent le même PC et restent sérialisés.

### Within Each User Story

Les scénarios de décision sont écrits avant le comportement correspondant et constatés en échec
pour la bonne raison. Les vérifications graphiques portent sur le binaire optimisé. Une correction
de comportement met à jour la spec avant le code si elle change le contrat.

### Parallel Opportunities

L'annotation `[P]` signifie une possibilité d'écriture disjointe, pas une autorisation de tests
Windows simultanés. Après T001, T002 et T003 sont séparables. Après T007, T008, T010, T011 et T012
peuvent avancer séparément. Après T050, T051, T052 et T055 sont séparables, puis T057 après T056.
Un responsable unique assemble ces fichiers et exécute les essais.

## Parallel Example: User Story 1

T017 peut préparer le dialogue à partir du contrat pendant T015 et T016.
T018 attend les trois. US2 reste séquentielle car moteur, coordinateur et intégration se recouvrent.
Pour US4, une revue d'accessibilité en lecture seule peut accompagner T033. Pour US3, la collecte
des scénarios de démonstration peut accompagner T037, puis T038 attend l'intégration.
En GPUI, T059 et T060 peuvent avancer en parallèle après T058, sur des fichiers distincts.

## Implementation Strategy

Le premier livrable de développement est la preuve de faisabilité, puis la démonstration US1.
Pour la migration, le premier livrable est la sonde GPUI vérifiée (T048), puis US1 en `--gpui`.
L'application Slint reste l'interface par défaut jusqu'à T071 ; chaque étape reste utilisable.
Une version utilisable au quotidien exige toutes les protections US2, l'accessibilité US4 et
les validations finales. Une case n'est cochée qu'avec son fichier et sa preuve disponibles.
Un résultat inaccessible ou non exécuté demeure un échec de validation pour la promesse concernée.

Ne pas publier le binaire, créer une tâche persistante ni changer les réglages personnels au titre
de cette conception. Ces tâches décrivent le futur produit. La livraison sera décidée séparément.

## Notes

Le plan suppose une UI française. Slint était le candidat initial ; GPUI le remplace par décision
produit du 2026-10-25 (D12). Un échec mesuré de GPUI revient au responsable produit avec une décision
écrite. Le design system Bientôt GPUI n'est pas modifié depuis ce dépôt. L'absence de Rust dans l'environnement actuel est traitée par T001.
L'outil ne garantit ni une hausse des FPS, ni une restauration immédiate après sa fermeture forcée.
