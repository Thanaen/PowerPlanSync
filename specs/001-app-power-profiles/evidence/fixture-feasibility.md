# Vérification de l'exécutable témoin

`demo_target.exe` dort pendant une durée bornée. Il ne charge aucun jeu et ne simule pas de charge CPU.
Son mode lanceur démarre un enfant, attend et récolte son résultat. Les durées sont validées avant le lancement.

Le script `scripts/verify-prototypes.ps1`, sans option de bascule, a confirmé les observations suivantes.

| Observation | Instances au chemin configuré | Homonyme ailleurs | Lanceur distinct vivant |
| --- | --- | --- | --- |
| Initiale | 2 | 1 | Oui |
| Intermédiaire | 1 | 0 | Oui |
| Finale | 0 | 0 | Oui |

La comparaison porte sur les chemins, avec les règles de casse Windows. Un homonyme ne contribue
pas au compte du chemin configuré. Le lanceur reste présent après la dernière fermeture du témoin.
Le plan Windows reste inchangé pendant cet essai en lecture seule.

Voir la [trace brute](validation/read-only/windows-probe.txt) et les fichiers de plan initial/final voisins.
La campagne de récupération, les erreurs de processus protégé et les arrêts forcés du produit
restent à implémenter. Aucun lancement de Battlefield 6 n'a eu lieu et aucune compatibilité
anti-triche n'est déduite de ce test.
