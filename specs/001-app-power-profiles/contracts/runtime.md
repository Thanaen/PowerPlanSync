# Contrats d'exécution

## Frontière entre le moteur et Windows

Le moteur reçoit un snapshot et une action, puis produit un nouvel état et une intention.
Il ne lit pas Windows et n'écrit aucun fichier. Les adaptateurs convertissent les erreurs en
résultats typés avec le code Windows disponible pour le diagnostic.

| Opération prévue | Entrée | Résultat |
| --- | --- | --- |
| Lire les plans | Aucune | Catalogue de GUID et noms, ou erreur |
| Lire le plan actif | Aucune | Un GUID, ou erreur |
| Appliquer un plan | GUID encore présent | Code d'application puis GUID relu |
| Observer les processus | Règles activées et session | Instances vérifiées et observations incertaines |
| Enregistrer la configuration | Configuration validée | Succès durable ou erreur sans altération du dernier fichier valide |
| Préparer la transaction | Journal versionné | Succès durable préalable à l'appel Windows |
| Publier l'état | ViewState immuable | Affichage accessible, sans capacité directe de changer le plan |

Les callbacks Windows et UI transmettent des événements au coordinateur unique.
Les événements doublons sont tolérés. Une commande en cours empêche toute application concurrente.
Après une écriture, la lecture du plan fait autorité. Une notification tardive identique ne crée
pas une nouvelle bascule. Une valeur distincte déclenche la suspension, même si son origine est inconnue.

## Démarrage et instance unique

Acquérir le droit exclusif de contrôle avant toute récupération ou modification de plan.
Un objet Windows nommé global protège les contrôleurs de comptes différents. Un accès refusé
interdit le contrôle. Son descripteur de sécurité et son abandon sont validés dans le prototype.
La notification « ouvrir la fenêtre » reste locale à l'utilisateur et à sa session, avec un
contrôle d'identité. Elle ne transporte ni chemin arbitraire ni commande de changement de plan.

Au démarrage, traiter la récupération avant la règle courante. La session inactive n'acquiert pas
le contrôle. La sortie de session libère le contrôle après une tentative de retour et la
persistance de son résultat. Le changement de compte ne transfère aucun journal privé.

## Processus de fenêtre

Décidé le 2026-10-31, voir D18 de `research.md`. La fenêtre s'exécute dans
`power-plan-sync.exe --window`, lancé par le processus de suivi seul. Ce processus ne prend pas
le contrôle, n'ouvre aucun dossier de données et ne lit ni n'écrit aucun plan.

| Sens | Message | Effet |
| --- | --- | --- |
| Suivi → fenêtre | `UiCommand` versionnée | Afficher un modèle, ouvrir, fermer, compléter un dialogue |
| Fenêtre → suivi | `UiAction` versionnée | Commande typée du coordinateur, comme depuis l'icône |
| Fenêtre → suivi | `WindowOpened` | Fenêtre affichée : preuve que le processus répond |
| Fin d'un canal | Aucun | Fenêtre fermée (code nul) ou perdue (autre code) |

Les canaux sont nommés, à une seule instance, refusent les clients distants et n'accordent l'accès
qu'au compte courant. Le processus de suivi les connecte avant le lancement et les transmet au seul
processus de fenêtre. Une trame d'une autre version, d'un type inconnu ou de plus de 1 Mio ferme
le canal. Une fenêtre qui ne répond pas à `Open` dans les 5 s est arrêtée, puis relancée une
seule fois pour cette demande ; au lancement, sans réponse en 15 s, elle est arrêtée sans
relance. Le processus de suivi ne relance jamais de lui-même. La perte du processus de fenêtre
n'interrompt jamais le suivi.

| État de la fenêtre | Événement | État suivant |
| --- | --- | --- |
| Absente | Demande d'ouverture | Lancement, délai de 15 s |
| Lancement | `WindowOpened` | Ouverte |
| Lancement | Délai dépassé | Absente, processus arrêté, perte notifiée |
| Ouverte | Demande d'ouverture | `Open` envoyé, délai de 5 s |
| `Open` en attente | `WindowOpened` | Ouverte |
| `Open` en attente | Délai dépassé | Processus arrêté, un nouveau lancement |
| Ouverte ou en attente | Fin du canal, code nul | Absente ; une demande en attente relance |
| Ouverte ou en attente | Fin du canal, autre code ou trame refusée | Absente, perte notifiée |
| Toute | `Quit` du coordinateur | Fermeture demandée, arrêt après 2 s |

## Modes d'essai prévus

- `--demo` fournit des plans et processus simulés. Ce mode n'appelle aucune écriture de plan réel.
- `--data-dir <chemin>` isole configuration et journaux. Il ne rend pas les API Windows fictives.
- `--start-minimized` ouvre seulement la zone de notification.
- `--help` explique ces options. Elles sont des aides de validation, pas le parcours utilisateur.

Le démarrage à la connexion pointe vers l'exécutable courant et `--start-minimized`.
Une désactivation supprime seulement l'entrée appartenant à PowerPlanSync.
Une erreur de configuration de cette option ne bloque pas l'usage manuel de l'application.

## Erreurs et reprise

Le refus de Windows affiche une explication courte et **Réessayer**. L'outil ne propose pas
un autre plan à la place d'une cible disparue. L'utilisateur peut choisir un remplacement.
Une application protégée inaccessible affiche « Fichier inaccessible · impossible à suivre ».
Aucun repli automatique au nom de fichier ou élévation ne contourne cette limite.

Quitter après un retour échoué propose **Réessayer**, **Ouvrir les options d'alimentation** et
**Quitter quand même**. Ce dernier choix conserve le journal pour la récupération et indique
le plan encore actif. Une fermeture imposée par Windows n'attend pas une réponse indéfiniment.
