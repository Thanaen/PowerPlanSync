# Essai Windows du 20 septembre 2026

La sonde lit quatre plans existants et leurs noms français. Elle ouvre les processus avec
`PROCESS_QUERY_LIMITED_INFORMATION`, sans injection, puis vérifie le chemin complet.
Le compte utilisé pour les bascules réelles n'est pas élevé.

La première tentative dans le bac à sable a été refusée avant toute bascule.
L'essai suivant s'est exécuté dans la session Windows ordinaire, toujours sans droits administrateur.
Le plan initial de la séance était **Haute performance**. Pour reproduire le parcours demandé,
la préparation a sélectionné **Utilisation normale**, puis chaque témoin a déclenché Haute performance.
Le plan initial de la séance a été restauré après les trois cycles.

| Cycle | Décalage du lancement | Création réelle vers cible confirmée | Sortie réelle vers retour confirmé |
| --- | --- | --- | --- |
| 1 | 170 ms | 837 ms | 4 081 ms |
| 2 | 530 ms | 495 ms | 3 740 ms |
| 3 | 890 ms | 136 ms | 3 385 ms |

Le témoin vit dix secondes. Présence et disparition sont observées sur des ticks monotones
d'une seconde. La grâce de trois secondes commence à l'observation d'absence. Les heures de
création et de sortie proviennent de Windows et servent au calcul des délais réels.
Les trois essais passent les seuils individuels de deux et cinq secondes. Ils ne constituent
pas la campagne de cent cycles nécessaire au p95 de SC-002.

Les GUID ont été relus après les écritures. Les preuves brutes sont dans
[power-cycles/windows-probe.txt](validation/power-cycles/windows-probe.txt),
[session-original-plan.txt](validation/power-cycles/session-original-plan.txt) et
[session-final-plan.txt](validation/power-cycles/session-final-plan.txt).

Un autre processus détenant `Global\PowerPlanSync.Probe.Controller` fait refuser la sonde
avant toute bascule. Le plan reste inchangé. Voir [mutex-conflict.txt](validation/mutex-conflict.txt).
Ce contrôle porte sur deux processus du même compte. Les comptes et sessions multiples restent à tester.

La sonde utilise une restauration de fin de test et le script ajoute un `finally` indépendant.
Cette protection de séance n'est pas le journal de récupération du futur produit et ne résiste
pas à l'arrêt brutal de la machine. Le respect des changements externes reste une tâche du coordinateur.
