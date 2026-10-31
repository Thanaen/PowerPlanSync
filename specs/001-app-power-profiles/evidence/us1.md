# US1 : cycle réel avec l'exécutable témoin (T019)

Date : 31 octobre 2026. Machine : Ryzen 7 9800X3D, 16 processeurs logiques, 32 Go,
Windows 11 Pro 26H2 build 26300.9711, compte standard. Aucun jeu lancé : le logiciel suivi est
`demo_target.exe`, compilé avec le dépôt.

Binaires testés, compilés depuis `main` (`6847ce7`) avec les seuls correctifs d'interface de la
PR #30, qui ne touchent ni le moteur ni le coordinateur. Les essais lancent l'application sans
`--gpui`, donc avec l'interface par défaut ; le cycle passe par le même moteur.

```text
power-plan-sync.exe  DCD96027F8BFACADFBA5FDC53B80DDD83267B979B81E1729E95D9A129F17003C
demo_target.exe      3C818F825667E7922CC01EAD9DC7A3E9DE94DF16405B2A1EA8AFE37100E8AFF9
```

Plans installés : Utilisation normale, Haute performance (plan initial du PC), Économie
d'énergie et Performances optimales. Les essais ont changé le plan réel, un seul à la fois. Le
plan initial a été rétabli et relu après chaque cas et à la fin de chaque campagne.

## Comportements

`scripts/verify-behaviors.ps1 -AllowPowerSwitch`, dossier de données isolé par cas. Le
chronomètre part de la création ou de la sortie réelle du processus témoin, lue par
`System.Diagnostics.Process`, jamais de sa détection. Rapport :
[behaviors/20261031-162909-fa4c40fa/result.json](behaviors/20261031-162909-fa4c40fa/result.json).

| Cas | Attendu | Résultat |
| --- | --- | --- |
| Deux instances | Le plan cible reste après la fin de la première, retour après la dernière | Réussi : activation 741 ms, retour 3 970 ms |
| Homonyme dans un autre dossier | Aucun cycle pendant dix secondes | Réussi |
| Lanceur qui reste ouvert | Retour à la fin de l'enfant suivi, lanceur encore ouvert | Réussi : activation 725 ms, retour 3 642 ms |
| Arrêt forcé du témoin | Retour après la sortie forcée | Réussi : activation 724 ms, retour 3 943 ms |
| Plan changé dans Windows pendant le cycle | Le plan choisi reste après la fin du témoin | Réussi |
| Arrêt forcé de l'application pendant un cycle | Au relancement, retour au plan d'avant le cycle | Réussi : reprise en 520 ms |
| Deux règles (voir [us3.md](us3.md)) | Passage d'un gagnant à l'autre sans plan intermédiaire | Réussi |

## Cent cycles

`scripts/verify-windows.ps1 -Cycles 100 -AllowPowerSwitch` : plan cible Utilisation normale,
retour Haute performance. Le témoin vit au moins dix secondes par cycle ; les sorties tombent
naturellement à des phases variées de la seconde de sondage. Rapport :
[windows-cycles/v1-100-cycles-20261031.json](windows-cycles/v1-100-cycles-20261031.json).

| Mesure | Résultat | Limite SC-002 |
| --- | ---: | ---: |
| Cycles réussis | 100 sur 100 | 100 |
| Activation p95 depuis la création réelle | 1 009,3 ms | 2 000 ms |
| Retour p95 depuis la sortie réelle | 4 063,6 ms | 5 000 ms |

Les résultats rejoignent ceux du 20 septembre (1 007,5 ms et 4 056,7 ms), sur un moteur qui a
reçu depuis la relecture des réglages (T061). Le retour inclut la grâce de trois secondes.
Ces mesures valident les transitions SC-002, pas un gain de performances ni une compatibilité
avec un jeu.

## Banc

Le banc vit dans `scripts/verify-behaviors.ps1` et `scripts/verify-windows.ps1`, pas dans
`tests/windows_integration.rs` comme T019 le prévoyait : un test Cargo changerait le plan réel
de toute machine qui lance `cargo test`, y compris la CI Windows. Les scripts exigent
`-AllowPowerSwitch` et restaurent le plan initial dans un bloc `finally`.
