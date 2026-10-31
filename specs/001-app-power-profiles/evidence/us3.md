# US3 : priorité entre logiciels (T038)

Date, machine et binaires : voir [us1.md](us1.md).

## Transition entre gagnants

Cas `two-rules-priority` de `scripts/verify-behaviors.ps1`. Deux copies du témoin dans deux
dossiers, deux règles dans l'ordre : la première vers Utilisation normale, la seconde vers
Économie d'énergie ; retour vers Haute performance, le plan initial. Pendant chaque transition,
le plan Windows est relu toutes les 50 ms, et tout plan autre que le précédent ou l'attendu fait
échouer le cas.

| Étape | Plan attendu | Délai mesuré |
| --- | --- | ---: |
| La seconde règle s'ouvre seule | Économie d'énergie | 714 ms depuis la création |
| La première règle s'ouvre aussi | Utilisation normale, sans passer par Haute performance | 282 ms depuis la création |
| La première règle se ferme | Économie d'énergie après la grâce, sans plan intermédiaire | 3 941 ms depuis la sortie |
| La seconde règle se ferme | Haute performance | 4 095 ms depuis la sortie |

Rapport : [behaviors/20261031-162909-fa4c40fa/result.json](behaviors/20261031-162909-fa4c40fa/result.json).

## Logiciels déplacés

Un exécutable du même nom dans un autre dossier ne déclenche pas la règle : cas
`homonym-other-path`, dix secondes sans changement de plan. Le réordonnancement des règles
et la désactivation sont couverts par `ordered_rules_recompute_immediately_for_explicit_edits`
(`tests/engine_scenarios.rs`).

**Non exécuté** : le scénario 4 d'US3 dans [spec.md](../spec.md). Quand l'ancienne règle ne
trouve plus le programme, **Modifier** propose de choisir le nouveau fichier
(`src/interface/rule_dialog.rs`, mode `Edit { missing: true }`), et le plan doit rester. Aucun
test ni essai réel ne le prouve encore. T038 reste ouverte pour ce point.
