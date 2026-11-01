# Vérifier les cycles et le réglage Windows

L'automatisation applique le plan de la règle prioritaire, puis revient au retour réservé. Un autre plan observé dans Windows suspend son action pour le cycle.

## Sub-features

- cycles-demo. Activation simulée et retour après fermeture.
- cycles-pause. Pause et reprise explicites.
- cycles-reserved-return. Retour conservé pendant un cycle déjà commencé.
- cycles-manual. Priorité du changement Windows et reprise volontaire.
- cycles-failure. Erreur de retour, nouvel essai et journal conservé.
- cycles-live. Campagne sérialisée avec le témoin réel et retour au plan initial du PC.

## How to get to it (user POV)

La page **Logiciels** propose **Activer l’automatisation** à l'étape 3 de **Pour commencer**, puis **Mettre en pause** ou **Reprendre** (Ctrl+Alt+P). En démo, **Lancer le logiciel simulé** et **Arrêter le logiciel simulé** remplacent le témoin réel. Après un plan choisi dans Windows, la page propose **Reprendre l’automatisation**. En cas d'erreur, utiliser les actions de reprise effectivement présentes. Le menu de l'icône offre aussi **Mettre en pause** ou **Reprendre**, et **Quitter PowerPlanSync**.

## Driving it with sky

Preconditions: configuration-file et configuration-return réussis. Utiliser deux plans simulés distincts. Aucune autre campagne ne modifie Windows.

- **Activer.** Cliquer **Activer l’automatisation**, puis **Lancer le logiciel simulé**. Observer **Plan actif dans Windows**, le statut (« Automatisation active · … est ouvert »), la ligne du plan de retour et le badge **Appliqué** de la ligne. Lire le journal simulé. Le plan Windows réel reste celui enregistré par Doctor.
- **Fermer le logiciel.** Cliquer **Arrêter le logiciel simulé**. Observer l'attente de retour puis le plan simulé de retour. Mesurer le délai réellement observé si ce délai fait partie du verdict. Le journal ne conserve plus un cycle actif.
- **Pause et reprise.** Relancer le logiciel simulé, cliquer **Mettre en pause** et constater le retour. Cliquer **Reprendre**. Le logiciel encore présent déclenche une nouvelle décision. Après un plan choisi dans Windows, **Reprendre l’automatisation** reste accessible et la ligne porte **Suspendu**, pas **Appliqué**, avant la reprise.
- **Retour réservé.** Pendant un cycle simulé, changer **Plan de retour** dans **Réglages**. Capturer le message « À la fin du cycle en cours, PowerPlanSync reviendra au plan … ». Arrêter le logiciel et vérifier que le cycle utilise son ancien retour. Le cycle suivant utilise le nouveau.
- **Plans réels.** Exécuter séparément la commande `verify-windows.ps1` du skill principal. Lire chaque résultat et la confirmation finale du plan initial. Pour une vérification manuelle de priorité Windows déjà autorisée, utiliser uniquement le témoin, observer le plan externe, puis **Reprendre l’automatisation**. Sauvegarder le plan initial dans un nettoyage vérifié. Ne pas improviser cette action pendant une mesure.
- **Erreur contrôlée.** Un scénario de panne doit disposer de sa propre fixture autorisée. Capturer l'état bloqué et les actions **Réessayer**, **Ouvrir les options d’alimentation** et **Quitter quand même** lorsqu'elles apparaissent. Le dernier choix conserve la récupération et ne constitue jamais un succès de retour. Sans fixture, marquer ce parcours non exécuté et citer séparément les tests Rust.

## Gotchas

La veille ne déclenche pas un retour à elle seule. Le réveil relit plan, source et processus. Un GUID observé identique ne prouve pas l'absence d'écritures intermédiaires. Ne pas mélanger preuve du moteur, campagne de cent cycles et parcours UI. `--data-dir` seul utilise les vrais plans. Ne jamais lancer Battlefield. Ne pas paralléliser deux essais qui peuvent appliquer un plan.
