# Notes techniques de la sonde Slint

La sonde utilise Slint 1.18.0. `SystemTrayIcon` est un composant racine distinct de la fenêtre.
Sous Windows, Slint le relie à la zone de notification avec `Shell_NotifyIcon`.
La fenêtre est créée à l'ouverture, retirée de l'état Rust quand elle est masquée, puis recréée
depuis l'icône. `--smoke` exécute un cycle fini masquer, recréer et quitter.

Les contrôles viennent de `std-widgets.slint`. Les libellés français décrivent le statut et
les actions sans dépendre de la couleur. Le graphe Cargo confirme les fonctionnalités `std`,
`backend-winit`, `renderer-software`, `accessibility`, `compat-1-18` et `system-tray`.
Le build release des exemples réussit. Les essais clavier, Narrateur, DPI et ressources restent
réservés à la validation Windows réelle.

Slint 1.18.0 déclare `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR
LicenseRef-Slint-Software-3.0` dans ses métadonnées publiées.
Le régime de distribution devra être choisi avant toute publication. Ce prototype n'est pas publié.
