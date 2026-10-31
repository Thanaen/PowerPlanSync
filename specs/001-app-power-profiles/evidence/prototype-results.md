# Premier lot de faisabilité

Date : 20 septembre 2026. Code intégré sur `main` dans `114ae26` puis `9fc0ff4`.
Cette livraison contient des sondes et des adaptateurs. Elle ne contient pas encore le moteur
de règles, la configuration persistante ni le journal de récupération du produit.

## Vérifications exécutées

- Build release des trois exemples, formatage, Clippy sans avertissement et quatre tests Rust réussis.
- Énumération des plans, détection du témoin, instances multiples, homonyme et lanceur restant ouvert.
- Trois cycles réels Haute performance puis Utilisation normale. Réglage initial restauré.
- Refus d'un second contrôleur dans un autre processus du même compte.
- Inspection native de l'interface, saisie, Tabulation, case activée avec Espace et sélection de plan avec les flèches puis Entrée.
- Captures à facteurs Slint 1, 1,5 et 2, sans changer les paramètres d'affichage Windows.
- Masquage de la fenêtre avec maintien du processus, puis mesure au repos dans les deux états.

Les quatre tests Rust couvrent les conversions d'identité et les comparaisons de noms/chemins.
Ils ne remplacent pas les futurs tests du moteur et de récupération. La racine a réexécuté les
contrôles dans le worktree de construction après son transfert, puis vérifié l'identité des
sources avec le checkout principal. Les sorties sont dans [validation/](validation/).

Les premiers écrans débordaient horizontalement. Le titre, la largeur du contenu et les boutons
ont été corrigés, puis vérifiés dans Windows. La hauteur initiale est passée à 520 unités.
Voir [avant correction](accessibility/prototype-v1-before.png),
[écran corrigé](accessibility/prototype-v3-normal.png) et [arbre accessible](accessibility/prototype-v3-tree.txt).

## Première mesure de consommation

Machine Ryzen 7 9800X3D, 16 processeurs logiques, Windows 11 25H2 build 26200.9457.
Mesure de 60 secondes par état, échantillon chaque seconde, CPU normalisé sur toute la machine.
L'interface est seule ; aucun moteur de surveillance n'existe encore dans ce processus.

| État | CPU moyen total machine | Mémoire privée maximale | Working set maximal |
| --- | --- | --- | --- |
| Fenêtre ouverte | 0,0469 % | 5,406 Mio | 26,500 Mio |
| Fenêtre détruite, icône active | 0,0000 % mesuré | 3,691 Mio | 25,230 Mio |

Les données et rapports sont dans [performance/prototype-window/](performance/prototype-window/)
et [performance/prototype-tray/](performance/prototype-tray/).
Zéro mesuré est un résultat arrondi sur cette minute, pas une garantie d'absence de travail.
Les campagnes de trente minutes, vingt règles et cent cycles de fenêtre restent à faire sur le produit.

## Critères encore ouverts

- Narrateur, annonces de statut et focus accessible. Le focus visuel se déplace, mais le pilote
  d'inspection continue à signaler le champ de saisie. Le défaut peut provenir de l'extracteur
  ou de l'intégration accessible. Aucun succès Narrateur n'est déclaré.
- Thème clair, thème à contraste élevé, texte agrandi et changement d'écran/DPI Windows réel.
- Réouverture par le menu de l'icône. Le pilote n'expose pas la barre des tâches dans cette session.
  Le masquage a été exercé ; la réouverture par l'utilisateur reste à vérifier.
- Mesures prolongées, ETW, sessions multiples et reprise de veille.

Les captures [100 %](accessibility/prototype-scale-100.png),
[150 %](accessibility/prototype-scale-150.png) et [200 %](accessibility/prototype-scale-200.png)
prouvent le rendu à ces facteurs propres au processus, pas le passage entre écrans Windows.
Tous les processus créés pour les essais ont été arrêtés.

## Décision

Les appels Windows et la détection du témoin sont faisables sur cette machine sans élévation.
Slint reste le candidat retenu. Son accessibilité complète n'est pas encore validée.
T001, T002, T003 et T005 sont terminées. T004 et T006 restent ouvertes ; le passage au moteur
complet n'est pas déclaré acquis. Les réserves ne sont pas effacées pour annoncer une version distribuable.

Binaires inspectés et exécutés, SHA-256 :

```text
ui_probe.exe       98FA45B0DD0AC07457C6D0690547A476C4499041F2A599D9C3B0B4FD70181EDC
windows_probe.exe  07A88A5107CAA9F9571FA20AEE65948FEAC6179252A36BB3A38764B6CA82BA08
demo_target.exe    DD74F02F001D2FDACD727174EED5D8F7703037B4CC8F30CD0905C0E6C860438C
```

## Sonde GPUI sur Windows réel (T048)

Date : 31 octobre 2026. Machine de référence : Ryzen 7 9800X3D, 16 processeurs logiques, 32 Go,
GeForce RTX 3080 et Radeon intégrée, Windows 11 Pro 26H2 build 26300.9711, un seul écran
2560 × 1440 réglé à 125 %. Binaires release compilés sur la machine avec Rust 1.98.1 et MSVC :

```text
gpui_probe.exe       CDFBC9D6956C70CD05F13E9377EC42D9B17BEAE15C281AB3B2DC79E2806F61B9 (sonde finale ; les essais d'accessibilité précèdent les modes de fermeture)
power-plan-sync.exe  mesuré : voir app-gpui-window-states.json (champ sha256)
```

Aucun plan Windows n'a été modifié : la sonde n'en touche aucun et l'application tournait en
`--demo` avec un dossier isolé. Le plan actif a été relu identique après chaque mesure.
Battlefield n'a pas été lancé. Les preuves sont dans [accessibility/gpui/](accessibility/gpui/)
et [performance/gpui-20261031/](performance/gpui-20261031/).
Les mesures de coût portent sur l'application de `main`. Les captures et journaux `app-*` viennent
de l'application avec les correctifs d'interface du même jour (focus, régions actives et bandeau
de démonstration), livrés séparément.

### Méthode

Le pilotage n'utilise pas le plugin Computer Use de Codex, absent de cette session. Il passe par
des scripts du dépôt, en lecture d'UI Automation, et par `agent-desktop` pour les captures :
`scripts/uia-inspect.ps1` (arbre et capture), `uia-events.ps1` (régions actives, notifications,
changements de nom et de focus), `uia-focus.ps1`, `uia-act.ps1`, `send-keys.ps1` (touches par
`SendInput`), `windows-appearance.ps1` et `display-scale.ps1` (thème, contraste et échelle,
rétablis dans un bloc `finally`). Ce que le Narrateur a réellement prononcé est lu dans sa
fenêtre **Récapitulatif de la reconnaissance vocale** (Insert + Alt + X), pas déduit de l'arbre.

### Accessibilité

| Contrôle | Résultat | Preuve |
| --- | --- | --- |
| Arbre UI Automation | Rôles et noms publiés ; l'interrupteur est un bouton à bascule (« Suivre, Bouton, activé ») | `probe-tree-after-fix.txt` |
| Position dans une liste | **Défaut corrigé** : la sonde passait `index + 1` et le Narrateur aurait dit « 2 sur ? » pour le premier élément. AccessKit compte à partir de 0 et lit la taille sur le conteneur. Après correction, le Narrateur dit « 2. Blender, 2 sur 3, sélectionné » | `probe-tree-before-fix.txt`, `probe-narrator-recap-positions.png` |
| Clavier | Tab et Maj+Tab parcourent boutons, interrupteur et éléments de liste | idem |
| Statut sans déplacer le focus | **Échec sans contournement** : un `Role::Status` sans région active ne lève qu'un `NameChanged`, que le Narrateur ne lit pas | `probe-events-status-without-live.txt` |
| Contournement 1 : région active | Le crochet `a11y_synthetic_children` pose `Live::Polite` sur le nœud ; AccessKit lève alors `LiveRegionChanged`. L'application l'utilise pour ses annonces | journaux d'événements de l'application |
| Contournement 2 : notification UIA | Un fournisseur UIA minimal dont l'hôte est le HWND lève `UiaRaiseNotificationEvent` ; le Narrateur a dit « Annonce de test 3 » et « Annonce de test 4 » sans déplacement du focus. Levée sur le seul fournisseur hôte de `UiaHostProviderFromHwnd`, la notification n'atteint aucun client | `probe-events-uia-notification.txt`, `probe-narrator-recap-notifications.png` |
| Échelles 100, 150 et 200 % | Changement d'échelle de l'écran pendant l'exécution : la sonde et l'application se redessinent nettes, sans débordement | `probe-scale-*.png`, `app-scale-*.png` |
| Thèmes clair, sombre et contraste élevé | Bascule pendant l'exécution, suivie par l'application ; le contraste élevé remplace les jetons par les couleurs système. La sonde a des couleurs fixes et ne sert pas à ce contrôle | `app-theme-*.png` |
| Texte agrandi | **Non pris en charge** : ni GPUI ni `src/interface/` ne lisent la taille du texte de Windows, que l'interface Slint appliquait (`text_scale_percent`). À traiter en T068 | recherche dans les sources |
| Changement d'écran | **Non exécuté** : un seul écran sur la machine | — |
| « Effets d'animation » désactivé | **Non exécuté** sur la sonde, qui lit le réglage au démarrage seulement | — |

### Coût

Mesure par `scripts/measure-prototype.ps1` : échantillon chaque seconde, CPU normalisé sur les 16
processeurs logiques, mémoire privée `PrivateMemorySize64`. Dix secondes de stabilisation avant
chaque état. Une minute par état, donc pas la campagne de trente minutes de SC-004 (T039).

| Processus et état | CPU moyen machine | Mémoire privée max | Handles | Threads |
| --- | ---: | ---: | ---: | ---: |
| Sonde, jamais de fenêtre (`--tray-only`, 30 s) | 0,0065 % | 59,7 Mio | 454 | 44 |
| Application `--gpui --demo`, fenêtre ouverte | 0,0906 % | 92,5 Mio | 654 | — |
| Application `--gpui --demo`, fenêtre fermée | 0,0178 % | **91,1 Mio** | 649 | 64 |
| Rappel : application Slint, fenêtre fermée, 20 règles (20 sept.) | 0,0277 % | 3,1 Mio | 189 | — |

SC-004 fixe 100 Mio fenêtre ouverte et **40 Mio fenêtre fermée**. Fenêtre ouverte, l'application
GPUI tient le budget avec 7,5 Mio de marge. Fenêtre fermée, elle le dépasse de 51 Mio : fermer
la fenêtre ne libère presque rien. La sonde sans aucune fenêtre occupe déjà 59,7 Mio. Le coût
vient donc de l'initialisation de GPUI (rendu DirectX, polices, exécuteurs), pas des vues de
PowerPlanSync. Les pilotes graphiques comptent dans la mémoire privée ; une machine sans carte
dédiée donnerait un autre chiffre, non mesuré ici.

### Cent cycles de fenêtre et réouvertures

`scripts/verify-window-lifecycle.ps1 -ExecutablePath gpui_probe.exe -Arguments --lifecycle` suit le
protocole de la sonde Slint : une ouverture d'échauffement, 100 cycles entre deux marqueurs,
101 ouvertures et 101 fermetures comptées. Le marqueur de départ est relevé sans fenêtre, 200 ms
avant la première ouverture. Les premiers essais le relevaient pendant cette ouverture et
donnaient à tort une baisse de mémoire.

| Fermeture des 100 fenêtres | Mémoire privée, marqueur de départ → fin | Handles | Preuve |
| --- | --- | --- | --- |
| `remove_window()` | 112,3 → 135,4 Mio (+23,2) | 544 → 632 (+88) | `probe-lifecycle-remove-window.json` |
| `WM_CLOSE`, comme le × de la barre de titre | **+1 312 Mio** | **+887** | `probe-lifecycle-wm-close-raw.json` |
| `WM_CLOSE` refusé par `on_window_should_close`, puis `remove_window()` | +24,1 Mio | +88 | `probe-lifecycle-wm-close-guarded.json` |

Avec `remove_window()`, la courbe échantillonnée chaque seconde (`probe-lifecycle-run2-curve.csv`)
montre un palier atteint au premier cycle, puis des handles stables pendant les 100 cycles. Les
deux cas dépassent le seuil du protocole Slint (5 Mio, 10 handles). **Fermée par `WM_CLOSE`,
une fenêtre GPUI 0.3.9 garde son rendu** : environ 13 Mio de mémoire privée par fermeture dans la
sonde. Les compteurs « GPU Process Memory » de l'application montrent 12,6 Mo de mémoire GPU
dédiée en plus à chaque ouverture, jamais rendus. Refuser la fermeture native et appeler
`remove_window()` supprime cette fuite.

L'application de `main` se ferme par `WM_CLOSE` (× dessiné, Alt+F4). `scripts/measure-window-reopen.ps1`
l'ouvre et la ferme six fois dans le même processus, sans `--demo` et avec un dossier neuf
(automatisation désactivée, aucun plan changé) :

| Fenêtre fermée après l'ouverture n° | 1 | 2 | 3 | 4 | 5 | 6 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `main` | 90,8 Mio | 116,0 | 141,3 | 165,9 | 193,0 | 219,1 |
| avec `on_window_should_close` | 90,7 Mio | 97,5 | 97,3 | 98,1 | 97,3 | 97,4 |

La PR #30 (commit `8db9a95`) applique ce contournement à `MainWindow` ; la seconde ligne du
tableau vient de ce binaire. Sans elle, `main` fuit toujours à chaque fermeture.
Aucune boucle de rendu ne reste après la fermeture : le CPU fenêtre fermée est le plus bas mesuré.

### Conclusion

GPUI fonctionne sur Windows réel pour la fenêtre, l'icône, le clavier, l'échelle, les thèmes et
le Narrateur, avec des contournements documentés pour les annonces et pour la fuite à la
fermeture. SC-004 échoue fenêtre fermée, même sans fuite. La décision revient au responsable produit : voir D17 dans [research.md](../research.md).
Décision du 2026-10-31 : option 1, la fenêtre dans un processus séparé. Voir la section suivante
et D18.

## Processus de suivi sans GPUI (T073)

Date : 31 octobre 2026, même machine de référence que la section précédente. La question de T073
est de faisabilité : un processus de suivi sans GPUI tient-il les 40 Mio de SC-004 fenêtre fermée ?

Le prototype est le correctif [prototype.patch](performance/tracker-20261031/prototype.patch),
appliqué au commit `107ffbf` et non fusionné. Il ajoute `src/tracker.rs` : l'icône `tray-icon` de
`src/interface/tray.rs` sur une boucle de messages Win32 du thread principal, sans aucune fenêtre
GPUI. Le coordinateur, son thread et la fenêtre cachée des notifications sont ceux de
`src/runtime.rs`. La variable `POWERPLANSYNC_TRACKER_PROBE` choisit ce chemin avec `--gpui`. Le
binaire release est le même que celui de l'interface GPUI : GPUI est lié, mais pas initialisé.

Conditions de SC-004 : vrai mode, pas `--demo`, car la démonstration ne relève pas les processus.
Dossier de données neuf avec la configuration de 20 règles de la mesure Slint du 20 septembre.
Ses exécutables témoins `TARGET-01.EXE` à `TARGET-20.EXE` ne tournaient pas : aucune bascule
n'était possible. 225 processus sur la machine. Automatisation active, relevé chaque seconde.
Lancement `--gpui --start-minimized --data-dir <dossier>`, 10 s de stabilisation, puis
`scripts/measure-prototype.ps1` pendant 60 s. La fenêtre `tray_icon_app` du processus confirme
l'icône. Le plan Haute performance a été rétabli puis relu dans un bloc `finally`.

| Processus et état | CPU moyen machine | Mémoire privée max | Working set max | Handles | Threads |
| --- | ---: | ---: | ---: | ---: | ---: |
| Prototype de suivi, icône seule, 20 règles | 0,0291 % | **3,6 Mio** | 16,7 Mio | 201 → 200 | 6 |
| Rappel : GPUI dans le même processus, fenêtre fermée (`--demo`) | 0,0178 % | 91,1 Mio | — | 649 | 64 |
| Rappel : application Slint, fenêtre fermée, 20 règles (20 sept.) | 0,0277 % | 3,1 Mio | — | 189 | — |

Données : [tracker-live-20-rules.json](performance/tracker-20261031/tracker-live-20-rules.json) et
[tracker-live-20-rules-samples.csv](performance/tracker-20261031/tracker-live-20-rules-samples.csv).

**Conclusion.** Le prototype du processus de suivi sans GPUI occupe 3,6 Mio, soit 36 Mio de marge
sous SC-004. Le CPU du relevé (0,029 % sur 60 s) est proche de celui de la version Slint (0,028 %,
sur une autre durée). La faisabilité de l'option 1 de D17 est
établie ; D18 en décrit la conception. Ce n'est pas la mesure de 30 minutes de SC-004 (T039), ni
une mesure fenêtre ouverte, qui exige le processus de fenêtre (T077, T081).
