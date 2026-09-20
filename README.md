# PowerPlanSync

PowerPlanSync est un projet d'application Windows en Rust. Il associe un plan d'alimentation
à un logiciel et revient à un plan choisi lorsque ce logiciel se ferme.
Le premier scénario est Battlefield 6 avec Haute performance pendant le jeu et Utilisation normale après.

Le premier lot de faisabilité est implémenté en Rust. Il comprend une sonde Windows,
un petit exécutable témoin et un prototype d'interface. L'application de suivi complète
n'est pas encore implémentée. Aucun essai n'a lancé Battlefield.

Les [résultats du prototype](specs/001-app-power-profiles/evidence/prototype-results.md)
contiennent les mesures, les captures et les vérifications encore ouvertes.

## Essayer les prototypes

Sur cette machine, les exécutables compilés sont dans `target/release/examples/`.

```powershell
.\target\release\examples\ui_probe.exe
.\scripts\verify-prototypes.ps1
```

La fenêtre utilise des données de démonstration et ne change aucun réglage Windows.
La seconde commande vérifie les plans et les processus sans basculer de plan.
Pour une séance dédiée de bascules réelles avec restauration du plan initial :

```powershell
.\scripts\verify-prototypes.ps1 -AllowPowerSwitch -Cycles 3
```

La sonde choisit comme cible un autre plan existant, puis revient au plan actif au début du test.
Le témoin dort dix secondes. Les essais différencient son chemin, ses instances,
un fichier homonyme et un lanceur restant ouvert. Fermer la fenêtre laisse l'icône active ;
utiliser **Quitter** pour arrêter la sonde UI.

## Compiler

Avec Rust et les outils MSVC installés normalement, utiliser `cargo build --locked --release --examples`.
L'installation locale de cette séance est dans `.tools`, hors Git et sans changement global du PATH.

```powershell
$env:CARGO_HOME = Join-Path $PWD '.tools/cargo'
$env:RUSTUP_HOME = Join-Path $PWD '.tools/rustup'
$env:PATH = (Join-Path $env:CARGO_HOME 'bin') + ';' + $env:PATH
cargo build --locked --release --examples
```

`cargo run` affiche uniquement l'état du projet. Les sondes sont des exemples séparés.

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

Ouvrir ce projet et utiliser `$speckit-implement` pour poursuivre les tâches de faisabilité.
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
