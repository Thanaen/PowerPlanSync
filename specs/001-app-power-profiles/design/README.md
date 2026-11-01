# Maquettes de l'interface GPUI

Ces maquettes préparent la migration de l'interface vers GPUI. Elles reprennent le design system
**Bientôt GPUI** (anciennement WorkTime GPUI) : thème WorkTime One clair et sombre, IBM Plex Sans et Mono, composants GPUI Kit.
Les originaux se trouvent sur la toile Claude Design « PowerPlanSync GPUI · maquettes »
(https://claude.ai/artifact/Lzw5kCJu484qb26Tk6V9ni, privée). Les captures ci-dessous en
sont des copies figées. Les libellés et les comportements qui font foi sont dans
[contracts/user-interface.md](../contracts/user-interface.md). Le brouillon du guide utilisateur pour cette interface est [guide-gpui.md](guide-gpui.md).

Deux boucles de revue indépendantes ont validé ces maquettes. La première en quatre tours, avec
trois relecteurs : design, textes et captures. La seconde en trois tours, avec un quatrième
relecteur consacré au mouvement.

## Écrans

| Écran | Capture |
| --- | --- |
| Cycle actif | [Main](ecrans/Main.png), [thème clair](ecrans/MainLight.png), [fenêtre étroite](ecrans/Narrow.png) |
| Premier lancement | [Pour commencer](ecrans/FirstLaunch.png), [prêt](ecrans/FirstLaunchReady.png), [plan normal absent](ecrans/FirstLaunchMissing.png) |
| Ajouter ou modifier | [Ajouter](ecrans/AddRule.png), [plan invalide](ecrans/AddRuleInvalid.png), [fichier introuvable](ecrans/EditRule.png) |
| États | [en attente, retrait annulable](ecrans/Idle.png), [en pause](ecrans/Paused.png), [plan changé dans Windows](ecrans/External.png), [changement refusé](ecrans/Error.png), [sur batterie](ecrans/Battery.png), [retour échoué en quittant](ecrans/QuitFailed.png), [réglages illisibles](ecrans/ConfigError.png), [démonstration](ecrans/Demo.png), [contrôle indisponible](ecrans/Unavailable.png) |
| Réordonner | [Blender passe en tête](ecrans/Reorder.png) |
| Autres pages | [Réglages](ecrans/Settings.png), [Aide et limites](ecrans/Help.png), [palette de commandes](ecrans/Palette.png) |
| Clavier | [focus dans la liste](ecrans/Focus.png), [focus dans le dialogue](ecrans/FocusDialog.png) |
| Zone de notification | [icônes, menu, notification, infobulle](ecrans/Tray.png) |

## Mouvement

Le mouvement suit les skills d'Emil Kowalski (`animate`, `find-animation-opportunities`,
`review-animations`) et la section Mouvement du design system, qui décrit le module
`theme::motion` de Bientôt (dépôt WorkTime) : poses `slide_in` et `slide_over`, fermeture
instantanée, entrée seulement si l'événement arrive fenêtre ouverte. Les durées sont : PRESS 140 ms, TOOLTIP 150 ms,
MENU 200 ms et DIALOG 250 ms. Les courbes sont : `ease_out` = `cubic-bezier(0.23, 1, 0.32, 1)`
pour une entrée, et `ease_in_out` = `cubic-bezier(0.77, 0, 0.175, 1)` pour un déplacement.

| Transition | Recette | Étapes |
| --- | --- | --- |
| Retrait annulable | `slide_in(MENU, 4 px)` sous la liste, uniquement si le retrait vient d'être fait fenêtre ouverte. Un second retrait garde le message et change son texte. Annuler, Ctrl+Z et × sont instantanés. | [0 %](mouvement/Idle-000.png), [50 %](mouvement/Idle-050.png), [100 %](mouvement/Idle-100.png) |
| Monter ou Descendre à la souris | `with_spring` sans rebond, 0,2 s, sur la hauteur mesurée de l'autre ligne. La ligne qui descend part 40 ms plus tard. La ligne qui monte passe au-dessus avec une bordure `border` de 1 px, sans ombre, le temps du déplacement. Le numéro, le badge et le plan actif changent immédiatement. | [0 %](mouvement/Reorder-000.png), [50 %](mouvement/Reorder-050.png), [100 %](mouvement/Reorder-100.png) |
| Ouverture au clic du dialogue Ajouter un logiciel | Voile en fondu sur DIALOG, carte en `slide_over(DIALOG, 6 px)`, opaque dès 10 % de l'animation, comme le composant Dialog du design system. Ctrl+N ouvre sans animation. La fermeture est instantanée. | [0 %](mouvement/AddRule-000.png), [50 %](mouvement/AddRule-050.png), [100 %](mouvement/AddRule-100.png) |
| Étape validée dans « Pour commencer » | La pastille entre avec `with_spring` (bounce 0.2) en 250 ms. C'est un moment unique : ce rebond n'est pas un jeton. | [0 %](mouvement/FirstLaunchReady-000.png), [50 %](mouvement/FirstLaunchReady-050.png), [100 %](mouvement/FirstLaunchReady-100.png) |

Les messages Plan modifié dans Windows, Changement refusé et Retour échoué apparaissent sans
animation. Ces événements surviennent surtout fenêtre fermée, et animer leur hauteur forcerait
une mise en page à chaque image. Rien n'est animé au clavier. La palette, le menu de l'icône, les
valeurs lues et la navigation ne le sont pas non plus. `App::reduce_motion` suit le réglage Windows
« Effets d'animation » et affiche directement l'état final. Il faut vérifier chaque animation au
ressenti dans l'application réelle, avec les durées ralenties quatre fois.

Les ressorts gardent leurs paramètres (`SpringConfig` de GPUI), mais `motion::Entrance` les calcule
sur l'horloge de l'exécuteur : `with_spring` et `with_animation` lisent l'horloge murale, que les tests
de vue ne peuvent pas avancer. Pendant un échange, les deux lignes gardent leur ancien ordre jusqu'au
repos : la ligne qui monte est ainsi dessinée par-dessus l'autre.

## Écarts avec le design system

Le design system s'appelle désormais **Bientôt GPUI** et reflète le code de Bientôt. Il n'est donc
pas modifié depuis PowerPlanSync : les écarts ci-dessous restent propres à PowerPlanSync, dans son
thème, jusqu'à ce que Bientôt les adopte.

- anneau de focus de 2 px tracé à 1 px du contrôle, et `border-focused` à `#4560c4` dans le thème clair ;
- texte du bouton Tinted en `accent-on-active` dans le thème clair ;
- bouton Tinted désactivé : fond transparent, bordure `border`, texte `text-placeholder` ;
- `g-badge--error` (texte `#e8999c` dans le thème sombre) et badge accent en `accent-on-active` ;
- Switch éteint : pastille en `text-muted`, piste en `element-active` et bordure en `text-placeholder` ;
- pastille du Switch sans glissement et liste déroulante ouverte sans `slide_over` : le contrat n'anime que quatre transitions, et la liste s'ouvre aussi au clavier ;
- boutons de fenêtre comme ceux de Windows : glyphes de 10 px, glyphe blanc sur le rouge de **Fermer** survolé (`#c42b1c` dans le thème sombre, HighlightText sur Highlight en contraste élevé) et glyphe de restauration quand la fenêtre est agrandie ;
- infobulles de la barre latérale étroite affichées sans le fondu `tooltip` de 150 ms : GPUI les montre au survol sans animation d'entrée.
- liste **Application ouverte** sans barre de défilement visible, que GPUI ne dessine pas : la troisième ligne coupée signale le défilement ;
- voile du dialogue posé sous la barre de titre, qui garde ses boutons de fenêtre utilisables.
- texte agrandi de Windows, que Bientôt ne traite pas : les styles de texte, les hauteurs des contrôles qui portent du texte (`button-*`, barre de titre, barre d'état, badge, capuchon), la barre latérale, les listes et colonnes de largeur fixe et les seuils de mise en page sont convertis en rems (16 px à 100 %), et la fenêtre pose le rem à 16 px × la taille du texte. Les icônes, les espacements et les interrupteurs gardent leurs pixels, comme dans les applications Windows. À 100 %, le rendu est identique au pixel près ;
- lignes de **Logiciels** sur plusieurs niveaux sous 840 px × la taille du texte, même barre latérale entière, et capuchons des raccourcis masqués sous 400 px × la taille du texte ;
- ligne de **Logiciels associés** focalisée au clavier : l'anneau de 2 px est tracé à l'intérieur de la ligne, dont le bord est celui de la liste, et suit ses coins arrondis ;

Les couleurs `tray-*` ont un sens propre à PowerPlanSync : bleu pour un plan de logiciel appliqué,
gris quand aucun ne l'est, orange pour une automatisation suspendue par un choix fait dans Windows, rouge quand une action est
nécessaire.

L'implémentation de l'icône (`src/interface/tray.rs`) s'écarte de la maquette `Tray.png` sur deux
points : les pictogrammes sont pleins, plus nets à 16 px que les traits de la maquette, et la seconde
ligne de l'infobulle reprend la ligne d'état entière (« Automatisation active · Battlefield 6 est
ouvert »), comme la première ligne du menu, pour nommer l'état de l'automatisation en pause ou
suspendue.

La page **Réglages** (`src/interface/settings_page.rs`) s'écarte de la maquette `Settings.png` sur
un point : la description de **Dossier des données** ne cite pas `%LOCALAPPDATA%\PowerPlanSync`,
faux quand `--data-dir` choisit un autre dossier. **Ouvrir le dossier** mène toujours au dossier
réellement utilisé.

La page **Aide et limites** (`src/interface/help_page.rs`) s'écarte de la maquette `Help.png` sur
trois points. La section **Limites** précise que les plans s'affichent sous leur nom dans Windows,
comme le demande le contrat. La section sur le choix fait dans Windows dit « conserve ce choix »
au lieu de « le garde », dont le pronom était ambigu. Le texte ne dépasse pas 680 px de large :
fenêtre agrandie, les lignes restent lisibles. Le numéro de version est celui de l'exécutable.

La palette de commandes (`src/interface/palette.rs`) suit `Palette.png`, sans le voile de la palette
de Bientôt : la maquette laisse la page lisible derrière elle. Elle ajoute deux entrées absentes de
la maquette, listées seulement quand elles servent : **Logiciels : annuler le retrait** après un
retrait et **Logiciels : ouvrir** hors de la page **Logiciels**.
