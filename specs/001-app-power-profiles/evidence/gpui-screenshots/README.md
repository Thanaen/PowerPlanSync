# Captures GPUI hors écran

Rendues sous Linux par `src/bin/screenshots.rs` (T057), à l'échelle 2, avec lavapipe. Sous
Debian ou Ubuntu, le build demande les en-têtes de fontconfig et le rendu un pilote Vulkan :

```sh
sudo apt-get install libfontconfig-dev mesa-vulkan-drivers
cargo run --features screenshots --bin screenshots -- specs/001-app-power-profiles/evidence/gpui-screenshots
```

La page **Logiciels** est en place (T058), avec ses dialogues **Ajouter un logiciel** et **Modifier** (T059) « Pour commencer » avant la première activation (T060) et ses messages intégrés (T062). Les pages **Réglages** et **Aide et limites** sont en place (T065). Les
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
| `external.png` | `External.png` | Conforme, sauf l'en-tête, qui garde le mot-clé « Automatisation suspendue ». |
| `refused.png` | `Error.png` | Conforme ; en-tête « Automatisation suspendue · Windows garde le plan Utilisation normale ». |
| `refused-light.png` | aucune | Mêmes constats en thème clair. |
| `quit-failed.png` | `QuitFailed.png` | Conforme ; en-tête « Automatisation suspendue · retour échoué, le plan Haute performance reste actif ». |
| `quit-failed-narrow.png` | aucune | Fenêtre étroite : les trois actions passent l'une sous l'autre. |
| `removed.png` | `Idle.png` | Conforme : message de réussite sur une ligne, **Annuler** (Ctrl+Z) et × à droite. |
| `removed-light.png` | aucune | Même message en thème clair. |
| `removed-last.png` | aucune | Le seul logiciel suivi retiré : « Pour commencer » revient, **Annuler** reste sous les étapes. |
| `removed-narrow.png` | aucune | Fenêtre étroite : **Annuler** passe sous le texte. |
| `settings.png` | `Settings.png` | Conforme, sauf la description de **Dossier des données**, qui ne cite pas de chemin (écart noté dans `design/README.md`). Le message du retour réservé au cycle en cours s'affiche sous **Plan de retour**. |
| `settings-light.png` | aucune | Mêmes constats en thème clair. |
| `settings-idle.png` | aucune | Automatisation en pause : pas de message sous **Plan de retour**. |
| `settings-narrow.png` | aucune | Fenêtre étroite : la liste et **Ouvrir le dossier** passent sous leur texte, les interrupteurs restent à droite. |
| `settings-refused.png` | aucune | Réglage refusé (**Démarrer avec Windows** avec `--data-dir`) : le message d'erreur s'affiche en tête de page. |
| `settings-read-only.png` | aucune | Contrôle tenu par une autre session : le message de la page **Logiciels**, puis **Données** seulement. Les valeurs non lues ne sont pas montrées. |
| `settings-read-only-unreadable.png` | aucune | Réglages illisibles : le message **Les réglages sont illisibles** seul, qui ouvre déjà le dossier des données. |
| `settings-unreadable.png` | `ConfigError.png` | Conforme, sauf la barre d'état, qui garde le mot-clé « Automatisation arrêtée ». |
| `settings-unreadable-light.png` | aucune | Mêmes constats en thème clair. |
| `help.png` | `Help.png` | Conforme, sauf la section **Limites**, qui précise que les plans s'affichent sous leur nom dans Windows, et « conserve ce choix » au lieu de « le garde » (écarts notés dans `design/README.md`). Comme dans la maquette, les boutons passent sous la barre d'état et la page défile. |
| `help-light.png` | aucune | Mêmes constats en thème clair. |
| `help-narrow.png` | aucune | Fenêtre étroite : le texte se replie, les deux boutons tiennent sur une ligne. |
| `unavailable-dark.png` | `Unavailable.png` | Maquette tirée de cette capture. |
| `unavailable-light.png` | aucune | Mêmes constats en thème clair. |
| `add-rule.png` | `AddRule.png` | Conforme, sauf le nom du processus (`bf6.exe`) dans la liste : Windows ne donne pas le nom du produit. |
| `add-rule-light.png` | aucune | Mêmes constats en thème clair. |
| `add-rule-invalid.png` | `AddRuleInvalid.png` | Entrée sur un dialogue vide : chaque champ dit quoi corriger. La maquette montre le cas du plan disparu, signalé de la même façon sous **Plan pendant l'exécution**. |
| `edit-rule.png` | `EditRule.png` | Conforme. |
| `first-launch.png` | `FirstLaunch.png` | Conforme : plan normal proposé, **Activer l'automatisation** indisponible. |
| `first-launch-missing.png` | `FirstLaunchMissing.png` | Conforme : aucun plan présélectionné, **Confirmer** indisponible. |
| `first-launch-ready.png` | `FirstLaunchReady.png` | Conforme : étapes 1 et 2 cochées, pastilles au repos. |
| `first-launch-unfollowed.png` | aucune | Seul logiciel non suivi : l'automatisation s'est arrêtée, les étapes reviennent et la liste reste dessous pour le suivre de nouveau. |
| `first-launch-light.png` | aucune | Mêmes constats en thème clair. |
| `first-launch-narrow.png` | aucune | Fenêtre étroite : chaque commande passe sous le texte de son étape. |

Aucun écart hors de ceux listés dans `design/README.md`.
