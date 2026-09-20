# Cent cycles Windows sur l'application intégrée

Le 20 septembre 2026, `scripts/verify-windows.ps1` a exécuté 100 cycles réels.
Le témoin léger est resté actif au moins dix secondes par cycle. Aucun jeu n'a été lancé.
Le rapport brut est [v1-100-cycles.json](v1-100-cycles.json).

| Mesure | Résultat | Limite SC-002 |
|---|---:|---:|
| Cycles réussis | 100 sur 100 | 100 |
| Activation p95 depuis la création réelle | 1 007,464 ms | 2 000 ms |
| Retour p95 depuis la sortie réelle | 4 056,666 ms | 5 000 ms |

L'essai utilise le binaire copié dans `.audit/tested-v1`, SHA-256
`FC694AD160EE968CD99D53359ADD5DB8D18751631D0E7676BF62115B561C42DB`.
Il comprend le moteur intégré et les corrections de session jusqu'à `165f984`.
Les corrections suivantes de présentation et de libération des ressources après
un échec d'initialisation ne sont pas couvertes par ce hash.

Le plan cible était Utilisation normale et le retour était Haute performance,
également actif avant l'essai. Windows a confirmé ce plan final. L'application
fonctionnait avec les droits ordinaires du compte interactif, hors du bac à sable
de l'agent. La machine est Windows 11 25H2, build 26200, Ryzen 7 9800X3D,
16 processeurs logiques et 32 Go de mémoire.

Les phases de sortie varient naturellement entre les observations. Aucun
décalage de phase déterministe n'est imposé par ce script. Ce résultat valide les
cent transitions SC-002, pas cent parcours d'interface ni un gain de performances.
