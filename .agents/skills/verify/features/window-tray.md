# Vérifier la fenêtre et la zone de notification

Fermer la fenêtre conserve l'application et son suivi dans la zone de notification. Quitter termine l'instance après le retour protégé.

## Sub-features

- window-hide. Masquer la fenêtre sans arrêter le processus.
- window-reopen. Ouvrir depuis la zone de notification.
- window-second-launch. Réouvrir l'instance propriétaire lors d'un second lancement réel.
- window-quit. Quitter par la fenêtre ou le menu de notification.
- window-overhead. Mesurer séparément les états masqué et ouvert.

## How to get to it (user POV)

Utiliser **Fermer la fenêtre**, la fermeture standard de la fenêtre ou **Ouvrir PowerPlanSync** dans le menu de notification. **Quitter PowerPlanSync** est présent dans la fenêtre ; **Quitter** se trouve dans le menu.

## Driving it with sky

Preconditions: instance démo appartenant à la séance, Doctor valide et règle simulée configurée. Aucune mesure ne tourne pendant les manipulations.

- **Masquer.** Cliquer **Fermer la fenêtre**. Rafraîchir `sky.list_windows()` et constater l'absence de la fenêtre principale. Vérifier en lecture seule que le même PID et la même heure de création existent encore. Ne pas appeler cette action Quitter.
- **Réouvrir.** Observer la zone de notification Windows avec sky, y repérer PowerPlanSync et ouvrir son menu. Sélectionner **Ouvrir PowerPlanSync**. Retrouver la nouvelle fenêtre retournée, lire son état et comparer les règles persistées.
- **Autre fermeture.** Répéter par le bouton de fermeture standard observé. Le résultat reste une application active avec possibilité de réouverture.
- **Quitter.** Dans une séance démo sans cycle, utiliser **Quitter PowerPlanSync**, puis confirmer la sortie du PID. Répéter l'entrée **Quitter** du menu dans une nouvelle séance possédée. Si un cycle est actif, conserver les preuves du retour avant de conclure.
- **Second lancement réel.** Dans une campagne réelle distincte autorisée et sans cycle, lancer exactement le même binaire et le même dossier. Le premier PID demeure propriétaire, la fenêtre se rouvre et le second processus sort. Ne pas utiliser cette attente en démo, dont l'exclusivité est portée par le dossier.
- **Consommation.** Après préparation et fermeture constatée de la fenêtre, exécuter `measure-overhead.ps1` selon le skill principal avec `-WindowState hidden`. Refaire une mesure avec `open` après réouverture constatée. Conserver vingt règles et éviter les interactions pendant chaque période.

## Gotchas

La fenêtre réouverte peut avoir un nouvel identifiant sky. La barre des tâches peut masquer l'icône dans son débordement ; observer avant de la chercher. Ne pas fixer ses coordonnées. Les indices de menu ne survivent pas à sa fermeture. Le script de mesure ne vérifie pas l'état visuel déclaré. L'absence de fenêtre n'est pas l'absence de processus. Ne jamais arrêter toutes les instances par nom.
