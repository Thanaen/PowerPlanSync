# Vérifier le clavier et l'accessibilité

Les actions principales doivent rester utilisables sans souris et conserver un focus visible. Les erreurs et les changements d'état doivent être compréhensibles dans la vraie interface Windows.

## Sub-features

- access-keyboard. Parcours clavier, listes et activation des boutons.
- access-error. Erreur lisible avec saisie et focus conservés.
- access-scroll. Petite hauteur et longue liste sans contrôles comprimés.
- access-scale. Échelles 100 %, 150 %, 200 % et texte Windows agrandi.
- access-contrast. Thème Windows et contraste élevé.
- access-narrator. Annonces réellement entendues par le Narrateur.

## How to get to it (user POV)

La fenêtre principale, **Ajouter un logiciel**, **Modifier**, **Réglages** et le menu de l'icône sont des entrées distinctes. Le dialogue expose **Parcourir…**, **Application ouverte**, **Fichier** (chemin complet en lecture seule), **Nom affiché**, **Plan pendant l’exécution**, **Ajouter** et **Annuler**. Dans **Modifier**, le bouton principal est **Enregistrer**.

## Driving it with sky

Preconditions: séance démo Doctor valide, fixture de règle disponible. Enregistrer les réglages Windows initiaux avant toute variation d'échelle ou de contraste autorisée. Ne pas modifier ces réglages pendant une autre campagne.

- **Clavier.** Utiliser `sky.press_key({window: ppsWindow, key: 'Tab'})`, puis lire `focused_element` et la capture. Progresser contrôle par contrôle. Activer un bouton observé avec `Return` ou `space` et vérifier son effet. Ne pas traiter un ordre de tabulation supposé comme un résultat.
- **Annuler.** Ouvrir **Ajouter un logiciel**, saisir un brouillon dans **Nom affiché**, puis utiliser `Escape`. Le dialogue se ferme, le focus revient à **Ajouter un logiciel** et aucune règle n'est enregistrée. Revenir à une nouvelle capture avant l'action suivante.
- **Erreur.** Activer **Ajouter** sans fichier, sans nom ni plan. Capturer les messages sous les champs (« Choisissez le fichier du logiciel. », « Saisissez le nom affiché. », « Choisissez un plan. »), la saisie restante et le focus. Compléter les champs puis ajouter. Vérifier que les mises à jour du statut ne réinitialisent pas le brouillon.
- **Défilement.** Avec une liste longue, réduire la hauteur par le bord réel de la fenêtre observée. Parcourir jusqu'à la dernière ligne de **Logiciels associés** et jusqu'à **Quitter** en bas de la barre latérale. Utiliser `sky.scroll` uniquement avec des coordonnées et un screenshotId de la capture courante.
- **Échelle et contraste.** Exécuter les variantes Windows autorisées une à une, capturer l'échelle déclarée et le résultat réel. Vérifier les noms non tronqués, le focus, les dialogues et les messages. Restaurer ensuite les valeurs initiales et relire la fenêtre.
- **Narrateur.** Effectuer une séance avec le Narrateur réellement actif. Noter les noms, rôles, états et annonces de changement entendus. Si aucune observation vocale n'est disponible, marquer access-narrator non exécuté, même si l'arbre sky paraît complet.

## Gotchas

Un arbre accessible et une capture ne prouvent pas à eux seuls la lecture vocale. Une inspection native n'autorise pas à inventer des APIs UI Automation. Les raccourcis Windows interdits par le plugin restent interdits ; ne pas employer la boîte Exécuter. Une capture prise avant la fin d'une animation n'est pas une preuve de contrôle manquant. Restaurer les réglages d'affichage modifiés par l'essai et conserver leurs valeurs avant/après dans le rapport.
