# PowerPlanSync Constitution

## Core Principles

### I. La spécification précède le code

Toute fonctionnalité DOIT suivre le processus spec-kit. Les exigences résident dans
`specs/<feature>/spec.md`, les choix techniques dans `plan.md` et `research.md`, et le travail
dans `tasks.md`. Une modification du comportement commence par mettre ces documents à jour.
Un prototype répond à une question de faisabilité inscrite dans les tâches. Il ne remplace pas
la spécification. Aucun second plan produit ne fait autorité en dehors de spec-kit.

### II. Le réglage de l'utilisateur reste prioritaire

L'application DOIT expliquer quel plan elle applique et pourquoi. Elle DOIT préparer une
récupération durable avant toute modification. Une modification externe observée suspend son
action pour le cycle courant. La restauration relit le plan actif avant d'agir et vérifie le
résultat. Les limites de cette protection face aux changements simultanés DOIVENT être documentées.
Un échec de lecture, d'écriture ou de restauration reste visible et ne devient jamais un succès.

### III. Rust et un coût mesuré

La logique produit et l'intégration Windows DOIVENT être écrites en Rust. Un langage déclaratif
d'interface compilé avec Rust est admis. Aucun navigateur embarqué, service distant, pilote ou
service Windows permanent ne fait partie du premier périmètre. L'application DOIT mesurer son
coût en version optimisée, fenêtre fermée comme ouverte. Le choix de Rust ne constitue pas une
preuve de faible consommation. Les budgets chiffrés sont définis dans chaque spécification.

### IV. Une interface utilisable sans expertise système

Le parcours principal DOIT permettre de choisir un logiciel, un plan associé et un plan de retour
sans saisir une commande, un identifiant Windows ou une expression de détection. Tous les contrôles
DOIVENT être accessibles au clavier et au Narrateur. Le focus, les noms, les états, le contraste,
la mise à l'échelle et les messages d'erreur font partie de la validation. Le produit DOIT distinguer
les plans classiques des modes d'alimentation sans imposer ce vocabulaire à chaque action.

### V. Des droits limités et des preuves explicites

Le fonctionnement normal DOIT utiliser les droits de l'utilisateur. Aucun accès à la mémoire du jeu,
injection, contournement d'anti-triche ou privilège de débogage n'est autorisé. Les règles et diagnostics
restent locaux. Aucune télémétrie ni communication réseau n'est nécessaire au fonctionnement.
Chaque livraison DOIT fournir des tests des décisions sensibles, une vérification Windows réelle et
des mesures de consommation. La compatibilité Battlefield 6 ne peut être annoncée avant un essai réel.
Sur demande du 2026-09-20, les essais de développement utilisent un exécutable témoin léger.
L'essai Battlefield reste facultatif et différé. Son absence ne bloque pas la livraison générique.
Les gains de performances ne peuvent être annoncés avant des mesures comparables.

## Contraintes produit

Le premier périmètre vise Windows 11 x64, une session interactive active et les plans déjà installés.
Il conserve une seule instance de contrôle à l'échelle de la machine. Les autres sessions restent
inactives tant que ce contrôle est détenu. L'activation au démarrage de session est facultative.
L'application ne crée, ne supprime et ne modifie pas le contenu des plans Windows.

La fermeture forcée de l'application peut laisser le dernier plan actif. La récupération au prochain
lancement DOIT être spécifiée et testée. L'arrêt de Windows reste une restauration au mieux.
Les erreurs de droits, les processus inaccessibles et les plans manquants DOIVENT avoir une issue
visible, sans boucle de tentatives ni élévation silencieuse.

## Déroulement et validation

Suivre `speckit-constitution`, `speckit-specify`, `speckit-plan`, `speckit-tasks`, puis
`speckit-analyze` avant `speckit-implement`. Utiliser `speckit-clarify` lorsqu'une décision produit
n'a aucun défaut raisonnable. Les hypothèses retenues restent visibles dans la spécification.

Les tâches de faisabilité Windows et d'accessibilité passent avant l'interface définitive.
Les tests couvrent notamment les instances multiples, les changements manuels, les erreurs Windows,
la persistance interrompue et la récupération. Les scénarios réels utilisent d'abord un exécutable
de démonstration avec des données isolées. Toute mesure indique le matériel, la version, la durée
et la méthode. Un contrôle non exécuté porte la mention « non exécuté ».

## Governance

Cette constitution gouverne les choix du projet. Une proposition d'amendement explique sa raison,
les artefacts affectés et leurs migrations. Les exigences explicites de l'utilisateur priment.
Chaque plan et chaque revue vérifient les cinq principes et justifient les exceptions avant le code.
Une suppression ou redéfinition incompatible augmente la version majeure. Un nouveau principe
augmente la version mineure. Une précision sans changement de règle augmente le correctif.

**Version**: 1.0.1 | **Ratified**: 2026-09-20 | **Last Amended**: 2026-09-20
