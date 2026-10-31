# Sonde GPUI (T047)

La sonde `examples/gpui_probe.rs` vérifie que GPUI 0.3.9 (paquet `gpui-pre`) fournit les API
dont l'interface a besoin. Elle ne change aucun réglage Windows et n'utilise aucune donnée réelle.

```powershell
cargo run --locked --release --example gpui_probe
cargo run --locked --release --example gpui_probe -- --smoke
cargo run --locked --release --example gpui_probe -- --tray-only
pwsh scripts/verify-window-lifecycle.ps1 -ExecutablePath target/release/examples/gpui_probe.exe -Arguments --lifecycle
```

`--smoke` ouvre la fenêtre, la détruit, la rouvre, la détruit de nouveau puis quitte.
`--tray-only` démarre sans fenêtre, pour mesurer GPUI avant tout rendu. `--lifecycle` suit le
protocole des cent cycles de `scripts/verify-window-lifecycle.ps1`. Ctrl+Alt+A change le statut
sans déplacer le focus, pour vérifier son annonce. `POWERPLANSYNC_PROBE_CLOSE=wm` ferme les fenêtres
de `--lifecycle` par `WM_CLOSE`, avec le contournement de la fuite ; `wm-raw` sans lui.
Sans option, la sonde reste dans la zone de notification après la fermeture de la fenêtre.
**Ouvrir la sonde** ou un clic gauche sur l'icône rouvre la fenêtre ; **Quitter la sonde** arrête tout.

## État

La sonde compile pour `x86_64-pc-windows-msvc` : `cargo clippy --locked --all-targets
--target x86_64-pc-windows-msvc -- -D warnings` réussit le 26 octobre 2026, dans le conteneur Linux
de la routine. Le 31 octobre 2026, elle a été exécutée sur Windows réel (T048) : résultats dans
[prototype-results.md](prototype-results.md), section « Sonde GPUI sur Windows réel », et
décision dans D17 de [research.md](../research.md) : option 1, la fenêtre dans un processus
séparé (D18).

## API confirmées dans les sources

| Besoin | API utilisée | Remarque |
| --- | --- | --- |
| Fenêtre créée à la demande | `cx.open_window(WindowOptions, …)` | Renvoie un `WindowHandle`. |
| Fenêtre détruite à la fermeture | `window.remove_window()` | La vue est libérée avec la fenêtre. |
| Application gardée après fermeture | `Application::with_quit_mode(QuitMode::Explicit)` | Arrêt par `cx.quit()`. |
| Fermeture observée | `cx.on_window_closed` | Plus fiable qu'un `update` refusé, qui peut venir d'un `App` occupé. |
| Barre de titre dessinée | `TitlebarOptions { appears_transparent: true }` | Zones par `window_control_area(Drag / Close)`. |
| Rôles et noms accessibles | `.role(Role::…)`, `.aria_label(…)` | Rôles AccessKit, adaptateur `accesskit_windows` 0.34. |
| Interrupteur | `Role::Switch`, `.aria_toggled(Toggled::…)` | |
| Liste | `Role::List`, `Role::ListItem`, `aria_position_in_set`, `aria_size_of_set`, `aria_selected` | |
| Statut | `Role::Status` | Voir les manques. |
| Ordre de tabulation | `.focusable()`, `.tab_stop(true)`, `window.focus_next` | Liés à Tab et Maj+Tab. |
| Mouvement réduit | `cx.set_reduce_motion(…)` | Lu par `SPI_GETCLIENTAREAANIMATION` au démarrage. |
| Ressort | `.with_spring(id, SpringAnimation::new(config).to(cible), …)` | Statique quand `reduce_motion` est actif. |
| Sélecteur GPUI | `cx.prompt_for_paths(PathPromptOptions)` | Réponse asynchrone. `prompt` nomme le bouton de validation. |
| Sélecteur Win32 | `src/windows/file_dialog.rs::choose_executable` | Sur un thread dédié ; HWND obtenu par `raw-window-handle`. |
| DPI par écran | `WM_DPICHANGED` traité par `gpui-pre-windows` (`events.rs`) | Vérifié en T048. |
| Icône de notification | `tray-icon` 0.26.1 et son menu `muda` 0.21.2 | Événements lus toutes les 100 ms. |

## Manques relevés

- **Annonce d'un statut sans déplacer le focus.** GPUI 0.3.9 n'expose aucune région active
  (`aria-live`). Vérifié en T048 : un `Role::Status` seul ne lève qu'un `NameChanged`, que le
  Narrateur ne lit pas. Deux contournements fonctionnent. Le crochet `a11y_synthetic_children`
  pose `accesskit::Live` sur le nœud. La sonde montre l'autre : un fournisseur UIA minimal, côté
  serveur, dont l'hôte est le HWND, lève `UiaRaiseNotificationEvent`. Levée sur le fournisseur
  hôte rendu par `UiaHostProviderFromHwnd`, la notification n'atteint aucun client.
- **Position dans un ensemble.** `aria_position_in_set` compte à partir de 0 et la taille se pose
  sur le conteneur : `accesskit_windows` ajoute 1 et lit `size_of_set` sur les parents.
- **Coût mémoire.** GPUI occupe 59,7 Mio de mémoire privée sans aucune fenêtre ; voir D17.
- **Fuite à la fermeture par `WM_CLOSE`.** La fenêtre garde son rendu DirectX. `ProbeView::new`
  refuse la fermeture native par `on_window_should_close` et appelle `remove_window()`.
- **Filtre et titre du sélecteur GPUI.** `PathPromptOptions` ne permet ni de limiter le choix aux
  `.exe` ni de donner un titre : `prompt` nomme seulement le bouton de validation. Le sélecteur Win32
  existant garde son filtre et son titre ; il reste le choix par défaut de T059.
- **Sélecteur Win32 et boucle GPUI.** `choose_executable` ne doit pas être appelé directement depuis
  un écouteur GPUI. Sa boucle modale distribue les messages de la fenêtre propriétaire, donc les
  tâches GPUI, pendant que `App` est emprunté : une tâche qui appelle `cx.update` panique. GPUI
  ouvre ses propres dialogues sur un thread séparé pour cette raison. La sonde fait de même, et
  T059 doit suivre ce modèle.
- **Changement d'« Effets d'animation » en cours d'exécution.** La sonde lit le réglage au démarrage
  seulement. T052 relira le réglage à chaque `WM_SETTINGCHANGE`.
- **Événements de l'icône.** `tray-icon` publie ses événements sur des canaux globaux. La sonde les
  lit toutes les 100 ms depuis la boucle GPUI. T055 jugera si ce délai convient.

## Licences publiées

| Paquet | Version | Licence |
| --- | --- | --- |
| `gpui-pre` et les 13 paquets `gpui-pre-*` du graphe Windows | 0.3.9 | Apache-2.0 |
| `accesskit` | 0.24.1 | MIT OR Apache-2.0 |
| `accesskit_windows` | 0.34.0 | MIT OR Apache-2.0 |
| `tray-icon` | 0.26.1 | MIT OR Apache-2.0 |
| `muda` | 0.21.2 | Apache-2.0 OR MIT |
| `image` | 0.25.10 | MIT OR Apache-2.0 |
| `futures` | 0.3.34 | MIT OR Apache-2.0 |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib |

`Cargo.lock` liste aussi des paquets `gpui-pre-*` pour d'autres plateformes (macOS, web) ; ils ne
sont pas compilés pour Windows et leurs licences n'ont pas été vérifiées ici.
Aucune de ces licences n'impose de copyleft. Les notices seront régénérées en T071.
