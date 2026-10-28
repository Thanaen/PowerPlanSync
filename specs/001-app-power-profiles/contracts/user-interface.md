# Contrat de l'interface

Les maquettes de référence sont décrites dans [design/README.md](../design/README.md). Elles
utilisent le design system Bientôt GPUI (anciennement WorkTime GPUI). Ce contrat fixe les
libellés, les comportements et l'accessibilité. En cas d'écart, ce contrat et la spécification
font foi.

## Vocabulaire

- **Automatisation** désigne l'état global : active, en attente, en pause, suspendue,
  désactivée ou arrêtée.
- **Suivre** s'applique à un logiciel. Un logiciel suivi a son interrupteur activé. La liste
  s'intitule **Logiciels associés**. Le mot « règle » reste interne et n'apparaît pas à l'écran.
- Quand un nom de plan est le sujet d'une phrase, on écrit « le plan Haute performance ». Les noms
  de plan cités sont insécables. Les textes affichés à l'écran suivent la typographie française
  (espace insécable avant « : ; ? », espace fine dans les guillemets, apostrophe ’).

## Fenêtre principale

La fenêtre a une barre de titre, une barre latérale (**Logiciels**, **Réglages**, **Aide et
limites**, puis **Quitter** en bas) et une barre d'état. La page **Logiciels** commence par
**Plan actif dans Windows** et le nom observé. Sous ce nom, un statut associe une icône, un
mot-clé coloré et une phrase, par exemple « Automatisation active · Battlefield 6 est ouvert ».
L'état ne repose jamais sur la couleur seule. La ligne suivante indique le plan de retour.

Les actions sont **Ajouter un logiciel** (Ctrl+N) et **Mettre en pause** ou **Reprendre**
(Ctrl+Alt+P). Chaque surface n'a qu'une action principale. En pause, c'est **Reprendre**, placé
en premier. Il n'y a pas d'interrupteur global permanent.

Chaque ligne de **Logiciels associés** présente son rang, le nom du logiciel, son chemin abrégé
au milieu (par exemple `C:\…\Blender 4.5\blender.exe`), le plan associé, l'interrupteur
**Suivre** et les actions **Monter**, **Descendre**, **Modifier** et **Supprimer**. Le chemin
complet est donné en infobulle et dans la description accessible. Un badge indique l'état du
logiciel : **Appliqué**, **Ouvert**, **Suspendu** ou **Refusé**. La phrase « Si plusieurs logiciels
suivis sont ouverts, le premier de la liste décide du plan. » explique les conflits. Une liste
longue défile sans réduire les contrôles ni tronquer leurs noms accessibles.

Supprimer un logiciel ne demande pas de confirmation. Un message sous la liste annonce
« Hollow Bloom a été retiré de la liste. » avec **Annuler** (Ctrl+Z). Un second retrait garde le
même message et change son texte.

La barre d'état reprend l'état de l'automatisation, le nombre de logiciels suivis (« 2 logiciels
suivis sur 3 ») et la source d'alimentation. La palette de commandes (Ctrl+Maj+P, bouton
**Commandes**) propose « Domaine : verbe », par exemple « Logiciels : ajouter ».

## Premier lancement

La page **Logiciels** affiche « Pour commencer », avec trois étapes : **Ajouter un logiciel**,
**Confirmer le plan de retour** (le plan normal installé est proposé et doit être confirmé ; s'il
manque, rien n'est présélectionné) et **Activer l'automatisation**. L'étape 2 associe la liste
des plans à un bouton **Confirmer**, qui valide le plan choisi. Le bouton de l'étape 3 reste
indisponible tant que les deux premières ne sont pas faites. Rien ne change dans Windows avant
l'étape 3. Une fois l'automatisation activée, la liste « Logiciels associés » remplace ces étapes.

## Ajouter ou modifier un logiciel

Le dialogue **Ajouter un logiciel** propose deux sources : **Parcourir…** ou **Application
ouverte**, avec un filtre. Il affiche ensuite le fichier complet, **Nom affiché** et **Plan pendant
l'exécution**. Le bouton principal est **Ajouter** (Entrée) ; **Annuler** (Échap) ferme sans
rien changer. Dans **Modifier**, le bouton principal est **Enregistrer**.

Le sélecteur ne propose que des chemins vérifiables. Un processus inaccessible apparaît
désactivé, avec « Fichier inaccessible · impossible à suivre », et n'est jamais enregistré par
approximation. Le message « Le même nom dans un autre dossier compte comme un autre logiciel. »
accompagne le chemin. Battlefield 6 n'est pas une règle intégrée qui prétend connaître son
exécutable. Un plan disparu de Windows est signalé sous son champ (« Ce plan n'existe plus dans
Windows. Choisissez-en un autre. ») et **Ajouter** reste indisponible. Un fichier déplacé
s'annonce par « Fichier introuvable » ; l'utilisateur choisit le nouveau fichier et garde le plan.

## États signalés dans la fenêtre

Les situations à signaler s'affichent dans un message intégré à la page, jamais
dans une boîte de dialogue bloquante :

- **Plan modifié dans Windows** : statut « Automatisation suspendue · plan choisi dans Windows »,
  badge **Suspendu**, action **Reprendre l'automatisation**.
- **Windows a refusé le changement de plan** : badge **Refusé**, actions **Réessayer** et
  **Ouvrir les options d'alimentation**. PowerPlanSync ne réessaie pas seul.
- **Le retour au plan Utilisation normale a échoué** (en quittant) : actions **Réessayer**,
  **Ouvrir les options d'alimentation** et **Quitter quand même**.
- **Sur batterie** : « Automatisation en attente · sur batterie, aucun plan ne sera changé ».
- **Contrôle indisponible** (une autre session Windows gère déjà les plans) : message titré
  « Contrôle indisponible », texte « PowerPlanSync gère déjà les plans depuis une autre session
  Windows. Aucun plan n’a été modifié. Quittez-le dans cette autre session, puis réessayez. »,
  action **Réessayer**. La page garde le plan actif et indique « Automatisation en attente · une
  autre session gère les plans », sans actions ni liste. La barre d'état n'affiche pas le compte
  des logiciels, que cette instance ne lit pas. Si le verrou ne peut pas être lu, le message, sans titre, dit « PowerPlanSync
  ne peut pas vérifier si une autre session gère déjà les plans. Aucun plan n’a été modifié.
  Réessayez dans un instant. ». Une seconde instance de la même session ne montre rien : elle
  ramène la fenêtre de la première et s'arrête.
- **Les réglages sont illisibles** : l'automatisation reste arrêtée. Actions **Ouvrir le dossier
  des données**, **Relire les réglages** et **Ouvrir le guide**, qui renvoie à la section
  « Réglages illisibles » du guide. Le texte prévu pour ce guide est dans
  [design/guide-gpui.md](../design/guide-gpui.md).

## Réglages et notification

**Réglages** regroupe **Après fermeture** (**Plan de retour**), **Conditions** (**Changer de plan
sur batterie**, **Démarrer avec Windows**) et **Données** (**Ouvrir le dossier**). Les deux options
sont désactivées par défaut. Si un cycle est en cours quand le plan de retour change, un message
indique : « À la fin du cycle en cours, PowerPlanSync reviendra au plan Utilisation normale,
choisi à son début. Le plan Économies d'énergie s'appliquera à partir du prochain cycle. »

Le menu natif de l'icône contient l'état (« Automatisation active · Battlefield 6 est ouvert »),
**Ouvrir PowerPlanSync**, **Mettre en pause** ou **Reprendre**, puis **Quitter PowerPlanSync**. Un
clic ouvre la fenêtre. L'infobulle donne le plan actif et sa raison. La couleur de l'icône suit
l'état : bleu pour un plan de logiciel appliqué, gris quand aucun plan de logiciel n'est appliqué,
orange pour une automatisation suspendue par un choix fait dans Windows, rouge quand une action est nécessaire. Fermer la fenêtre
conserve l'icône et libère les ressources de la fenêtre. Au premier masquage, une notification
l'explique une seule fois : « PowerPlanSync reste ouvert ».

Les bascules normales ne génèrent pas de notifications à répétition. Les erreurs persistantes
et les cas demandant une action sont visibles dans la fenêtre et l'état de l'icône.

## Accessibilité

L'ordre de tabulation suit la lecture et le focus est un anneau visible de 2 px. Les listes,
menus et dialogues portent noms, rôles et états. Une ligne s'annonce par exemple « 1. Battlefield 6,
plan Haute performance, appliqué », avec le chemin complet en description. Le Narrateur annonce
une modification de statut sans déplacer le focus. Après **Monter** ou **Descendre**, il annonce
« Blender déplacé en position 1 sur 3. Plan actif : Performances optimales. » (la seconde phrase
seulement si le plan actif change). Si le bouton utilisé devient indisponible, le focus passe au
bouton opposé de la même ligne. Fermer un dialogue rend le focus au bouton qui l'a ouvert. Échap
annule un dialogue et Entrée active son action principale. Un dialogue invalide explique le champ
à corriger et conserve sa saisie.

Les thèmes suivent Windows, y compris les thèmes de contraste : les jetons sont alors remplacés par
les couleurs système et l'état reste porté par l'icône et le mot. Les parcours passent à 100 %,
150 % et 200 % de mise à l'échelle et avec texte agrandi. Une fenêtre étroite réduit la barre
latérale à ses icônes et défile plutôt que de serrer les contrôles.

## Mouvement

Le mouvement reste rare et court. Il suit la section Mouvement du design system Bientôt GPUI (MENU 200 ms,
DIALOG 250 ms, poses `slide_in` et `slide_over`). Quatre transitions seulement sont animées :

1. le message de retrait annulable, qui entre en glissant de 4 px, seulement si le retrait vient
   d'être fait fenêtre ouverte ;
2. l'échange de deux lignes après **Monter** ou **Descendre** à la souris ;
3. l'ouverture au clic du dialogue **Ajouter un logiciel** ;
4. la pastille (coche) d'une étape validée dans « Pour commencer ».

Une action au clavier n'est jamais animée. La palette, le menu de l'icône, les valeurs lues
(plan, statut, compteur) et la navigation ne le sont pas non plus. Les fermetures sont
instantanées. Quand le réglage Windows « Effets d'animation » est désactivé, l'interface affiche
directement l'état final.

## Limites à expliquer dans l'aide

La page **Aide et limites** explique comment le plan est choisi, que le choix fait dans Windows
reste prioritaire, ce qui se passe après un arrêt forcé et les limites du produit. Le produit
choisit des plans d'alimentation Windows existants. Le mode « Recommandé » des Paramètres peut
être un réglage distinct. Les libellés affichés viennent de la machine. Un arrêt forcé peut laisser
le dernier plan actif ; au lancement suivant, PowerPlanSync vérifie le plan actif et conserve un
plan choisi entre-temps. Aucun gain d'images par seconde, de température ou de consommation n'est
annoncé. La page se termine par **Voir les licences** et **Ouvrir le guide**.
