# Chaîne de compilation retenue

Le prototype épingle Rust 1.98.1 avec la cible MSVC x64 et l'édition Rust 2024.
La machine a fourni `rustc 1.98.1 (48a229cea 2026-09-01)` et
`cargo 1.98.1 (797e8a9bc 2026-08-05)`.
Il épingle `windows` 0.62.2, `slint` 1.18.0 et `slint-build` 1.18.0.

Le build utilise MSVC 14.44.35207 et le SDK Windows 10.0.26100.0.
Rust et Cargo ont été installés dans `.tools/rustup` et `.tools/cargo` ; MSVC dans `.tools/msvc`.
Ces outils locaux ne sont pas versionnés. Aucun ajout global au PATH n'a été effectué.

Slint désactive ses fonctionnalités par défaut. Le prototype active seulement `std`,
`backend-winit`, `renderer-software`, `accessibility`, `compat-1-18` et `system-tray`.
`compat-1-18` est la fonctionnalité de compatibilité minimale exigée par Slint 1.18.

`cargo build --locked --release --examples`, `cargo test --locked` et
`cargo clippy --locked --all-targets -- -D warnings` réussissent dans le worktree du prototype.
La racine a réexécuté ces contrôles après intégration, sources identiques au worktree de construction.
Les essais système séparés et les premières mesures figurent dans [prototype-results.md](prototype-results.md).
