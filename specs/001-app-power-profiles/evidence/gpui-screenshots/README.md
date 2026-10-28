# Captures GPUI hors écran

Rendues sous Linux par `src/bin/screenshots.rs` (T057), à l'échelle 2, avec lavapipe. Sous
Debian ou Ubuntu, le build demande les en-têtes de fontconfig et le rendu un pilote Vulkan :

```sh
sudo apt-get install libfontconfig-dev mesa-vulkan-drivers
cargo run --features screenshots --bin screenshots -- specs/001-app-power-profiles/evidence/gpui-screenshots
```

La page **Logiciels** est en place (T058). La page **Réglages** est vide en attendant T065. Les
coins sont carrés : hors écran, aucun compositeur ne les arrondit. `widgets-dark.png` et
`widgets-light.png` montrent les composants de T053 seuls.

## Comparaison aux maquettes

| Capture | Maquette | Constat |
| --- | --- | --- |
| `main-dark.png` | `Main.png` | Conforme. |
| `main-light.png` | `MainLight.png` | Conforme. |
| `main-narrow.png` | `Narrow.png` | Conforme : lignes sur trois niveaux, actions l'une sous l'autre. |
| `paused.png` | `Paused.png` | Conforme : **Reprendre** en premier, plan de retour rétabli. |
| `battery.png` | `Battery.png` | Conforme : attente sur batterie, badge **Ouvert**. |
| `unavailable-dark.png` | `Unavailable.png` | Maquette tirée de cette capture. |
| `unavailable-light.png` | aucune | Mêmes constats en thème clair. |

Aucun écart hors de ceux listés dans `design/README.md`.
