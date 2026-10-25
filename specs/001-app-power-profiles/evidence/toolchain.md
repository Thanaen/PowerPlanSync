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

## Dépendances GPUI (T046, 25 octobre 2026)

La migration ajoute GPUI sans retirer Slint. Les versions résolues dans `Cargo.lock` sont :

| Dépendance | Paquet | Version | Cible |
| --- | --- | --- | --- |
| `gpui` | `gpui-pre` | 0.3.9 | toutes |
| `gpui_platform` | `gpui-pre-platform` | 0.3.9 | toutes |
| `image` (`ico`, `png`) | `image` | 0.25.10 | toutes |
| `futures` | `futures` | 0.3.34 | Windows |
| `raw-window-handle` | `raw-window-handle` | 0.6.2 | Windows |
| `tray-icon`, sans fonctionnalités par défaut | `tray-icon` | 0.26.1 | Windows |
| `gpui_wgpu`, optionnel | `gpui-pre-wgpu` | 0.3.9 | Linux |

`tray-icon` 0.26.1 apporte `muda` 0.21.2. Slint garde de son côté `muda` 0.19.3.
La fonctionnalité `screenshots` active `gpui/test-support`, `gpui_platform/test-support`
et `gpui_wgpu`. Elle ne sert qu'au binaire de captures prévu en T057.

`rust-toolchain.toml` déclare maintenant `components = ["rustfmt", "clippy"]`.
Les contrôles ont été exécutés dans un conteneur Linux avec `rustc 1.98.1 (48a229cea 2026-09-01)`
et `cargo 1.98.1 (797e8a9bc 2026-08-05)` :

- `cargo fmt --check` réussit ;
- `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings` réussit.

Ce contrôle croisé vérifie les types et Clippy pour la cible Windows, sans édition de liens.
La compilation complète et les tests restent à exécuter sur Windows, par la CI ou sur le poste.
