# Feuille de route

Cette feuille de route pilote la routine qui termine le développement de PowerPlanSync,
puis assure sa maintenance. Seule la routine modifie ce fichier.
Elle ne remplace pas la conception : le plan produit reste dans
`specs/001-app-power-profiles/` et l'ordre des tâches dans `tasks.md`.

## Cap

Livrer une v1 portable pour Windows 11, avec une interface GPUI conforme à la conception
approuvée, validée sur un Windows réel avec l'exécutable témoin, puis la maintenir.

## Règles

- Ne jamais pousser sur `main`. Une seule PR de la routine à la fois, sur la branche
  `claude/project-thread-rmf7v6`, repartie de `main` après chaque merge.
- Dépôt public (consigne du 26 octobre 2026) : titres de commits et de PR en Conventional
  Commits avec un scope (`app`, `interface`, `windows`, `docs`, `ci`, `site`…). La description
  d'une PR décrit directement le changement, sobrement, sans raconter ce que Clément a fait.
  Aucune attribution : ni ligne « Requested by », ni marqueur, ni lien vers le projet Claude
  dans les PR, ni trailer dans les commits. Les notes de version excluent les commits du site.
  Chaque PR porte `Closes #N` (ou `Refs #N` pour un epic) et est assignée à Clément.
- Valider en local avant chaque push : `cargo fmt --check`,
  `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings`, puis
  sous Linux (`libfontconfig-dev`, `mesa-vulkan-drivers`)
  `cargo clippy --locked --all-targets --features screenshots -- -D warnings` et
  `cargo test --locked --lib`. Les tests propres à Windows tournent dans la CI Windows (#7). Les
  captures se rendent sous Linux avec `cargo run --features screenshots --bin screenshots`.
- Merger soi-même (squash, titre Conventional Commits) seulement si les trois conditions
  sont réunies : la dernière revue de `pullfrog[bot]` sur le SHA de tête est APPROVED,
  ou COMMENTED avec « No new issues found » (Pullfrog n'approuve jamais), toute la CI
  est verte, aucun conflit. Jamais si Clément a demandé des changements ou écrit d'attendre.
- Si le budget GitHub Actions est épuisé, Pullfrog et la CI sont remplacés par une revue d'un
  agent relecteur dédié, lancé à neuf sur le diff du SHA de tête, et par la validation locale.
  Le merge exige alors son verdict APPROVED sur ce SHA, publié en commentaire sur la PR.
- Autonomie : Clément a demandé le 25 octobre que la routine décide seule. Elle tranche
  les choix réversibles (design, libellés, priorités, règles de la routine) et ne lui laisse
  que ce qui exige son PC ou son compte, et le choix de la licence.
- Interface : chaque décision de design ou de libellé passe par Designer
  (`design:design-critique`), UX Writer (`design:ux-copy`) et Screenshot reviewer
  (un agent qui compare les captures de l'application aux maquettes de
  `specs/001-app-power-profiles/design/ecrans/`), jusqu'à leur accord. Chaque changement
  d'UI met à jour les maquettes. Le design system
  « Bientôt GPUI » n'est pas modifié depuis ce dépôt : les écarts sont listés dans
  `specs/001-app-power-profiles/design/README.md` et leur adoption est demandée au projet Bientôt.
- Ne jamais lancer Battlefield. Ne jamais toucher aux PR des autres fils.
- Issues : labels `bug`, `enhancement` ou `documentation`, jalon v1, cases des epics à jour.
- Rythme : deux passages par jour, à 8 h 47 et 20 h 47, heure de Paris (décision du 27 octobre
  2026, la consommation étant devenue raisonnable). Pas de rythme plus serré sans demande de
  Clément.

## Priorités

### 1. Bugs ouverts

Aucun bug ouvert au 25 octobre 2026.

### 2. Fonctionnalités

- [ ] Migrer l'interface de Slint vers GPUI (#5). La conception est synchronisée
      ([#4](https://github.com/Thanaen/PowerPlanSync/pull/4)). La migration est planifiée
      ([#10](https://github.com/Thanaen/PowerPlanSync/pull/10)). La routine enchaîne les tâches
      T046 à T072 dans l'ordre de `tasks.md`. T048 et T070 se font sur un Windows réel.
- [x] Ajouter une CI Windows : formatage, Clippy, tests ([#11](https://github.com/Thanaen/PowerPlanSync/pull/11)).

### 3. Tâches qui attendent Clément

- [x] Relire et merger la synchronisation de la conception GPUI ([#4](https://github.com/Thanaen/PowerPlanSync/pull/4)).
- [x] Budget GitHub Actions rétabli le 26 octobre 2026 : Pullfrog et la CI tournent de nouveau.
- [ ] Créer le jalon v1 : l'outil GitHub de la routine ne crée pas de jalon.
- [ ] Valider la v1 sur un Windows réel (#6).
- [ ] Choisir la licence et préparer la livraison (#8).

## Journal

| Date | PR | Résultat |
| --- | --- | --- |
| 2026-10-25 | [#9](https://github.com/Thanaen/PowerPlanSync/pull/9) | Feuille de route créée, issues #5 à #8 ouvertes. |
| 2026-10-25 | [#4](https://github.com/Thanaen/PowerPlanSync/pull/4) | Conception GPUI synchronisée (fil de la refonte GPUI). |
| 2026-10-25 | [#11](https://github.com/Thanaen/PowerPlanSync/pull/11) | CI Windows ajoutée ; règles de merge et d'autonomie mises à jour. |
| 2026-10-25 | [#10](https://github.com/Thanaen/PowerPlanSync/pull/10) | Migration GPUI planifiée (fil de la refonte GPUI). |
| 2026-10-25 | [#12](https://github.com/Thanaen/PowerPlanSync/pull/12) | T046 : dépendances GPUI épinglées. |
| 2026-10-26 | [#13](https://github.com/Thanaen/PowerPlanSync/pull/13) | T047 : sonde GPUI, compilée ; essais réels en T048. |
| 2026-10-26 | [#14](https://github.com/Thanaen/PowerPlanSync/pull/14) | T050 : présentation sans type graphique, libellés du contrat, tests. |
| 2026-10-26 | [#15](https://github.com/Thanaen/PowerPlanSync/pull/15) | T052 : jetons de mouvement et réglage « Effets d'animation ». |
| 2026-10-26 | [#16](https://github.com/Thanaen/PowerPlanSync/pull/16) | T051 : thème WorkTime One, écarts, couleurs de l'icône et contraste élevé, polices IBM Plex. |
| 2026-10-26 | [#17](https://github.com/Thanaen/PowerPlanSync/pull/17) | T055 : icône de notification, infobulle, menu natif et notification du premier masquage. |
| 2026-10-26 | [#18](https://github.com/Thanaen/PowerPlanSync/pull/18) | T053 : boutons, Switch, badges, liste de plans, messages, touches, champ texte et pictogrammes Lucide. |
| 2026-10-26 | [#19](https://github.com/Thanaen/PowerPlanSync/pull/19) | T054 : fenêtre GPUI, barre de titre, barre latérale, barre d'état et fenêtre étroite. |
| 2026-10-26 | [#20](https://github.com/Thanaen/PowerPlanSync/pull/20) | T056 : interface GPUI derrière `--gpui`, vue « Contrôle indisponible » ; T012 constatée terminée. |
| 2026-10-27 | [#21](https://github.com/Thanaen/PowerPlanSync/pull/21) | T057 : crate compilable sous Linux, binaire `screenshots` et captures comparées aux maquettes. |
| 2026-10-28 | [#22](https://github.com/Thanaen/PowerPlanSync/pull/22) | T058 : page **Logiciels** (en-tête, actions, liste) et maquette `Unavailable.png`. |
| 2026-10-28 | #23 | T059 : dialogues **Ajouter un logiciel** et **Modifier**, Ctrl+N, sélecteur de fichier sur thread STA. |
