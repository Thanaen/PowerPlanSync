# Configurer les logiciels

Une règle associe un chemin de logiciel vérifiable à un plan. L'utilisateur confirme séparément le retour avant d'activer l'automatisation.

## Sub-features

- configuration-file. Ajouter avec le sélecteur de fichier natif.
- configuration-running. Ajouter depuis une application ouverte vérifiable.
- configuration-edit. Modifier sans changer l'identité ni réinitialiser le plan choisi.
- configuration-order. Suivre ou non, monter et descendre les logiciels.
- configuration-return. Confirmer le plan de retour et déverrouiller l'activation.
- configuration-validation. Préserver la saisie lors d'un champ manquant ou d'un doublon.

## How to get to it (user POV)

Sur la page **Logiciels**, **Ajouter un logiciel** (Ctrl+N) ouvre le dialogue du même nom. **Parcourir…** et **Application ouverte** sont ses deux sources. Chaque ligne de **Logiciels associés** propose l'interrupteur **Suivre**, **Monter**, **Descendre**, **Modifier** et **Supprimer**. Sur des données neuves, **Pour commencer** contient l'étape **Confirmer le plan de retour**, avec la liste **Plan de retour** et le bouton **Confirmer**. Ensuite, **Réglages** garde **Plan de retour** sous **Après fermeture**.

## Driving it with sky

Preconditions: Launch et Doctor réussis en démo, données neuves, chemin du témoin connu et plan Windows initial enregistré.

- **Ajouter par fichier.** Cliquer **Ajouter un logiciel**, observer, puis **Parcourir…**. Cibler la fenêtre retournée **Choisir un logiciel**. Saisir le chemin du témoin dans son champ de fichier observé, valider, puis relire le dialogue dans la fenêtre principale. **Fichier** affiche le chemin complet choisi.
- **Nommer et associer.** Saisir `Témoin de vérification` dans **Nom affiché**. Ouvrir **Plan pendant l’exécution**, observer les choix puis sélectionner **Haute performance (simulation)**. Cliquer **Ajouter**. Capturer la ligne et copier `simulation/config.json`. L'automatisation reste désactivée.
- **Confirmer le retour.** Dans l'étape **Confirmer le plan de retour**, choisir **Équilibré (simulation)** dans **Plan de retour**, puis cliquer **Confirmer**. Vérifier la valeur persistée, puis la disponibilité de **Activer l’automatisation**. Un plan affiché dans la liste ne prouve pas une confirmation.
- **Modifier.** Cliquer **Modifier Témoin de vérification**. Le dialogue porte le même titre. Observer le plan présélectionné, changer le nom et cliquer **Enregistrer**. Le fichier conserve l'identifiant de la règle et son plan si celui-ci n'a pas été changé.
- **Autre entrée.** Dans **Ajouter un logiciel**, cliquer **Application ouverte**, observer la liste **Applications ouvertes** et son champ **Filtrer les applications ouvertes**, sélectionner une proposition observée et terminer l'ajout. Une entrée « Fichier inaccessible · impossible à suivre » reste désactivée. En mode réel autorisé, ne choisir qu'un témoin possédé par la séance.
- **Valider.** Activer **Ajouter** avec un champ manquant, puis choisir de nouveau le fichier d'un logiciel déjà associé. Le message reste visible, sous le champ ou en tête du dialogue, le brouillon reste présent et le dernier fichier valide reste identique. Fermer avec **Annuler** ou `Escape`.
- **Priorité.** Avec deux chemins distincts de fixtures, utiliser les noms accessibles **Monter** ou **Descendre** suivis du nom du logiciel. Comparer l'ordre affiché à l'ordre persisté et vérifier les boutons de bord désactivés. Tester l'interrupteur **Suivre** suivi du nom.

## Gotchas

Les listes et les dialogues changent leurs indices après chaque action. La fenêtre du sélecteur appartient au parcours mais possède un autre objet Window et un autre processus, celui du suivi. Un chemin normalisé peut différer en casse dans le fichier. La preuve doit garder le chemin complet, pas seulement le nom de fichier. **Démarrer avec Windows** est indisponible en démo et avec un dossier personnalisé. Ne pas toucher au démarrage réel pour vérifier l'éditeur. La suppression d'une fixture UI reste limitée aux données possédées par l'essai et suit les instructions de confirmation applicables.
