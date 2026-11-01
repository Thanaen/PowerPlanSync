# Recherche et décisions de conception

Recherche effectuée le 2026-09-20. Les constats Windows sont locaux et en lecture seule.
Les choix ci-dessous définissent le point de départ des prototypes, pas leurs résultats.
Complétée le 2026-10-25 pour la migration de l'interface vers GPUI, décidée par le responsable
produit. D12 à D16 remplacent D05 pour l'interface. D01 à D04 et D06 à D11 restent valables.

## D01. Commencer par les plans Windows existants

**Décision.** La version initiale utilise les plans classiques, identifiés par leur GUID.
Elle présente les noms Windows et exige un plan de retour explicite.

**Raison.** Le PC inspecté possède déjà les plans nécessaires au scénario demandé.
Le plan actif était Haute performance. Restaurer seulement le plan initial ne satisferait donc
pas nécessairement le retour souhaité vers Utilisation normale.

**Alternatives.** Les modes d'alimentation modernes restent hors périmètre.
Microsoft distingue ces modes, anciennement overlays, des plans classiques.
Un changement de plan ne prouve pas le déplacement du sélecteur des Paramètres.
Voir [les modes Windows](https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nf-powrprof-powersetuserconfiguredacpowermode)
et [les overlays](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/customize-power-slider).

| Constat local | Valeur observée |
| --- | --- |
| Version Windows | 10.0.26200, version 25H2, révision 9457 |
| Plan normal | Utilisation normale, `381b4222-f694-41f0-9685-ff5bb260df2e` |
| Plan actif | Haute performance, `8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c` |
| Autres plans | Économie d'énergie et Performances optimales |
| Veille | S3 disponible, S0 basse consommation indisponible |
| Outils présents | Git, PowerShell, Python 3.11.15 et uv 0.12.5 |
| Rust | `cargo` et `rustc` absents du PATH, aucun binaire trouvé dans `.cargo/bin` |

Preuves locales dans `evidence/environment.txt`. Le nom ProductName du registre porte encore
« Windows 10 Pro ». Le numéro de build décrit mieux l'environnement observé. L'interrogation
CIM a été refusée dans le contexte d'exécution et n'est pas une preuve de privilèges du produit.

## D02. Utiliser les API Windows depuis Rust

**Décision.** Utiliser `windows` et les fonctions `PowerEnumerate`, `PowerReadFriendlyName`,
`PowerGetActiveScheme` et `PowerSetActiveScheme`. Libérer l'allocation retournée par la lecture
du plan actif avec `LocalFree`. Ne lancer ni PowerShell ni `powercfg` dans la boucle du produit.

**Raison.** Les appels documentés évitent un sous-processus et l'analyse d'un texte traduit.
`powercfg` reste un outil indépendant de vérification.

**Alternatives.** Les écritures directes au registre et la création automatique de plans sont exclues.
Les droits standards sont la cible. Les restrictions de stratégie ou d'ACL deviennent des erreurs
visibles. Elles ne déclenchent pas une demande d'élévation permanente.
Voir [gestion des plans](https://learn.microsoft.com/en-us/windows/win32/power/managing-power-schemes),
[énumération](https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nf-powrprof-powerenumerate),
[noms](https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nf-powrprof-powerreadfriendlyname),
[lecture](https://learn.microsoft.com/en-us/windows/win32/api/powersetting/nf-powersetting-powergetactivescheme),
[activation](https://learn.microsoft.com/en-us/windows/win32/api/powersetting/nf-powersetting-powersetactivescheme)
et [droits et stratégies](https://learn.microsoft.com/en-us/windows/win32/power/power-policy-settings).

## D03. Mesurer un suivi de processus simple avant de complexifier

**Décision.** Un snapshot Toolhelp par seconde, uniquement lorsque le suivi est utile.
Filtrer les noms candidats, puis vérifier session, heure de création et chemin complet.
N'ouvrir que les candidats, avec `PROCESS_QUERY_LIMITED_INFORMATION`.
Conserver les chaînes Windows en UTF-16 ou `OsString`, sans conversion UTF-8 destructive.
Comparer les chemins normalisés avec les règles Windows insensibles à la casse.
La résolution du chemin final traite liens et jonctions sans accepter une correspondance au nom seul.

**Raison.** L'approche ne demande pas un contrôleur de traces privilégié et reste vérifiable.
Son coût réel doit être mesuré. L'exécutable témoin valide la détection pendant cette réalisation ;
l'accès à Battlefield reste un éventuel essai distinct, conformément à D09.
Les instances sont identifiées par PID, heure de création et session, jamais par PID seul.

**Alternatives.** WMI ajoute COM et un fournisseur sans fournir directement un chemin fiable
dans l'événement de démarrage. ETW peut être étudié si le prototype dépasse le budget CPU.
Son accès et ses privilèges empêchent de le présenter comme une solution gratuite en complexité.
Le sondage peut manquer un processus plus court que son intervalle. Le cas du jeu accepte cette limite.
Le budget de retour part de la fermeture réelle. Un intervalle d'une seconde et trois secondes
de grâce peuvent consommer environ quatre secondes avant l'appel Windows. Le prototype doit
mesurer la marge restante jusqu'à confirmation, en variant la sortie entre deux observations.
La cible de cinq secondes n'est pas tenue pour acquise.
Voir [Toolhelp](https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot),
[chemin complet](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew),
[droits des processus](https://learn.microsoft.com/en-us/windows/win32/procthread/process-security-and-access-rights),
[WMI](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/krnlprov/win32-processstarttrace)
et [sessions ETW](https://learn.microsoft.com/en-us/windows/win32/etw/configuring-and-starting-a-systemtraceprovider-session).

## D04. Garder un seul décideur et un journal récupérable

**Décision.** Une machine d'état pure produit des intentions. Un seul coordinateur réalise
les lectures, l'enregistrement durable et les écritures Windows en série.
Le journal contient l'intention avant l'appel Windows, puis le résultat confirmé.
Une relecture protège chaque écriture contre un changement externe déjà observable.
Les notifications du plan actif accélèrent la détection. La relecture régulière reste la référence.

**Raison.** Il faut distinguer le plan initial, le retour demandé et le dernier plan appliqué.
Un cycle déjà sur Haute performance doit tout de même revenir au retour choisi.
Une erreur de stockage avant application interdit la bascule. Une erreur après application conserve
la dernière intention durable et suspend le moteur.

**Alternatives.** Restaurer aveuglément le plan initial, imposer le retour au repos ou reprendre
la main à chaque seconde sont rejetés. Un service de surveillance des crashs est différé.
`PowerSetActiveScheme` n'offre pas de comparaison et écriture atomiques. Deux changements très proches
ou un retour externe vers le même GUID peuvent rester indétectables. Aucun document ne promet
une restauration immédiate après un arrêt forcé.
Voir [notifications d'alimentation](https://learn.microsoft.com/en-us/windows/win32/power/registering-for-power-events)
et [arrêt de Windows](https://learn.microsoft.com/en-us/windows/win32/shutdown/shutting-down).

## D05. Valider Slint sur un petit parcours accessible

> Remplacée par D12 le 2026-10-25. Ce texte reste l'historique du premier lot et des preuves
> Slint conservées dans `evidence/`.

**Décision.** Premier candidat, Rust avec Slint et ses contrôles standards. La déclaration `.slint`
est compilée avec le binaire Rust. Le moteur reste indépendant de cette interface.
Créer les réglages à la demande et libérer la fenêtre à leur fermeture.
Évaluer `SystemTrayIcon` sur une version stable qui le fournit, sans supposer que la documentation
`latest` correspond à toutes les versions disponibles.

**Raison.** Slint documente les propriétés accessibles, le clavier et une icône de notification
qui n'a pas besoin de fenêtre propre. Cela correspond au parcours réduit du produit.
La prise en charge documentée ne remplace pas les essais Narrateur, contraste et DPI.
Voir [propriétés accessibles](https://docs.slint.dev/latest/docs/slint/reference/common/),
[clavier](https://docs.slint.dev/latest/docs/slint/reference/keyboard-input/focusscope/)
et [icône de notification](https://docs.slint.dev/latest/docs/slint/reference/window/systemtrayicon/).

| Candidat | Décision pour le prototype | Motif |
| --- | --- | --- |
| Slint | Premier essai | Interface déclarative compilée, accessibilité et notification documentées |
| egui avec AccessKit | Repli à comparer si Slint échoue | Intégration Rust, arbre accessible, notification à intégrer séparément |
| Contrôles Win32 depuis Rust | Repli ciblé | Accessibilité des contrôles standards, coût de maintenance du rendu et du placement |
| Tauri | Non retenu | Interface web et runtime WebView2 supplémentaires pour ce petit outil |
| iced | Non retenu à ce stade | Maturité de l'accessibilité à établir avant de satisfaire le périmètre |

Sources des alternatives, [egui](https://github.com/emilk/egui/blob/main/docs/accessibility.md),
[Win32 accessible](https://learn.microsoft.com/en-us/windows/win32/winauto/accessibility-best-practices),
[Tauri](https://github.com/tauri-apps/tauri) et [suivi iced](https://github.com/iced-rs/iced/issues/552).
La [licence Slint](https://github.com/slint-ui/slint/blob/master/LICENSE.md) propose plusieurs régimes.
Le prototype vérifie les conditions applicables et l'attribution. Aucune licence du projet ni
engagement commercial n'est décidé ici. Cette vérification précède toute distribution.

## D06. Réduire les choix du premier écran

**Décision.** Afficher le plan actif, son explication, la liste ordonnée des logiciels et un bouton
**Ajouter un logiciel**. Le plan de retour et l'option batterie sont dans les réglages.
Les erreurs disposent d'une action concrète. Aucun éditeur d'expressions ni tableau de compteurs
système n'est présent dans le parcours normal.

**Raison.** Le principe Experience First privilégie ici le parcours logiciel, plan, retour.
Le principe Prove It Works impose ensuite de vérifier ce parcours sur Windows.
Les seules alertes spontanées concernent une action nécessaire ou une erreur persistante.

**Alternatives.** Profils automatiques par jeu, réglages CPU, lancement de jeux, synchronisation cloud,
analyses de FPS et règles horaires ne sont pas nécessaires au scénario initial.

## D07. Traiter les sessions comme une limite de produit

**Décision.** Verrou de contrôle unique à l'échelle de la machine, créé sans élévation.
L'instance doit aussi appartenir à la session interactive active. Une perte de session restaure
au mieux et suspend le contrôle. L'autre compte ne récupère pas automatiquement le journal privé.
Un verrou inaccessible interdit les écritures et produit un message.

**Raison.** Plusieurs sessions peuvent voir des processus, mais le plan d'alimentation peut affecter
le même matériel. Un verrou uniquement par utilisateur laisserait deux comptes se battre.
Le prototype confirme les droits du verrou global et son comportement après abandon.
Voir [énumération des processus](https://learn.microsoft.com/en-us/windows/win32/procthread/process-enumeration)
et [notifications de session](https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtssession-change).

## D08. Démarrer avec spec-kit officiel et conserver ses modèles

**Décision.** `specify-cli` 1.0.8 installé localement depuis le tag officiel `v1.0.8`, commit
`0cc9a6a1159471a3108b9bad718ba17006dd6039`. Intégration Codex en skills et scripts PowerShell.
Les artefacts suivent les résolveurs de modèles fournis. Aucune extension tierce n'est installée.

**Raison.** La conception est reproductible et indépendante d'un workflow pstack parallèle.
Cette version suit la fonctionnalité dans `.specify/feature.json` sans imposer une branche Git.
Voir [installation officielle](https://github.github.com/spec-kit/installation.html),
[intégration Codex](https://github.github.com/spec-kit/reference/integrations.html)
et [release utilisée](https://github.com/github/spec-kit/releases/tag/v1.0.8).

## État des vérifications de faisabilité

| Vérification | Critère de décision | État |
| --- | --- | --- |
| API et droits standards | Plans identiques à powercfg, bascule et lecture concordantes, refus maîtrisé | Trois cycles sous compte non élevé, GUID confirmés ; voir D10 |
| Détection de l'exécutable témoin | Chemin exact, lanceur séparé, instances et homonyme reconnus | Réussi, sans Battlefield ; voir D10 |
| Interface Slint | Parcours clavier et Narrateur complet, contraste et DPI conformes | Partiel, Narrateur et contrastes ouverts ; remplacée par GPUI, voir D12 |
| Interface GPUI | Même critère, plus tray, mouvement réduit et budgets SC-004 et SC-007 | Non exécuté ; sonde prévue en T047 et T048 |
| Coût du moteur et de la fenêtre | Budgets SC-004 et SC-007, mesures brutes conservées | Interface seule mesurée une minute par état ; moteur et campagnes longues à faire |
| Session et verrou global | Une seule instance écrit, abandon récupérable sans élévation | Refus interprocessus réussi ; sessions et récupération à faire |
| Récupération interrompue | Chaque coupure entre journal et écriture possède une issue sûre documentée | Non exécuté |

Ces éléments deviennent les premières tâches. Ils ne sont pas des clarifications produit cachées.
En cas d'échec, mettre à jour la décision et les tâches avant d'étendre l'implémentation.

## D09. Tester avec un exécutable témoin léger

L'utilisateur demande au démarrage de l'implémentation de ne pas lancer Battlefield 6.
Le témoin Rust teste les vrais processus Windows et les vrais plans, sans charge de jeu.
Un processus lanceur distinct reste ouvert après la fermeture de l'enfant configuré.
La règle distingue aussi deux copies homonymes par leur chemin. Les erreurs d'accès liées aux
processus protégés seront testées par injection, sans annoncer une compatibilité anti-triche.
Ce changement remplace les gates Battlefield dans T005, T041 et SC-006.

## D10. Résultats du premier lot de faisabilité

Les preuves du 20 septembre 2026 sont dans [prototype-results.md](evidence/prototype-results.md).
Les appels de plans et la détection du témoin fonctionnent sous compte non élevé. Trois cycles
avec sondage d'une seconde et grâce de trois secondes respectent les budgets individuels.
Le p95 sur cent cycles reste à mesurer. La fenêtre seule consomme 5,406 Mio de mémoire privée
au maximum sur une minute ; après masquage, 3,691 Mio. Ce n'est pas la consommation du moteur futur.

Le premier chronométrage proposé observait trop vite et comptait la grâce depuis la sortie réelle.
Il a été rejeté à la relecture avant exécution. La version retenue reproduit le sondage prévu,
compte la grâce depuis l'absence observée et conserve les heures Windows comme origine des métriques.

Les contrôles Slint sont exposés et utilisables lors des essais de saisie, sélection et activation.
Deux défauts de disposition ont été corrigés puis recontrôlés. Le focus signalé par le pilote
reste ambigu. Le Narrateur, les contrastes et la réouverture par l'icône ne sont pas validés.
Slint reste candidat ; aucun échec certain justifiant un changement de framework n'est établi.
T004 et T006 restent ouvertes. L'ordre de poursuite est précisé par D11,
sans convertir des résultats inconnus en critères réussis.

## D11. Poursuivre les fondations pendant la validation d'accessibilité

L'objectif actif demande de poursuivre jusqu'à une v1 fonctionnelle. Le moteur pur, le journal
durable et les adaptateurs Windows ne dépendent pas du choix graphique. Les preuves de T002
et T005 permettent leur développement pendant les vérifications restantes de T004.
Les types et transactions suivent le contrat Rust déjà comparé et retenu.

La constitution exige la faisabilité d'accessibilité avant l'interface définitive. Cette règle
reste intacte. L'intégration graphique reste candidate tant que clavier, Narrateur et contrastes
ne sont pas vérifiés. Les critères SC-001 à SC-007 et les tâches de livraison ne changent pas.
Les tests de récupération accompagnent le coordinateur avant tout usage automatique réel.

## D12. Migrer l'interface vers GPUI en gardant le moteur

**Décision.** Le responsable produit a validé le 2026-10-25 la migration de l'interface vers GPUI,
avec les maquettes de [design/README.md](design/README.md) et le
[contrat d'interface](contracts/user-interface.md). Seule la couche d'interface est remplacée :
`src/ui.rs`, `ui/*.slint`, la boucle Slint de `src/runtime.rs` et l'icône Slint. Le domaine,
le moteur, le coordinateur, le stockage, la récupération, les adaptateurs Windows et la
présentation sans type Slint restent en place. La nouvelle couche vit dans `src/interface/`.
Le nom `gpui` est réservé à la dépendance pour éviter une ambiguïté de chemin Rust.

Les versions reprennent l'application de référence Bientôt, dépôt Thanaen/WorkTime. Les chemins
ci-dessous viennent de la branche `main` du clone local (commit `72d3f5b`, crates `worktime-*`)
ou de `origin/main` (commit `d698537`, crates renommées `bientot-*` par la PR #65 de WorkTime).

| Dépendance | Version retenue | Preuve dans Thanaen/WorkTime |
| --- | --- | --- |
| `gpui` | `=0.3.9`, paquet `gpui-pre` | `crates/worktime-app/Cargo.toml` |
| `gpui_platform` | `=0.3.9`, paquet `gpui-pre-platform` | `crates/worktime-app/Cargo.toml` |
| `gpui_wgpu` | `=0.3.9`, paquet `gpui-pre-wgpu`, Linux seulement, optionnel | fonctionnalité `screenshots` du même fichier |
| `tray-icon` | `0.26`, sans fonctionnalités par défaut, Windows seulement | même fichier ; `Cargo.lock` résout 0.26.1 |
| `image` | `0.25`, fonctionnalités `ico` et `png` | même fichier |
| `futures`, `raw-window-handle` | `0.3`, `0.6`, Windows seulement | même fichier |
| `windows` | `0.62.2`, déjà épinglé ici | `gpui-pre-windows` dépend de `windows 0.62.2` dans `Cargo.lock` |

**Raison.** Les maquettes utilisent le design system Bientôt GPUI et ses composants. Bientôt
utilise ces versions sur Windows avec une icône de notification, des fenêtres créées à la demande
et un arbre accessible. Slint n'avait pas d'échec certain (D10), mais garder Slint obligerait à
recréer ce design system dans un autre langage. Le retrait de Slint supprime aussi l'obligation
d'attribution `AboutSlint` notée dans [ui-probe.md](evidence/ui-probe.md).
La constitution admet une interface Rust sans navigateur embarqué. GPUI ne change pas ce point.

**Alternatives.** Garder Slint et imiter les maquettes : rejeté, deux design systems à maintenir.
egui, Win32 et iced : non réévalués, la décision produit porte sur GPUI. Une bascule d'un seul
bloc est rejetée : la couche GPUI grandit à côté de Slint derrière l'option de développement
`--gpui`, puis T071 retire Slint et cette option. Voir Complexity Tracking dans [plan.md](plan.md).

**Version de Rust.** PowerPlanSync épingle Rust 1.98.1. Bientôt compile `gpui-pre` 0.3.9 avec
Rust 1.97.0 (`rust-toolchain.toml`). T046 vérifie la compilation avec 1.98.1 par Clippy pour
`x86_64-pc-windows-msvc`. Un échec est consigné avant tout changement de version.

**Réseau.** Les fonctionnalités `usage-ping`, `updates` et `release` de Bientôt
(`crates/bientot-app/Cargo.toml`, `origin/main`) ne sont pas reprises : la constitution exclut
toute télémétrie et toute communication réseau. `gpui-pre` dépend de `gpui-pre-http-client`
(`Cargo.lock`) ; T043 vérifie qu'aucune connexion n'est ouverte par l'application.

**Licence.** La licence publiée de `gpui-pre`, `gpui-pre-platform` et de leurs dépendances n'a
pas été vérifiée hors ligne. T044 la vérifie avant toute distribution, comme pour Slint.

## D13. Icône de notification, fenêtre et instance unique

**Décision.** L'icône et son menu natif utilisent `tray-icon` 0.26 et ses menus `muda`. Leurs
événements rejoignent le thread principal de GPUI par des canaux lus dans une tâche GPUI, comme
dans `crates/worktime-app/src/tray.rs`. La fenêtre des réglages est ouverte par `cx.open_window`
et détruite par `window.remove_window()` à sa fermeture, comme `crates/worktime-app/src/shell.rs`
et `crates/worktime-app/src/week_view.rs`. Ce dernier montre aussi une barre de titre dessinée
par l'application, avec `WindowControlArea::Close`, nécessaire à la barre « Commandes » des maquettes.

Le contrôle unique reste celui de PowerPlanSync, dans `src/windows/session.rs` : objet nommé
global et ouverture de la fenêtre dans la même session (FR-016). Bientôt garde une instance par
dossier de données et par session avec un canal nommé (`crates/worktime-windows/src/single_instance.rs`).
Ce modèle ne protège pas contre une seconde session et n'est donc pas repris. Le démarrage
avec Windows reste `src/windows/startup.rs` (T033). Bientôt utilise la même clé `Run`
(`crates/worktime-windows/src/startup.rs`), ce qui confirme le choix sans le changer.

Les notifications Windows du produit restent dans la fenêtre cachée de
`src/windows/notifications.rs`, qui reçoit déjà `WM_SETTINGCHANGE` et `WM_THEMECHANGED`.
Le sélecteur de fichier garde `src/windows/file_dialog.rs`. Bientôt utilise
`cx.prompt_for_paths` (`crates/bientot-app/src/file_dialogs.rs`, `origin/main`). T047 compare les
deux, car le filtre `.exe` de `prompt_for_paths` n'est pas confirmé.

**Raison.** GPUI n'a pas d'icône de notification ; Bientôt documente ce manque dans `tray.rs`.
Créer la fenêtre à la demande garde l'objectif SC-007 déjà prévu pour Slint.

**Alternatives.** Garder l'icône Slint : impossible sans boucle Slint. Écrire l'icône avec
`Shell_NotifyIcon` : possible, mais `tray-icon` est déjà éprouvé avec GPUI par Bientôt.

## D14. Thème, composants et écarts locaux

**Décision.** `src/interface/theme.rs` reprend les jetons du design system Bientôt GPUI : thème
WorkTime One clair et sombre, IBM Plex Sans et Mono. Le modèle est `crates/worktime-app/src/theme.rs`,
qui choisit la variante selon `window.appearance()`. Les écarts propres à PowerPlanSync, listés
dans [design/README.md](design/README.md) « Écarts avec le design system », vivent seulement dans
ce thème. Le design system lui-même n'est pas modifié depuis ce dépôt.

Le champ de texte suit `crates/worktime-app/src/text_input.rs`, que Bientôt maintient lui-même.
La palette de commandes suit `crates/bientot-app/src/palette.rs` et `palette_view.rs` (`origin/main`).

**Raison.** Les maquettes ont été validées sur ce design system. Les écarts sont déjà revus.

**Risque.** Bientôt ne traite pas les thèmes de contraste : aucune occurrence de
`SPI_GETHIGHCONTRAST` ni de couleur système dans ses crates. PowerPlanSync lit déjà le contraste
élevé dans `src/windows/preferences.rs`. Le remplacement des jetons par les couleurs système
(contrat, section Accessibilité) est donc à construire et à vérifier (T068, T070).

## D15. Mouvement et réglage « Effets d'animation »

**Décision.** `src/interface/motion.rs` reprend le module `theme::motion` de Bientôt,
`crates/bientot-app/src/theme/motion.rs` (`origin/main`) : durées PRESS 140 ms, TOOLTIP 150 ms,
MENU 200 ms et DIALOG 250 ms, courbes `ease_out` `(0.23, 1, 0.32, 1)` et `ease_in_out`
`(0.77, 0, 0.175, 1)`, poses `slide_in` et `slide_over`, `Entrance` et `OPAQUE_AT` = 0,1.
Le ralenti de débogage `BIENTOT_MOTION_SLOWDOWN` devient `POWERPLANSYNC_MOTION_SLOWDOWN`.

`App::reduce_motion` suit le réglage Windows « Effets d'animation ». Bientôt lit
`SPI_GETCLIENTAREAANIMATION`, appelle `cx.set_reduce_motion` au démarrage puis à chaque
`WM_SETTINGCHANGE` (`crates/bientot-windows/src/animation_effects.rs`, `animation_effects/native.rs`
et `crates/bientot-app/src/main.rs`, fonction `follow_animation_effects`). Une lecture échouée
garde le mouvement. Bientôt installe un hook de thread, car GPUI ne transmet pas `WM_SETTINGCHANGE`.
PowerPlanSync reçoit déjà ce message dans sa fenêtre cachée et n'a pas besoin de ce hook.

**Raison.** Le contrat d'interface et les maquettes citent ce module. Le commentaire de
`motion.rs` indique que `with_animation` et `with_spring` montrent leur état final quand
`reduce_motion` est actif.

**Risque.** Bientôt n'appelle `with_spring` nulle part ; seul ce commentaire le mentionne.
Les recettes « Monter ou Descendre » et « Étape validée » de design/README.md en dépendent.
T047 vérifie que `with_spring` existe dans `gpui-pre` 0.3.9 avec les paramètres voulus.
Sinon, ces deux transitions passent sur `ease_in_out` et `ease_out`, après accord du design.

## D16. Accessibilité, preuves et validation

**Décision.** Les éléments GPUI portent `.role(Role::…)`, `.aria_label`, `.aria_description`,
`.aria_toggled`, `.tab_index` et `.track_focus`, comme `crates/worktime-app/src/flyout.rs` et
`settings_view.rs`. `gpui-pre` 0.3.9 dépend d'`accesskit` 0.24.1, et `gpui-pre-windows` d'`accesskit_windows`
0.34.0 (`Cargo.lock` de Bientôt). L'arbre accessible passe donc par UI Automation. Un texte simple
n'est pas dans cet arbre : Bientôt passe par `readable` dans `flyout.rs`. PowerPlanSync suit la même règle.

Les tests de vues utilisent `gpui/test-support`, `HeadlessAppContext`, `set_reduce_motion` et
`advance_clock`, comme `crates/bientot-app/tests/common/mod.rs` (`origin/main`). La fonctionnalité `screenshots` rend les écrans hors
écran sous Linux avec `gpui_wgpu` et Mesa lavapipe, comme `crates/worktime-app/src/bin/screenshots.rs`
et l'étape « Capture GPUI screenshots » de `.github/workflows/ci.yml`.

Dans le conteneur Linux, les contrôles locaux restent `cargo fmt --check` et
`cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings`.
Le crate dépend de `windows` et ne compile pas pour Linux. Les tests ne s'exécutent que sous
Windows, en attendant la CI Windows prévue (issue #7). Narrateur, contraste, échelle et texte
agrandi exigent un vrai Windows. Les cycles réels utilisent l'exécutable témoin, jamais Battlefield.

**Risques et questions ouvertes.**

- Annonce sans déplacement du focus : Bientôt utilise `Role::Status` et `Role::Alert`, mais
  aucune preuve ne montre que le Narrateur les annonce comme une région active. T048 le vérifie.
- Texte agrandi de Windows : aucun traitement trouvé dans Bientôt. T048 et T070 le vérifient.
- Échelle par écran : Bientôt lit seulement `GetDpiForMonitor` pour placer son volet
  (`crates/bientot-app/src/tray.rs`, `origin/main`). La gestion du DPI par écran de
  `gpui-pre-windows` n'est pas confirmée. T047 et T048 la vérifient, avec un changement d'écran.
- Budgets SC-004 : Bientôt fixe une porte v1 de 60 Mo de mémoire, indicative sur un runner sans
  GPU, et un budget de 150 Mio de working set (`docs/ci-quality-gates.md` et
  `scripts/measure-performance.ps1`, `origin/main`). PowerPlanSync
  vise 40 Mio de mémoire privée fenêtre fermée. T048 mesure avant les fondations GPUI. Un échec
  exige un amendement explicite de SC-004, jamais une baisse silencieuse.
- Captures Linux : `screenshots` exige que les modules Windows soient compilés sous
  `cfg(windows)`. T057 isole ces modules. S'il coûte trop, il reste facultatif.
- Licence de `gpui-pre` et des polices IBM Plex : à vérifier en T044.

## D17. Résultats de GPUI sur Windows réel (T048, 2026-10-31)

**Constats.** Les mesures et essais sont dans
[prototype-results.md](evidence/prototype-results.md), section « Sonde GPUI sur Windows réel ».

- Fenêtre, icône, clavier, échelles 100 à 200 % en cours d'exécution, thèmes clair, sombre et
  contraste élevé : conformes. Le risque DPI de D16 est levé pour un écran ; le changement
  d'écran n'a pas été exécuté.
- Annonce sans déplacement du focus (risque de D16) : un `Role::Status` seul n'est pas annoncé.
  Deux contournements fonctionnent sans modifier GPUI. Le crochet `a11y_synthetic_children` pose
  `accesskit::Live` sur le nœud, et AccessKit lève `LiveRegionChanged`. Un fournisseur UIA minimal
  dont l'hôte est le HWND lève `UiaRaiseNotificationEvent`, prononcé par le Narrateur. Le premier
  suffit à l'application ; le second reste disponible pour une annonce sans élément visible.
- `aria_position_in_set` compte à partir de 0 et la taille de l'ensemble se pose sur le
  conteneur, pas sur l'élément (AccessKit 0.24, `accesskit_windows` 0.34).
- Texte agrandi (risque de D16) : ignoré par GPUI comme par `src/interface/`. L'interface Slint
  l'appliquait. À construire en T068, par exemple en dérivant la taille de texte des jetons de
  `text_scale_percent`, déjà lu par `src/windows/preferences.rs`.
- Budgets SC-004 (risque de D16) : 92,5 Mio fenêtre ouverte, sous les 100 Mio. **91,1 Mio
  fenêtre fermée, contre 40 Mio.** Sans aucune fenêtre, GPUI occupe déjà 59,7 Mio, et la première
  fenêtre de la sonde porte ce chiffre à 111 Mio après sa fermeture. L'interface Slint occupait 3,1 Mio
  fenêtre fermée. Le contrat d'interface promet que fermer la fenêtre « libère les ressources de
  la fenêtre » : ce n'est pas le cas dans un processus qui garde GPUI.
- Fermeture par `WM_CLOSE` (× dessiné, Alt+F4) : GPUI 0.3.9 garde le rendu de la fenêtre,
  environ 13 Mio par fermeture dans la sonde et 25 Mio dans l'application, qui atteint 219 Mio
  fenêtre fermée après six ouvertures. Refuser la fermeture native par `on_window_should_close`
  et appeler `remove_window()` supprime la fuite : l'application se stabilise vers 97 Mio. Ce
  défaut est à signaler au projet GPUI ; la PR #30 ajoute le contournement à
  `src/interface/window.rs`.
- Cent cycles fermés par `remove_window()` : +23,2 Mio et +88 handles entre les marqueurs, en un
  palier au premier cycle, au-dessus du seuil de 5 Mio et 10 handles calibré sur Slint.

**Proposition d'amendement.** Comme le prévoit le plan, un dépassement de SC-004 arrête ce point
et revient au responsable produit avec les mesures.

1. **Recommandée : fenêtre dans un processus séparé.** Le processus de suivi garde le moteur, le
   coordinateur, le contrôle unique et l'icône `tray-icon`, sans initialiser GPUI. Ouvrir la
   fenêtre lance `power-plan-sync.exe --window`, qui reçoit l'état et renvoie ses commandes par un
   canal nommé local réservé à la session ; fermer la fenêtre arrête ce processus. SC-004 et
   SC-007 restent inchangés, et la fermeture libère réellement GPUI. Le découpage existant s'y
   prête : l'interface reçoit déjà des `UiCommand` par canal et renvoie des `UiAction` typées
   (T056). Coûts : sérialisation de ces messages, ouverture plus lente d'une fraction de seconde,
   un second processus à vérifier pour l'instance unique. À prouver d'abord par une mesure du
   processus de suivi sans GPUI.
2. **Amender SC-004** à 100 Mio fenêtre fermée. Aucun changement d'architecture, mais un outil
   d'arrière-plan trente fois plus lourd que la version Slint, et une promesse du contrat
   d'interface à retirer. Avec le contournement, l'application se stabilise à 97 à 98 Mio
   fenêtre fermée sur cette machine : la marge est de 2 Mio, et une autre carte graphique peut
   la dépasser. SC-007 demanderait aussi un seuil propre à GPUI.
3. **Revenir à Slint** : rejeté sauf avis contraire, la décision D12 reposant sur le design system.

**Décision du 2026-10-31 : option 1.** Le responsable produit retient la fenêtre dans un
processus séparé. SC-004 et SC-007 ne sont pas amendés. Le prototype du processus de suivi sans
GPUI occupe 3,6 Mio de mémoire privée avec 20 règles (T073, section « Processus de suivi sans
GPUI » de [prototype-results.md](evidence/prototype-results.md)). La mise en œuvre suit D18.
T065 à T070 continuent dans `src/interface/`. T071 attend T081.

## D18. Fenêtre dans un processus séparé

**Décision.** Deux rôles d'un même exécutable.

- **Le processus de suivi** est `power-plan-sync.exe`. Il garde le coordinateur, le contrôle
  unique, la fenêtre cachée des notifications, le sélecteur de fichier et l'icône `tray-icon`. Son
  thread principal pompe les messages Windows pour l'icône. Il n'initialise jamais GPUI.
- **Le processus de fenêtre** est `power-plan-sync.exe --window`. Il est lancé par le processus de
  suivi, et par lui seul. Il exécute GPUI et `src/interface/` sans icône, sans verrou de contrôle,
  sans dossier de données ni accès aux plans Windows. Fermer sa fenêtre l'arrête.

Le nom `--window` est confirmé. L'option n'apparaît pas dans `--help`. Lancée sans le canal du
processus de suivi, elle s'arrête aussitôt sans rien afficher.

**Canal.** Le processus de suivi crée deux canaux nommés à sens unique pour chaque processus de
fenêtre : commandes vers la fenêtre, actions vers le suivi. Chaque canal a :

- un nom `\\.\pipe\PowerPlanSync.Window.<session>.<pid>.<compteur>.<sens>` ;
- une seule instance, créée avec `FILE_FLAG_FIRST_PIPE_INSTANCE`, et `PIPE_REJECT_REMOTE_CLIENTS` ;
- une DACL protégée qui n'accorde l'accès qu'au SID du compte courant, comme le signal de
  réouverture de `src/windows/session.rs`.

Le processus de suivi ouvre lui-même l'extrémité cliente avant le lancement, puis la transmet au
seul processus de fenêtre comme entrée et sortie standard. Le processus de fenêtre n'ouvre aucun
canal par son nom. Aucun autre processus ne peut donc s'y connecter par le nom : le nom est déjà
pris, sa seule instance est connectée, et la DACL écarte les autres comptes. Un intrus connecté
avant le processus de suivi ferait échouer son ouverture : ce lancement est alors abandonné, sans
attente ni nouvel essai. Comme pour le signal de réouverture, un processus du même compte reste de
confiance : il pourrait de toute façon dupliquer un handle ou agir à la place de l'utilisateur.
L'enfant est créé dans la session du processus de suivi.

Toutes les extrémités sont créées non héritables. La bibliothèque standard ne rend héritables que
les handles standard du lancement, et le processus de suivi ferme ses copies des extrémités
clientes dès le lancement fait : sans cela, l'arrêt de l'enfant ne terminerait jamais le canal
des actions. Au démarrage, le processus de fenêtre reprend ses deux extrémités en propriété privée
et non héritable, puis remplace ses handles standard par `NUL`. Un `println!` d'une dépendance ne
peut pas corrompre une trame, et aucun processus qu'il lancerait n'hériterait du canal. Le
processus de fenêtre ne lance d'ailleurs rien : **Ouvrir le guide**, **Voir les licences** et
**Ouvrir le dossier des données** restent des `UiAction` exécutées par le processus de suivi.

**Messages.** Chaque message est une trame : longueur sur 4 octets en petit-boutiste, puis un objet
JSON `{"version": …, "message": …}`, où `version` vaut `window_protocol::VERSION`. Le processus de
fenêtre reçoit les `UiCommand` existantes et renvoie les `UiAction` existantes, rendues
sérialisables par `serde` dans `src/presentation.rs`. Une trame est refusée si sa version diffère,
si son message est inconnu, si un champ manque ou s'ajoute, ou si elle dépasse 1 Mio. Un champ
manque ou s'ajoute quand le message lu, sérialisé à nouveau, diffère du JSON reçu : la règle vaut
aussi pour un champ `Option`, que `serde` remplirait sinon par `None`. Un refus ferme le canal : le
processus de fenêtre s'arrête, et le processus de suivi traite la fenêtre comme perdue. La version
change à chaque modification de ces types. Les deux rôles viennent du même exécutable : un écart de
version ne survient qu'après un remplacement du fichier pendant l'exécution.

**Cycle de vie.** Au plus un processus de fenêtre existe.

- Ouvrir la fenêtre (clic sur l'icône, **Ouvrir PowerPlanSync**, second lancement, nouvelle
  erreur) lance le processus de fenêtre, ou lui envoie `UiCommand::Open` s'il existe. Le processus
  de suivi lui transmet d'abord le dernier modèle, puis chaque nouveau modèle. Une instance sans
  contrôle, parce qu'une autre session le détient, lance de même le processus de fenêtre avec le
  modèle « Contrôle indisponible » (FR-016).
- Une demande d'ouverture reçue pendant qu'un processus de fenêtre se ferme attend la fin de son
  canal, puis lance un nouveau processus.
- Le processus de fenêtre répond `UiAction::WindowOpened` après chaque affichage. Sans cette
  réponse dans le délai, le processus de suivi l'arrête par `TerminateProcess`. Après `Open`
  (5 s), il lance un nouveau processus pour la même demande, une seule fois. Au lancement (15 s),
  il ne relance pas. Il ne relance jamais de lui-même : une fenêtre perdue reste fermée jusqu'à
  la demande suivante. Aucune boucle de tentatives n'est possible.
- La file vers la fenêtre ne garde que le dernier `UiCommand::Update` en attente : un nouveau
  modèle remplace l'ancien. Au-delà de 64 autres commandes en attente, la fenêtre ne lit plus et
  son processus est arrêté. Le coordinateur n'attend jamais l'écriture.
- La fin du canal des actions signale l'arrêt du processus de fenêtre. Un code de sortie nul est
  une fermeture : au premier masquage, l'icône affiche « PowerPlanSync reste ouvert ». Un autre
  code, une trame refusée ou un arrêt forcé est une perte, notée dans le journal de diagnostic.
  L'icône l'explique par une notification : « La fenêtre de PowerPlanSync s'est fermée. Le suivi
  continue ; cliquez sur l'icône pour la rouvrir. » Le suivi continue dans les deux cas. Quand le
  processus de suivi remplace lui-même la fenêtre après un `Open` sans réponse, la perte n'est
  notée qu'au journal. La fin de canal d'une instance déjà arrêtée ou remplacée est ignorée. La
  table d'états complète est dans `contracts/runtime.md`.
- À l'arrêt du processus de suivi, `UiCommand::Quit` ferme la fenêtre ; après 2 s, le processus de
  fenêtre est arrêté. S'il perd son canal de commandes, le processus de fenêtre s'arrête de
  lui-même : le processus de suivi peut disparaître sans laisser de fenêtre orpheline.
- Windows n'accorde le premier plan qu'au processus qui a reçu la dernière saisie. Le processus de
  suivi appelle `AllowSetForegroundWindow` avec le PID du processus de fenêtre, connu après
  `CreateProcess`, avant d'envoyer le modèle ou `Open`. Un second lancement ne connaît pas le PID
  du processus de suivi : il appelle `AllowSetForegroundWindow(ASFW_ANY)` avant de signaler
  l'instance existante. Avant de demander le sélecteur de fichier, le processus de fenêtre
  autorise le processus de suivi, dont le PID est celui du serveur de son canal
  (`GetNamedPipeServerProcessId`).

**Budgets.** Fenêtre fermée, SC-004 porte sur le processus de suivi seul : le processus de fenêtre
n'existe plus. Fenêtre ouverte, il porte sur la somme des deux processus. SC-007 vérifie après
cent cycles qu'aucun processus de fenêtre ne reste, et mesure la dérive du processus de suivi.
Le script de référence de SC-007 est `scripts/measure-window-reopen.ps1`, adapté aux deux
processus en T080. Comme pour T073, un dépassement en T077 ou T081 arrête la phase et revient
au responsable produit avec les mesures. Le temps d'ouverture est relevé à titre informatif : la
spécification ne lui fixe aucun seuil.

**Raison.** La mesure T073 place le prototype du processus de suivi à 3,6 Mio en vrai mode avec
20 règles. GPUI dans le même processus occupait 91,1 Mio fenêtre fermée, mesuré en `--demo`. Le
processus de fenêtre s'arrêtant, sa mémoire et la fuite de GPUI à chaque `WM_CLOSE` devraient
disparaître avec lui : c'est attendu, et T081 le vérifie. Le contournement de `MainWindow`
(`on_window_should_close` refuse la fermeture native, puis `remove_window()`, PR #30) reste en
place. Il ne coûte rien et garde `src/interface/` correct hors du processus de fenêtre. Le
découpage existant s'y prête : `UiCommand` et `UiAction` passaient déjà par un canal entre le
coordinateur et l'interface (T056).

**Alternatives.**

- Canaux anonymes de la bibliothèque standard (`Stdio::piped`) : sûrs, car connectés avant le
  lancement, mais avec la DACL par défaut. Rejetés au profit d'une DACL explicite du compte.
- Un canal nommé que le processus de fenêtre ouvre par son nom : rejeté. Un autre processus du
  même compte pourrait l'ouvrir le premier, et il faudrait vérifier son identité après coup.
- Un objet tâche (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) pour arrêter la fenêtre avec le suivi :
  inutile, la fin du canal de commandes suffit.
- Un battement de cœur périodique : rejeté. Il réveillerait les deux processus sans demande. La
  réponse à `Open` suffit, puisque la vivacité ne compte qu'au moment où l'utilisateur demande la
  fenêtre.
- Le sélecteur de fichier dans le processus de fenêtre, avec la fenêtre pour propriétaire : à
  réévaluer en T080 si l'autorisation de premier plan ne suffit pas.

**Risques.** L'ouverture coûte un démarrage de GPUI, à mesurer en T081. Fenêtre ouverte, la
valeur cible de 100 Mio s'applique désormais à deux processus. La seule mesure disponible, GPUI et
icône dans un processus en `--demo`, donne 92,5 Mio : la marge attendue est de quelques Mio, sans
mesure de la somme avant T077.
