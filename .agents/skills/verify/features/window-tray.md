# Vérifier la fenêtre et la zone de notification

Fermer la fenêtre arrête son processus `--window` et conserve le suivi et son icône dans la zone de notification. Quitter termine le processus de suivi et celui de la fenêtre après le retour protégé.

## Sub-features

- window-hide. Masquer la fenêtre sans arrêter le processus de suivi.
- window-reopen. Ouvrir depuis la zone de notification.
- window-second-launch. Réouvrir l'instance propriétaire lors d'un second lancement réel.
- window-quit. Quitter par la barre latérale ou le menu de l'icône.
- window-overhead. Mesurer séparément les états masqué et ouvert.

## How to get to it (user POV)

Utiliser **Fermer** dans la barre de titre, une autre fermeture de la fenêtre ou **Ouvrir PowerPlanSync** dans le menu de l'icône. **Quitter** est en bas de la barre latérale (Ctrl+Q) ; **Quitter PowerPlanSync** se trouve dans le menu.

## Driving it with sky

Preconditions: instance démo appartenant à la séance, Doctor valide et règle simulée configurée. Aucune mesure ne tourne pendant les manipulations.

- **Masquer.** Cliquer **Fermer** dans la barre de titre. Rafraîchir `sky.list_windows()` et constater l'absence de la fenêtre principale. Vérifier en lecture seule la sortie du processus de fenêtre, puis la présence du même PID de suivi avec la même heure de création. Au premier masquage, capturer la notification « PowerPlanSync reste ouvert ». Ne pas appeler cette action Quitter.
- **Réouvrir.** Observer la zone de notification Windows avec sky, y repérer PowerPlanSync et ouvrir son menu. Sélectionner **Ouvrir PowerPlanSync**. Rappeler `Get-PpsWindowProcess` : la fenêtre appartient à un nouveau processus `--window`, enfant du même PID de suivi. Retrouver la nouvelle fenêtre retournée, lire son état et comparer les règles persistées.
- **Autre fermeture.** Répéter par une autre fermeture observée, par exemple `Alt+F4` si le plugin l'autorise. Le résultat reste un suivi actif avec possibilité de réouverture.
- **Quitter.** Dans une séance démo sans cycle, utiliser **Quitter** de la barre latérale, puis confirmer la sortie des deux PID, suivi et fenêtre. Répéter avec **Quitter PowerPlanSync** du menu de l'icône dans une nouvelle séance possédée. Si un cycle est actif, conserver les preuves du retour avant de conclure.
- **Second lancement réel.** Dans une campagne réelle distincte autorisée et sans cycle, lancer exactement le même binaire et le même dossier. Le premier PID demeure propriétaire, la fenêtre se rouvre dans un processus `--window` enfant de ce PID et le second processus sort. Ne pas utiliser cette attente en démo, dont l'exclusivité est portée par le dossier.
- **Consommation.** Après préparation et fermeture constatée de la fenêtre, exécuter `measure-overhead.ps1` selon le skill principal avec `-WindowState hidden` et le PID de suivi. Refaire une mesure avec `open` après réouverture constatée. Le script ne compte pas le processus de fenêtre : le signaler dans le rapport. Conserver vingt règles et éviter les interactions pendant chaque période.

## Gotchas

La fenêtre réouverte appartient à un nouveau processus et a un nouvel identifiant sky. La barre des tâches peut masquer l'icône dans son débordement ; observer avant de la chercher. Ne pas fixer ses coordonnées. Les indices de menu ne survivent pas à sa fermeture. Le script de mesure ne vérifie pas l'état visuel déclaré. L'absence de fenêtre n'est pas l'absence du processus de suivi. Ne jamais arrêter toutes les instances par nom.
