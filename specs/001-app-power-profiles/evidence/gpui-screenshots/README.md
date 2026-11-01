# Captures GPUI hors écran

Rendues sous Linux par `src/bin/screenshots.rs` (T057), à l'échelle 2, avec lavapipe. Sous
Debian ou Ubuntu, le build demande les en-têtes de fontconfig et le rendu un pilote Vulkan :

```sh
sudo apt-get install libfontconfig-dev mesa-vulkan-drivers
cargo run --features screenshots --bin screenshots -- specs/001-app-power-profiles/evidence/gpui-screenshots
```

La page **Logiciels** est en place (T058), avec ses dialogues **Ajouter un logiciel** et **Modifier** (T059) « Pour commencer » avant la première activation (T060) et ses messages intégrés (T062). Les pages **Réglages** et **Aide et limites** sont en place (T065). La palette de commandes (T066) s'ouvre par-dessus la page **Logiciels**. Les
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
| `palette.png` | `Palette.png` | Conforme : cinq entrées, raccourcis en capuchons, la page reste lisible sans voile. |
| `palette-light.png` | aucune | Mêmes constats en thème clair. |
| `palette-filtered.png` | aucune | Recherche « ou » : les verbes qui commencent par la saisie d'abord, la partie trouvée en accent. |
| `palette-no-match.png` | aucune | Aucune entrée : « Aucune commande ne correspond. », et le pied ne garde que **Échap** **Fermer**. |
| `palette-narrow.png` | aucune | Fenêtre étroite : sans le bouton **Commandes**, Ctrl+Maj+P ouvre la palette, centrée avec ses marges. |
| `demo.png` | `Demo.png` | Badge « Démo » de la barre de titre conforme (T067). L'état reste celui de l'application : « Automatisation en attente » avec son cercle, le plan de retour « quand le dernier logiciel suivi se fermera » et « Sur secteur » sans « (simulé) », la source d'alimentation étant lue dans Windows. |
| `demo-light.png` | aucune | Mêmes constats en thème clair. |
| `keyboard-row.png` | aucune | Tab jusqu'à la première ligne, puis ↓ : la ligne Blender a le focus, anneau de 2 px tracé dans la ligne (T067). |
| `keyboard-row-light.png` | aucune | La première ligne focalisée en thème clair : l'anneau suit les coins arrondis de la liste. |

## Mouvement (T069)

`motion-*-000.png` est la première image d'une transition déclenchée comme à la souris,
`motion-*-050.png` la mi-course. L'image au repos est la capture sans préfixe.

| Capture | Maquette | Constat |
| --- | --- | --- |
| `motion-reorder-000.png`, `motion-reorder-050.png` | `Reorder-000.png`, `Reorder-050.png` | Conforme : Blender monte par-dessus Battlefield 6 avec une bordure de 1 px, sans ombre ; numéros et badges déjà à jour. À mi-course, la ligne qui descend a parcouru 30 px sur 54, un peu moins que la maquette : le ressort sans rebond ralentit tôt. |
| `motion-removed-000.png`, `motion-removed-050.png` | `Idle-000.png`, `Idle-050.png` | Conforme : le message glisse de 4 px sous la liste. Le bouton **Supprimer** de la ligne remontée reste survolé, là où le pointeur a cliqué. |
| `motion-add-rule-000.png`, `motion-add-rule-050.png` | `AddRule-000.png`, `AddRule-050.png` | Conforme : rien à la première image, puis voile et carte opaques, la carte à quelques pixels de sa place. La maquette ouvre le dialogue depuis « Pour commencer », la capture depuis la liste. |
| `motion-first-launch-ready-000.png`, `motion-first-launch-ready-050.png` | `FirstLaunchReady-000.png`, `FirstLaunchReady-050.png` | Conforme : la pastille de l'étape 2 part de rien et a presque atteint sa taille à mi-course. |

## Texte agrandi et contraste élevé (T068)

Rendus avec `TextScale` et `HighContrast` posés comme le ferait le processus de fenêtre. Les
couleurs système sont celles des thèmes « Ciel nocturne » et « Désert » de Windows 11. À 100 %,
toutes les captures ci-dessus sont restées identiques au pixel près. Aucune maquette.

| Capture | Constat |
| --- | --- |
| `text-125.png` | 880 × 620 à 125 % : barre latérale entière, lignes sur plusieurs niveaux pour garder le nom lisible. |
| `text-150.png` | 880 × 620 à 150 % : sous le seuil de la fenêtre étroite (900 px), barre latérale réduite aux icônes. |
| `text-200.png` | 880 × 620 à 200 % : même disposition que `Narrow.png`, le texte double, les icônes gardent leur taille. |
| `text-200-wide.png` | 1 280 × 720 à 200 % : barre latérale entière de 400 px, lignes sur plusieurs niveaux. |
| `text-200-narrow.png` | 420 × 760 à 200 % : capuchons des raccourcis masqués, barre d'état coupée par des points de suspension. |
| `text-200-settings.png` | **Réglages** à 200 % : la liste passe sous son texte, le message défile. |
| `text-200-first-launch.png` | « Pour commencer » à 200 %. |
| `text-200-refused.png` | Message d'erreur intégré à 200 %. |
| `text-200-removed.png` | Message de retrait annulable à 200 %. |
| `text-200-help.png` | **Aide et limites** à 200 % : la largeur de lecture suit la taille du texte, la page défile. |
| `text-200-palette.png` | Palette de commandes à 200 % : la recherche, le titre du groupe et les entrées suivent la taille du texte, la liste défile dans la fenêtre. |
| `text-200-add-rule.png` | **Ajouter un logiciel** invalide à 200 % : le corps du dialogue défile, le pied reste visible. |
| `text-200-add-rule-plan.png` | **Ajouter un logiciel** à 200 % dans une fenêtre de 880 px de large : la liste **Plan pendant l'exécution** s'aligne sur **Nom affiché** sans sortir du dialogue. |
| `contrast-dark.png` | Ciel nocturne : couleurs système, sans ombre ; l'état reste porté par l'icône et le mot. |
| `contrast-light.png` | Désert : mêmes constats sur fond clair. |
| `contrast-help.png` | **Aide et limites** en Ciel nocturne. |
| `contrast-settings.png` | **Réglages** en Ciel nocturne. |
| `contrast-refused.png` | Message d'erreur en Ciel nocturne : bord WindowText, sans fond teinté. |
| `contrast-add-rule.png` | **Ajouter un logiciel** en Désert, sans voile ni ombre. |
| `contrast-text-200.png` | Ciel nocturne avec texte à 200 %. |

Aucun écart hors de ceux listés dans `design/README.md`.
