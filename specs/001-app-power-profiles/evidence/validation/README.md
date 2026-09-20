# Preuves du premier lot

`fmt.txt` est vide parce que le contrôle de formatage a réussi sans sortie.
`tests.txt`, `clippy.txt` et `release-build.txt` proviennent de la vérification de la racine.
Les sources du worktree de construction et du checkout principal sont identiques pour ce lot.

`read-only/` contient la vérification du témoin sans changement de plan.
`power-cycles/` contient les trois cycles réels et les plans avant/après.
`mutex-conflict.txt` démontre le refus d'un second contrôleur avant écriture.

Le wrapper reproductible est `scripts/verify-prototypes.ps1`. Les bascules nécessitent
`-AllowPowerSwitch` et s'exécutent une seule séance à la fois. Les chemins temporaires du témoin
sont créés pour chaque exécution puis nettoyés. Aucun test ne lance Battlefield.
