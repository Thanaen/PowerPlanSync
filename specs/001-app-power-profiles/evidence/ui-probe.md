# Notes techniques de la sonde Slint

La sonde utilise Slint 1.18.0. `SystemTrayIcon` est un composant racine distinct de la fenêtre.
Sous Windows, Slint le relie à la zone de notification avec `Shell_NotifyIcon`.
La fenêtre est créée à l'ouverture et retirée de l'état Rust quand elle est masquée.
Le code prévoit sa recréation depuis l'icône. `--smoke` programme un cycle fini masquer,
recréer et quitter ; son code de sortie seul ne prouve pas la réouverture par le menu utilisateur.

Les contrôles viennent de `std-widgets.slint`. Les libellés français décrivent le statut et
les actions sans dépendre de la couleur. Le graphe Cargo confirme les fonctionnalités `std`,
`backend-winit`, `renderer-software`, `accessibility`, `compat-1-18` et `system-tray`.
Le build release des exemples réussit. Les premiers essais clavier, de mise à l'échelle et
de ressources sont consignés plus bas. Narrateur, contrastes et sessions restent à vérifier.

Slint 1.18.0 déclare `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR
LicenseRef-Slint-Software-3.0` dans ses métadonnées publiées.
Le régime de distribution devra être choisi avant toute publication. Ce prototype n'est pas publié.

L'[aperçu officiel de la version 1.18.0](https://github.com/slint-ui/slint/blob/v1.18.0/LICENSE.md)
et le [texte Royalty-free 2.0](https://github.com/slint-ui/slint/blob/v1.18.0/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md)
ont été lus le 20 septembre 2026. Cette dernière option permet une application de bureau propriétaire
avec attribution. Elle demande soit le widget `AboutSlint` dans un écran À propos accessible,
soit le badge Slint sur une page publique facilement trouvable. Les notices des sources doivent rester.
Elle exclut notamment les systèmes embarqués et la redistribution du framework seul.
Les autres licences des dépendances et ressources devront aussi être recensées avant distribution.
Ce constat ne choisit pas la licence du projet.

Les contrôles réels de cette séance sont consignés dans [prototype-results.md](prototype-results.md).
