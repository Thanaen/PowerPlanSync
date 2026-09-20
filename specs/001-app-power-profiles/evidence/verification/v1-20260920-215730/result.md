# Parcours natif du binaire intégré

Le 20 septembre 2026, le skill local `verify` a été exécuté sur un dossier de
démonstration neuf. `doctor.json` identifie le PID, le binaire et son SHA-256.

Le parcours a ajouté « Logiciel témoin », saisi le chemin du petit exécutable,
choisi Haute performance simulée et enregistré la règle. Le plan Équilibré était
proposé mais non confirmé. L'automatisation est restée indisponible jusqu'au clic
sur « Confirmer ce plan de retour ». Son activation a ensuite persisté.

Le démarrage simulé a affiché Haute performance et Automatisation active. L'arrêt
a affiché la grâce, puis Équilibré et En attente. Le Narrateur a annoncé
« État : Automatisation active. Plan actif : Haute performance (simulation) »
avec le nom du logiciel. La transcription réelle figure dans
[narrator-live-status.png](narrator-live-status.png).

La section À propos affiche le composant officiel Slint et sa version 1.18.0.
Quitter PowerPlanSync a arrêté le PID 24980. Le plan Windows observé avant et
après est Haute performance, GUID `8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c`.
Le Narrateur lancé pour cet essai a également été arrêté.

Après copie de la configuration finale, seul le dossier temporaire de cette
démo sous `.audit/verify-data/` a été supprimé. Les captures, l'arbre accessible,
les configurations et Doctor sont conservés ici. Launch, Doctor, configuration,
Evidence et Cleanup du skill sont donc exercés.

Ce parcours utilise le thème sombre et l'échelle courante. La matrice complète
des thèmes et échelles, cinq utilisateurs non experts et Battlefield 6 ne sont
pas validés par cet essai. Le sélecteur de fichiers natif avait été exercé dans
la séance précédente ; cette séance utilise la saisie du chemin dans l'éditeur.
