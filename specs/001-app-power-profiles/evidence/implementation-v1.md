# Intégration de la première version

Le 20 septembre 2026, le moteur, les adaptateurs Windows, le stockage et l'interface
sont reliés dans le binaire principal. Les preuves de faisabilité précédentes restent
historiques ; elles ne décrivent plus seules l'état de l'implémentation.

## Vérifications automatisées

La suite intégrée passe : 37 tests de bibliothèque, 6 scénarios moteur, 17 scénarios de
récupération et 6 scénarios de stockage, soit 66 tests. Clippy sur toutes les cibles passe
avec les avertissements refusés. Les corrections récentes couvrent notamment :

- acquisition préparée mais jamais confirmée : aucun retour indu au redémarrage ;
- modification du seul retour : conservation de la grâce du cycle actif ;
- plan manuel : action Reprendre disponible et règle non présentée comme active ;
- veille : aucune restauration créée par une fausse perte de session ;
- chemins Windows UTF-16 conservés dans le stockage et le démarrage facultatif ;
- démonstration isolée même si son dossier parent est celui d'une configuration réelle ;
- proposition du plan normal sans confirmation ni activation implicite.

Les tâches de réalisation cochées correspondent à du code intégré et à ces contrôles.
Les tâches de validation Windows, de ressources et d'accessibilité complète restent séparées.

## Parcours natif de démonstration

Le binaire optimisé a été lancé sur le bureau Windows avec `--demo --data-dir` isolé.
Le sélecteur Windows a choisi `target/release/examples/demo_target.exe`. Son nom et
son chemin se sont remplis automatiquement. La règle Haute performance simulée a été
enregistrée, le retour Équilibré confirmé, puis l'automatisation activée explicitement.

Lancer le logiciel simulé donne le statut Automatisation active et le plan Haute
performance simulé. Arrêter donne Retour dans quelques secondes, puis En attente et
Équilibré simulé ; le journal est effacé. La commande Quitter arrête le processus.
Le GUID Windows réel est resté Haute performance pendant cette démonstration.

## Accessibilité du prototype

Le Narrateur Windows a réellement prononcé le nom du champ Logiciel, la valeur du
sélecteur de plan et les états désactivé/activé de la case de suivi. Son récapitulatif
vocal est conservé dans [la capture](accessibility/narrator-probe-announcements.png).
L'indication de focus du pilote d'inspection restait parfois périmée alors que la parole
et le focus visuel évoluaient. Ce résultat résout cette incertitude du prototype, sans
valider à lui seul tous les parcours de l'application finale.

Les essais réels de bascule, la campagne de ressources, les thèmes et les échelles
complètes, ainsi que le parcours de cinq personnes restent à consigner dans leurs preuves dédiées.
