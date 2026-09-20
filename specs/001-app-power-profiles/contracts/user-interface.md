# Contrat de l'interface

## Fenêtre principale

La fenêtre montre d'abord **Plan actif** avec le nom observé dans Windows.
Une phrase en explique la raison, par exemple « Haute performance pendant Battlefield 6 ».
Le statut dispose d'un texte, pas seulement d'une couleur. Les actions globales sont
**Ajouter un logiciel**, **Pause** ou **Reprendre**, et **Réglages**.

Chaque ligne présente nom du logiciel, plan associé, activation et état.
Les actions **Modifier**, **Supprimer**, **Monter** et **Descendre** sont accessibles au clavier.
La phrase « Le premier logiciel actif de la liste a priorité » explique les conflits.
Une liste longue défile sans réduire les contrôles ni tronquer leurs noms accessibles.

## Ajouter un logiciel

1. Choisir **Parcourir** ou **Choisir une application ouverte**.
2. Sélectionner le logiciel, puis un plan installé.
3. Confirmer avec **Enregistrer**.

Le sélecteur d'applications ne propose que des chemins vérifiables. Un processus inaccessible
peut être signalé mais n'est pas enregistré comme une correspondance approximative.
Le chemin complet est disponible dans les détails et distinguera deux logiciels homonymes.
Le nom Battlefield 6 n'est pas une règle intégrée qui prétend connaître son fichier exécutable.

Au premier usage, choisir aussi **Après fermeture, revenir à**. Le plan normal installé est
proposé. Si celui-ci manque, aucun remplacement n'est sélectionné silencieusement.
**Activer l'automatisation** reste indisponible avant une règle et un retour valides.
Enregistrer une première règle ne change pas encore le plan sans cette activation.

## Réglages et notification

Les réglages contiennent le plan de retour, **Autoriser sur batterie** et **Démarrer avec Windows**.
Les deux options sont désactivées par défaut. Si un cycle fonctionne, afficher son retour réservé
et expliquer qu'un nouveau retour prendra effet au prochain cycle.

Le menu de l'icône contient **Ouvrir PowerPlanSync**, l'état courant, **Pause** ou **Reprendre**,
et **Quitter**. Un clic ouvre les réglages. La fermeture de la fenêtre conserve l'icône et libère
les ressources de cette fenêtre. Au premier masquage, expliquer ce comportement une seule fois.

Les bascules normales ne génèrent pas de notifications à répétition. Les erreurs persistantes
et les cas demandant une action sont visibles dans la fenêtre et l'état de l'icône.

## Accessibilité

L'ordre de tabulation suit la lecture. Les listes, menus et dialogues portent noms, rôles et états.
Le Narrateur annonce une modification de statut sans déplacer le focus ni interrompre constamment
la lecture. Échap annule un dialogue et Entrée active son action principale lorsque c'est approprié.
Un dialogue rendu invalide explique le champ à corriger et conserve sa saisie.

Les thèmes suivent Windows, y compris le contraste élevé. Le focus reste visible sur chaque contrôle.
Les parcours passent à 100 %, 150 % et 200 % de mise à l'échelle et avec texte Windows agrandi.
La fenêtre accepte une petite hauteur avec défilement plutôt que des contrôles serrés.
Le prototype teste ces comportements avec les vrais outils d'accessibilité Windows.

## Limites à expliquer dans l'aide

Le produit choisit des plans d'alimentation Windows existants. Le mode « Recommandé » des Paramètres
peut être un réglage distinct. Les libellés affichés viennent de la machine.
Un arrêt forcé de l'outil peut laisser le dernier plan actif. Relancer l'outil déclenche la
récupération prévue. Un plan externe distinct observé reste prioritaire.
Les performances, la température et la consommation d'un jeu dépendent du PC. Aucun gain de FPS
n'est annoncé par l'interface.
