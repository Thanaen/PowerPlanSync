# Instructions du projet

Lire `.specify/memory/constitution.md` et `.specify/feature.json` avant tout travail produit.
Utiliser les skills spec-kit installés dans `.agents/skills` pour la conception et sa suite.
Leur contenu officiel est conservé sans adaptation locale. La conception canonique reste dans
`.specify` et `specs`. Ne pas créer un second plan produit dans un autre répertoire.

L'implémentation est autorisée. Les sondes Windows et UI existent et trois cycles réels
ont été vérifiés avec un exécutable témoin. Ne jamais lancer Battlefield pour les essais.
Suivre l'ordre actualisé de `specs/001-app-power-profiles/tasks.md`.
Les fondations indépendantes de l'interface peuvent avancer pendant les vérifications
d'accessibilité. L'interface définitive et la livraison conservent leurs critères complets.
Ne pas confondre une cible de mesure avec une valeur mesurée. Sérialiser les essais de plans
réels et restaurer le plan initial du PC dans un bloc de nettoyage vérifié.

<!-- BEGIN pstack model configuration -->
# pstack model configuration

feature, refactoring: gpt-5.6-sol
bug-fix: gpt-5.6-sol
perf-issue: gpt-5.6-sol
hillclimb: gpt-5.6-sol
judgment and prose: gpt-5.6-sol
strongest judgment: gpt-6-astra
how explorer: gpt-5.6-sol
how explainer: gpt-5.6-terra
how critics: gpt-5.6-sol, gpt-5.5, gpt-5.6-terra
why investigators: gpt-5.6-sol
why synthesizer: gpt-5.6-sol
reflect tooling: gpt-5.6-terra
reflect judgment, divergent, synthesizer: gpt-5.6-sol
arena runners: gpt-5.6-sol, gpt-5.6-terra, gpt-5.5
arena cross-judge pool: gpt-5.6-sol, gpt-5.5, gpt-5.6-terra
swarm workers: gpt-5.6-terra
architect runners: gpt-6-astra, gpt-5.6-sol, gpt-5.6-terra
interrogate reviewers: gpt-5.6-sol, gpt-5.5, gpt-5.6-terra
<!-- END pstack model configuration -->
