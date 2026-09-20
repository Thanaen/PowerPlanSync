# Configurer les logiciels

Une règle associe un chemin de logiciel vérifiable à un plan. L'utilisateur confirme séparément le retour avant d'activer l'automatisation.

## Sub-features

- configuration-file. Ajouter avec le sélecteur de fichier natif.
- configuration-running. Ajouter depuis une application ouverte vérifiable.
- configuration-edit. Modifier sans changer l'identité ni réinitialiser le plan choisi.
- configuration-order. Activer, désactiver, monter et descendre les règles.
- configuration-return. Confirmer le plan de retour et déverrouiller l'activation.
- configuration-validation. Préserver la saisie lors d'un chemin invalide ou d'un doublon.

## How to get to it (user POV)

Depuis la fenêtre principale, **Ajouter un logiciel** ouvre l'éditeur. **Parcourir les fichiers** et **Choisir une application ouverte** sont ses deux entrées. Chaque règle propose **Modifier**, **Supprimer**, **Monter**, **Descendre** et son activation. **Réglages** contient **Plan de retour** et **Confirmer ce plan de retour**.

## Driving it with sky

Preconditions: Launch et Doctor réussis en démo, données neuves, chemin du témoin connu et plan Windows initial enregistré.

- **Ajouter par fichier.** Cliquer **Ajouter un logiciel**, observer, puis **Parcourir les fichiers**. Cibler la fenêtre retournée **Choisir un logiciel**. Saisir le chemin du témoin dans son champ de fichier observé, valider, puis relire la fenêtre principale. **Chemin complet du logiciel** contient le chemin choisi.
- **Nommer et associer.** Saisir `Témoin de vérification` dans **Nom du logiciel**. Ouvrir **Plan pendant l'exécution**, observer les choix puis sélectionner **Haute performance (simulation)**. Cliquer **Enregistrer la règle**. Capturer la ligne et copier `simulation/config.json`. L'automatisation reste désactivée.
- **Confirmer le retour.** Choisir **Équilibré (simulation)** dans **Plan de retour** ou confirmer la proposition avec **Confirmer ce plan de retour**. Vérifier la valeur persistée, puis la disponibilité de **Activer l'automatisation**. Un texte de ComboBox affiché ne prouve pas une confirmation.
- **Modifier.** Cliquer **Modifier Témoin de vérification**. Observer le plan présélectionné, changer le nom et enregistrer. Le fichier conserve l'identifiant de la règle et son plan si celui-ci n'a pas été changé.
- **Autre entrée.** Ouvrir **Choisir une application ouverte**, observer **Applications ouvertes avec chemin vérifié**, sélectionner la proposition simulée et terminer l'ajout. En mode réel autorisé, ne choisir qu'un témoin possédé par la séance.
- **Valider.** Essayer un chemin inexistant puis un doublon du même chemin. L'erreur reste visible, le brouillon reste présent et le dernier fichier valide reste identique. Annuler avec **Annuler les modifications** ou `Escape`.
- **Priorité.** Avec deux chemins distincts de fixtures, utiliser les noms accessibles **Monter** ou **Descendre** suivis du nom du logiciel. Comparer l'ordre affiché à l'ordre persisté et vérifier les boutons de bord désactivés. Tester la case **Activer** suivie du nom.

## Gotchas

Les listes et les dialogues changent leurs indices après chaque action. La fenêtre du sélecteur appartient au parcours mais possède un autre objet Window. Un chemin normalisé peut différer en casse dans le fichier. La preuve doit garder le chemin complet, pas seulement le nom de fichier. **Démarrer avec Windows** est indisponible en démo et avec un dossier personnalisé. Ne pas toucher au démarrage réel pour vérifier l'éditeur. La suppression d'une fixture UI reste limitée aux données possédées par l'essai et suit les instructions de confirmation applicables.
