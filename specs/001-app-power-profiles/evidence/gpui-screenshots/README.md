# Captures GPUI hors écran

Rendues sous Linux par `src/bin/screenshots.rs` (T057), à l'échelle 2, avec lavapipe
(`mesa-vulkan-drivers`) :

```sh
cargo run --features screenshots --bin screenshots -- specs/001-app-power-profiles/evidence/gpui-screenshots
```

Seul le cadre de la fenêtre est en place. La page **Logiciels** montre les composants de T053
en attendant T058, la page **Réglages** est vide en attendant T065. Les coins sont carrés : hors
écran, aucun compositeur ne les arrondit.

## Comparaison aux maquettes

| Capture | Maquette | Constat |
| --- | --- | --- |
| `window-dark.png` | `Main.png` | Conforme : barre de titre, barre latérale, barre d'état, cotes, couleurs et typographie. Les composants suivent le thème sombre. |
| `window-light.png` | `MainLight.png` | Conforme : cadre et surfaces du thème clair, accent sur le bouton teinté, le badge « Appliqué » et le Switch. |
| `window-narrow.png` | `Narrow.png` | Conforme : 420 × 760, **Commandes** masqué, barre latérale réduite aux icônes, pas de compte dans la barre d'état. |
| `window-battery.png` | `Battery.png` | Conforme : automatisation en attente, « Sur batterie », **Réglages** sélectionné comme dans `Settings.png`. |
| `unavailable-dark.png` | aucune (T058) | Cadre conforme à `Main.png`. Titre, texte et **Réessayer** du contrat ; pas de compte dans la barre d'état. |
| `unavailable-light.png` | aucune (T058) | Mêmes constats en thème clair. |

Aucun écart du cadre hors de ceux listés dans `design/README.md`.
