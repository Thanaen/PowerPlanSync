# Recherche et décisions de conception

Recherche effectuée le 2026-09-20. Les constats Windows sont locaux et en lecture seule.
Les choix ci-dessous définissent le point de départ des prototypes, pas leurs résultats.

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
| Interface Slint | Parcours clavier et Narrateur complet, contraste et DPI conformes | Partiel, Narrateur et contrastes ouverts |
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
