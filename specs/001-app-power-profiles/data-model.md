# Modèle de données et transitions

Ce document décrit les types à implémenter. Aucun type Rust n'est encore livré.

## Identités et configuration

| Type prévu | Données | Invariants |
| --- | --- | --- |
| PlanId | GUID Windows | Comparaison par valeur, aucun nom traduit dans la clé |
| PowerPlan | PlanId, nom UTF-16 converti pour affichage, disponibilité | Catalogue rafraîchissable |
| RuleId | Identifiant local stable | Unique dans la configuration |
| ExecutablePath | Chemin absolu Windows normalisé, chemin affichable | Sélection de fichier existant, comparaison Windows, pas de correspondance au nom seul |
| AppRule | RuleId, libellé, ExecutablePath, PlanId, enabled | Un chemin normalisé ne possède qu'une règle |
| ConfigurationV1 | schema_version, rules ordonnées, return_plan, automation_enabled, paused, allow_on_battery, start_at_login | Version reconnue, retour obligatoire avant activation |
| ProcessInstance | PID, création Windows, session, chemin vérifié | Un PID réutilisé crée une nouvelle instance |
| Observation | Présent vérifié, absent confirmé, indéterminé avec raison | Une erreur de lecture n'est pas une absence |
| PowerSource | Secteur, batterie, inconnue | Batterie et inconnue inéligibles par défaut |
| Cycle | Identifiant, plan initial, retour réservé, dernière application confirmée, règle gagnante | Le retour reste figé pendant ce cycle |
| RecoveryRecordV1 | version, utilisateur et session propriétaire, cycle, transaction, horodatage | Écrit durablement avant tout changement |
| ViewState | plan observé, raison, état du moteur, retour du cycle, erreur, actions | L'affichage ne devine pas qu'une écriture a réussi |

Le tableau `rules` définit l'ordre, sans champ numérique de priorité concurrent.
`automation_enabled` vaut faux jusqu'à l'activation explicite initiale. Une configuration complète
reste inerte avant cette activation. `paused` représente ensuite une suspension persistante.
Le libellé produit peut être « Battlefield 6 ». Son chemin est choisi sur le PC.
Un changement du contenu du fichier au même chemin ne crée pas de nouvelle règle.
Une modification du chemin exige une nouvelle validation depuis Windows.

## États du moteur

| État | Sens | Écritures permises |
| --- | --- | --- |
| Unconfigured | Retour ou règle absents | Aucune |
| Idle | Configuré, aucun cycle | Acquisition seulement si une règle est éligible |
| Active | Cycle confirmé et règle gagnante reconnue | Changement de gagnant après validation |
| ExitGrace | Le gagnant n'est plus présent, délai de 3 s en cours | Aucune avant expiration, sauf commande explicite |
| Applying | Une intention durable attend son résultat Windows | Une seule transaction, aucune seconde intention |
| ManualOverride | Plan externe distinct observé | Aucune jusqu'à Reprendre ou fin du cycle |
| Paused | Pause explicite persistante | Retour protégé de clôture seulement à l'entrée |
| Blocked | Erreur de droits, stockage, plan manquant ou observation prolongée | Réessayer explicite, aucun retry par seconde |
| Recovering | Journal d'un cycle interrompu | Récupération protégée avant tout nouveau cycle |
| SessionInactive | Session inactive ou verrou indisponible | Retour protégé avant libération du contrôle, puis aucune |

Les raisons de blocage sont des variantes typées. Une accumulation de booléens ne définit pas l'état.
`Applying` conserve l'état de départ et la suite attendue après confirmation ou échec.

## Transitions qui décident du plan

1. Une règle éligible apparaît dans `Idle`. Lire le plan actif, réserver le retour configuré,
   préparer le journal puis appliquer le plan gagnant. Si ce plan est déjà actif, confirmer le
   cycle sans appel d'écriture. Le cycle conserve tout de même son retour explicite.
2. Le gagnant disparaît lors d'un snapshot complet. Entrer dans `ExitGrace` avec une échéance
   monotone à trois secondes. Une réapparition annule l'échéance.
3. À l'échéance, recalculer tous les candidats. Appliquer le prochain gagnant ou clôturer par
   le retour réservé. Ne pas passer par le retour si une autre règle éligible existe.
4. Ajouter, supprimer, activer ou réordonner une règle recalcule immédiatement le gagnant.
   Modifier le retour ne change pas le retour réservé au cycle courant.
5. Observer un plan différent de la dernière valeur attendue conduit à `ManualOverride`.
   Persister cette révocation. Aucun retour n'est appliqué lorsque ce cycle s'achève.
6. **Reprendre** révoque la pause ou la suspension, clôture le journal révoqué sans écriture,
   relit Windows et démarre un nouveau cycle si une règle est éligible.
7. **Pause**, batterie non autorisée, session quittée ou **Quitter** clôturent le cycle par un
   retour protégé. Le plan externe distinct observé reste prioritaire.
8. Une instance connue devient indéterminée. La conserver provisoirement durant dix secondes.
   Un snapshot complet sans son PID confirme son absence. Si l'incertitude persiste, tenter
   un retour protégé puis entrer dans `Blocked`. L'échec du retour reste dans le journal.
9. Sans règle activée, en pause ou sans contrôle de session, arrêter le timer de processus.
   Les commandes utilisateur et les notifications pertinentes peuvent réveiller le coordinateur.

Le verrouillage de l'écran ne signifie pas une perte de session interactive. Le suivi continue
dans cette session. La déconnexion, le changement de compte actif et la fermeture de session
déclenchent la clôture protégée. Les notifications exactes sont vérifiées par le prototype Windows.

## Transaction récupérable

Une transaction possède un numéro et les champs `before`, `intended`, `return_plan` et `phase`.
Les phases sont `Prepared`, `Applied`, `Revoked` et `RestorePending`.
Le journal conserve le dernier GUID confirmé même lorsqu'un nouveau gagnant est préparé.

| Point de coupure | Décision au redémarrage |
| --- | --- |
| Avant journal durable | Aucune écriture Windows n'était permise |
| Prepared, plan encore before | L'intention n'est pas confirmée. Recalculer avant de la reprendre |
| Prepared, plan égal intended | L'écriture a pu réussir. Reprendre le cycle ou restaurer selon les règles présentes |
| Prepared, plan différent des deux | Considérer une révocation externe, aucune écriture |
| Applied, plan égal au dernier appliqué | Reprendre si une règle est éligible, sinon retour protégé |
| Applied, plan différent | Révocation externe, conserver le plan observé |
| Revoked | Ne jamais restaurer, même si le GUID est redevenu identique |
| RestorePending, plan égal au retour | Finaliser le journal sans répéter l'écriture |
| RestorePending, plan égal au dernier appliqué | Retenter une fois pendant la récupération, puis bloquer en cas d'échec |
| RestorePending, autre plan | Révocation externe, aucune écriture |

Si `before` et `intended` sont égaux, aucune bascule n'était nécessaire mais le cycle avait été
autorisé. La récupération peut clôturer vers le retour réservé si aucun logiciel n'est éligible.
Une lecture échouée ne permet aucune écriture. Un journal illisible est conservé et bloque le suivi.
Un journal d'un autre compte n'est pas consommé.

Le cycle porte aussi `acquired`, qui distingue une acquisition initiale non confirmée d'un
changement de gagnant dans un cycle déjà acquis. Un journal `Prepared` initial, avec `before`
différent de `intended` et Windows encore sur `before`, est clôturé sans retour si la règle
n'est plus éligible. Un changement de gagnant interrompu conserve le retour du cycle acquis.
Le cas `before == intended` autorise toujours le retour, même sans appel d'écriture.
Une modification du seul retour configuré ne raccourcit pas une grâce de fermeture en cours.

La protection n'est pas un verrou Windows sur le plan. Une modification externe non observée
ou de même GUID ne peut pas être attribuée avec certitude. Cette limite reste visible dans la documentation.

## Persistance et confidentialité

Les fichiers portent une version de schéma. Une version future est refusée sans réécriture.
Écrire un fichier temporaire dans le même répertoire, vider ses buffers puis le remplacer
avec les primitives Windows appropriées. Tester les coupures réelles et les erreurs injectées.
Une sauvegarde de configuration valide est conservée. Les chemins des règles restent locaux.
Le journal d'activité décrit règles et erreurs sans lignes de commande ni inventaire de processus.
Sa rotation limite l'ensemble à 5 Mio.
