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
- Commits et titres de PR en Conventional Commits avec un scope. Descriptions sobres,
  sans attribution. Chaque PR porte `Closes #N` et est assignée à Clément.
- Valider en local avant chaque push : `cargo fmt --check`, puis
  `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings`.
  Les tests dépendent de Windows et ne tournent pas dans le conteneur Linux ; la CI
  Windows (#7) les exécutera.
- Merger soi-même (squash, titre Conventional Commits) seulement si les trois conditions
  sont réunies : la dernière revue de `pullfrog[bot]` est APPROVED sur le SHA de tête,
  toute la CI est verte, aucun conflit. Jamais si Clément a demandé des changements
  ou écrit d'attendre.
- Interface : chaque décision de design ou de libellé passe par Designer
  (`design:design-critique`), UX Writer (`design:ux-copy`) et Screenshot reviewer,
  jusqu'à leur accord. Chaque changement d'UI met à jour les maquettes et le design system.
- Ne jamais lancer Battlefield. Ne jamais toucher aux PR qui ne sont pas celles de la routine,
  en particulier #4.
- Issues : labels `bug`, `enhancement` ou `documentation`, jalon v1, cases des epics à jour.
- Rythme : toutes les 2 h (heure de Paris) tant qu'il reste du travail faisable seul ;
  une fois par jour quand il ne reste que des tâches qui attendent Clément.

## Priorités

### 1. Bugs ouverts

Aucun bug ouvert au 25 octobre 2026.

### 2. Fonctionnalités

- [ ] Migrer l'interface de Slint vers GPUI (#5). Commence après le merge de #4 et la
      planification dans `plan.md` et `tasks.md`, faite dans le fil de la refonte GPUI.
- [ ] Ajouter une CI Windows : formatage, Clippy, tests (#7). Attend le budget Actions.

### 3. Tâches qui attendent Clément

- [ ] Relire et merger la synchronisation de la conception GPUI (#4).
- [ ] Débloquer le budget GitHub Actions : Pullfrog ne démarre plus depuis le 17 octobre,
      et aucune CI ne peut tourner.
- [ ] Confirmer la règle de merge : Pullfrog publie ses revues en COMMENTED, jamais APPROVED.
- [ ] Créer le jalon v1 : l'outil GitHub de la routine ne crée pas de jalon.
- [ ] Valider la v1 sur un Windows réel (#6).
- [ ] Choisir la licence et préparer la livraison (#8).

## Journal

| Date | PR | Résultat |
| --- | --- | --- |
| 2026-10-25 | — | Feuille de route créée, issues #5 à #8 ouvertes. |
