# Sonde GPUI (T047)

La sonde `examples/gpui_probe.rs` vérifie que GPUI 0.3.9 (paquet `gpui-pre`) fournit les API
dont l'interface a besoin. Elle ne change aucun réglage Windows et n'utilise aucune donnée réelle.

```powershell
cargo run --locked --release --example gpui_probe
cargo run --locked --release --example gpui_probe -- --smoke
```

`--smoke` ouvre la fenêtre, la détruit, la rouvre, la détruit de nouveau puis quitte.
Sans option, la sonde reste dans la zone de notification après la fermeture de la fenêtre.
**Ouvrir la sonde** ou un clic gauche sur l'icône rouvre la fenêtre ; **Quitter la sonde** arrête tout.

## État

La sonde compile pour `x86_64-pc-windows-msvc` : `cargo clippy --locked --all-targets
--target x86_64-pc-windows-msvc -- -D warnings` réussit le 26 octobre 2026, dans le conteneur Linux
de la routine. **Elle n'a pas encore été exécutée.** Les comportements ci-dessous sont confirmés
dans les sources de GPUI, pas sur un bureau Windows. T048 les vérifie sur Windows réel.

## API confirmées dans les sources

| Besoin | API utilisée | Remarque |
| --- | --- | --- |
| Fenêtre créée à la demande | `cx.open_window(WindowOptions, …)` | Renvoie un `WindowHandle`. |
| Fenêtre détruite à la fermeture | `window.remove_window()` | La vue est libérée avec la fenêtre. |
| Application gardée après fermeture | `Application::with_quit_mode(QuitMode::Explicit)` | Arrêt par `cx.quit()`. |
| Barre de titre dessinée | `TitlebarOptions { appears_transparent: true }` | Zones par `window_control_area(Drag / Close)`. |
| Rôles et noms accessibles | `.role(Role::…)`, `.aria_label(…)` | Rôles AccessKit, adaptateur `accesskit_windows` 0.34. |
| Interrupteur | `Role::Switch`, `.aria_toggled(Toggled::…)` | |
| Liste | `Role::List`, `Role::ListItem`, `aria_position_in_set`, `aria_size_of_set`, `aria_selected` | |
| Statut | `Role::Status` | Voir les manques. |
| Ordre de tabulation | `.focusable()`, `.tab_stop(true)`, `window.focus_next` | Liés à Tab et Maj+Tab. |
| Mouvement réduit | `cx.set_reduce_motion(…)` | Lu par `SPI_GETCLIENTAREAANIMATION` au démarrage. |
| Ressort | `.with_spring(id, SpringAnimation::new(config).to(cible), …)` | Statique quand `reduce_motion` est actif. |
| Sélecteur GPUI | `cx.prompt_for_paths(PathPromptOptions)` | Réponse asynchrone. |
| Sélecteur Win32 | `src/windows/file_dialog.rs::choose_executable` | HWND obtenu par `raw-window-handle`. |
| Icône de notification | `tray-icon` 0.26.1 et son menu `muda` 0.21.2 | Événements lus toutes les 100 ms. |

## Manques relevés

- **Annonce d'un statut sans déplacer le focus.** GPUI 0.3.9 n'expose aucune région active
  (`aria-live`) ni aucune annonce explicite. `Role::Status` est publié, mais rien ne garantit que
  le Narrateur lise un changement de texte. T048 doit le vérifier. En cas d'échec, il faudra un
  contournement dans `src/interface/`, par exemple `UiaRaiseNotificationEvent` sur le HWND.
- **Filtre de type de fichier.** `PathPromptOptions` ne permet pas de limiter le choix aux `.exe`.
  Le sélecteur Win32 existant garde ce filtre ; il reste le choix par défaut de T059.
- **Changement d'« Effets d'animation » en cours d'exécution.** La sonde lit le réglage au démarrage
  seulement. T052 relira le réglage à chaque `WM_SETTINGCHANGE`.
- **Événements de l'icône.** `tray-icon` publie ses événements sur des canaux globaux. La sonde les
  lit toutes les 100 ms depuis la boucle GPUI. T055 jugera si ce délai convient.

## Licences publiées

| Paquet | Version | Licence |
| --- | --- | --- |
| `gpui-pre` et ses 14 paquets `gpui-pre-*` | 0.3.9 | Apache-2.0 |
| `accesskit` | 0.24.1 | MIT OR Apache-2.0 |
| `accesskit_windows` | 0.34.0 | MIT OR Apache-2.0 |
| `tray-icon` | 0.26.1 | MIT OR Apache-2.0 |
| `muda` | 0.21.2 | Apache-2.0 OR MIT |
| `image` | 0.25.10 | MIT OR Apache-2.0 |
| `futures` | 0.3.34 | MIT OR Apache-2.0 |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib |

Aucune de ces licences n'impose de copyleft. Les notices seront régénérées en T071.
