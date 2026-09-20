# PowerPlanSync

PowerPlanSync est un projet d'application Windows en Rust. Il associe un plan d'alimentation
à un logiciel et revient à un plan choisi lorsque ce logiciel se ferme.
Le premier scénario est Battlefield 6 avec Haute performance pendant le jeu et Utilisation normale après.

Le projet est en phase de conception. Aucun binaire produit n'est encore développé.
Les mesures de consommation, l'accessibilité et la compatibilité Battlefield restent à vérifier.

## Lire la conception

- [Spécification du produit](specs/001-app-power-profiles/spec.md).
- [Recherche, sources et décisions](specs/001-app-power-profiles/research.md).
- [Plan technique](specs/001-app-power-profiles/plan.md).
- [Tâches de réalisation](specs/001-app-power-profiles/tasks.md).
- [Parcours de validation Windows](specs/001-app-power-profiles/quickstart.md).
- [Constitution du projet](.specify/memory/constitution.md).

La conception suit GitHub spec-kit 1.0.8. Les skills Codex officiels sont dans `.agents/skills`.
La fonctionnalité active est `specs/001-app-power-profiles`, enregistrée dans `.specify/feature.json`.

## Continuer avec Codex

Ouvrir ce projet et utiliser `$speckit-implement` pour commencer les tâches de faisabilité.
Si la session ne recharge pas les nouveaux skills, rouvrir le projet. Les instructions installées
restent directement lisibles dans `.agents/skills/speckit-implement/SKILL.md`.

L'installation locale du CLI est dans `.tools/bin/specify.exe`, dossier exclu de Git.
Depuis un autre poste, installer la même version officielle avec uv.

```powershell
uv tool install specify-cli --from git+https://github.com/github/spec-kit.git@v1.0.8
specify version
```

Les modèles et skills présents suffisent à poursuivre le travail. Ne pas réinitialiser le dépôt
avec une option d'écrasement pour une simple reprise.

## Vérifier les prérequis documentaires

```powershell
.\.specify\scripts\powershell\check-prerequisites.ps1 -Json -RequireSpec -RequireTasks -IncludeTasks
```

Ce contrôle vérifie les documents disponibles. Il ne certifie ni le fonctionnement du futur
produit ni ses objectifs de performances.
