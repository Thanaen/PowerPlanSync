# PowerPlanSync

PowerPlanSync est un projet d'application Windows en Rust. Il associe un plan d'alimentation
à un logiciel et revient à un plan choisi lorsque ce logiciel se ferme.
Le premier scénario est Battlefield 6 avec Haute performance pendant le jeu et Utilisation normale après.

L'application, son moteur Rust et son interface française sont implémentés. La validation
Windows est en cours avec un petit exécutable témoin. Aucun essai ne lance Battlefield.
Les cases ouvertes du [plan de réalisation](specs/001-app-power-profiles/tasks.md)
identifient les essais restant à effectuer avant une distribution publique.

Le travail est en pause à la demande de l'utilisateur depuis le 20 septembre 2026.
La [note de reprise](specs/001-app-power-profiles/evidence/checkpoint-2026-09-20.md)
indique les résultats vérifiés et la première action à reprendre. Le dépôt GitHub est privé.
Le [guide utilisateur](docs/guide-utilisateur.md) décrit le fonctionnement actuel.

## Utiliser l'application

Lancer `target/release/power-plan-sync.exe`, ajouter un logiciel avec **Parcourir**,
choisir son plan, confirmer le plan de retour, puis cocher **Activer l'automatisation**.
L'application utilise uniquement les plans déjà disponibles sur le PC.

Le premier logiciel actif de la liste a priorité. Fermer la fenêtre conserve le suivi
dans la zone de notification. **Pause** et **Quitter** tentent le retour choisi.
Un changement manuel du plan dans Windows est conservé ; **Reprendre** relance le suivi.
Le retour d'un cycle déjà commencé reste celui choisi au début de ce cycle.

Pour découvrir l'interface sans modifier Windows :

```powershell
.\target\release\power-plan-sync.exe --demo
```

Les réglages sont locaux dans `%LOCALAPPDATA%\PowerPlanSync`. La démonstration dispose
de données séparées. `--data-dir CHEMIN` choisit un autre dossier ; seul `--demo` simule
les changements. Le démarrage avec Windows est facultatif et disponible avec le dossier normal.

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

Avec Rust et les outils MSVC installés normalement, utiliser `cargo build --locked --release --bins --examples`.
L'installation locale de cette séance est dans `.tools`, hors Git et sans changement global du PATH.

```powershell
$env:CARGO_HOME = Join-Path $PWD '.tools/cargo'
$env:RUSTUP_HOME = Join-Path $PWD '.tools/rustup'
$env:PATH = (Join-Path $env:CARGO_HOME 'bin') + ';' + $env:PATH
cargo build --locked --release --bins --examples
```

`cargo run -- --demo` ouvre la démonstration. Les sondes restent des exemples séparés.

Les contrôles du code sont `cargo test --locked --all-targets`, `cargo fmt --check` et
`cargo clippy --locked --all-targets -- -D warnings`. Le harnais
`scripts/verify-windows.ps1 -AllowPowerSwitch` vérifie cent cycles sur l'application réelle
et restaure le plan initial. Il exige PowerShell 7.5 ou plus récent et aucune instance
de PowerPlanSync déjà ouverte.

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

Ouvrir ce projet et utiliser `$speckit-implement` pour poursuivre les tâches encore ouvertes.
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
