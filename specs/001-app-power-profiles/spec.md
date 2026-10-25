# Feature Specification: Associer un plan d'alimentation à un logiciel

**Feature Branch**: conception locale, fonctionnalité `001-app-power-profiles`

**Created**: 2026-09-20

**Status**: Draft, prêt pour les prototypes de faisabilité

**Input**: « Quand je lance Battlefield 6, Windows passe en Haute performance, puis revient en Recommandé. Projet Rust léger, interface accessible, conception entièrement via spec-kit. »

## Clarifications

### Session 2026-10-25

Ces décisions viennent de la revue de design des maquettes GPUI, validée par l'utilisateur. Le détail se trouve dans [contracts/user-interface.md](contracts/user-interface.md) et dans [design/README.md](design/README.md).

- Q : Faut-il un interrupteur global d'automatisation ? → R : Non. L'activation initiale passe par les trois étapes de « Pour commencer ». Ensuite, seule la paire **Mettre en pause** / **Reprendre** reste disponible.
- Q : Comment l'interface nomme-t-elle les règles ? → R : « Logiciels associés » pour la liste, « suivre » pour l'activation d'un logiciel, « automatisation » pour l'état global. Le mot « règle » reste interne.
- Q : Faut-il confirmer une suppression ? → R : Non. La suppression est immédiate et annulable depuis un message (**Annuler**, Ctrl+Z).
- Q : Quelles transitions sont animées ? → R : Quatre seulement, jamais au clavier. Le réglage Windows « Effets d'animation » les désactive.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automatiser une session de jeu (Priority: P1)

L'utilisateur choisit Battlefield 6, le plan Haute performance pendant le jeu et son plan habituel après le jeu. Il ferme la fenêtre de réglages et joue normalement.

**Why this priority**: ce parcours constitue la raison d'exister du produit.

**Independent Test**: configurer un exécutable de démonstration léger, le lancer puis le fermer, et constater les deux changements dans Windows. Le test Battlefield est différé à la demande de l'utilisateur.

**Acceptance Scenarios**:

1. **Given** une règle activée et un plan de retour choisi, **When** l'exécutable configuré démarre, **Then** son plan devient actif et l'interface indique le logiciel responsable.
2. **Given** plusieurs instances du même logiciel, **When** une instance se termine, **Then** le plan reste actif tant qu'une autre instance reconnue fonctionne.
3. **Given** la dernière instance vient de se fermer, **When** trois secondes se sont écoulées sans instance correspondante, **Then** le plan de retour choisi devient actif.
4. **Given** le plan Haute performance était déjà actif avant le jeu, **When** ce cycle se termine, **Then** l'application revient quand même au plan de retour choisi.
5. **Given** seul le lanceur du jeu reste ouvert, **When** le jeu se ferme, **Then** le lanceur ne maintient pas la règle du jeu.
6. **Given** une règle désactivée ou un logiciel homonyme installé ailleurs, **When** ce logiciel démarre, **Then** cette règle ne change aucun plan.

### User Story 2 - Garder la maîtrise de Windows (Priority: P1)

L'utilisateur peut suspendre l'automatisation ou modifier son plan dans Windows. PowerPlanSync ne doit pas reprendre la main en boucle.

**Why this priority**: une restauration imprévisible rendrait le parcours principal inacceptable.

**Independent Test**: avec une règle active, changer le plan dans Windows puis fermer le logiciel. Vérifier que le choix externe reste actif.

**Acceptance Scenarios**:

1. **Given** une règle active, **When** un autre plan est observé, **Then** l'application affiche « Automatisation suspendue · plan choisi dans Windows » et n'effectue plus de changement durant ce cycle.
2. **Given** cette suspension, **When** tous les logiciels correspondants sont fermés depuis trois secondes, **Then** le cycle s'achève sans restauration et l'automatisation attend un nouveau lancement.
3. **Given** cette suspension, **When** l'utilisateur choisit **Reprendre l'automatisation**, **Then** l'application recalcule immédiatement la règle gagnante et démarre un nouveau cycle si nécessaire.
4. **Given** un cycle contrôlé par l'application, **When** l'utilisateur choisit **Mettre en pause** ou **Quitter**, **Then** elle tente un retour protégé au plan choisi. Un plan externe observé est conservé.
5. **Given** un arrêt forcé de PowerPlanSync, **When** il redémarre, **Then** il examine le cycle interrompu avant de reprendre ses règles. Un plan externe distinct reste inchangé.
6. **Given** un refus Windows ou un plan supprimé, **When** une bascule échoue, **Then** l'erreur reste visible et aucune réussite n'est annoncée.

### User Story 3 - Gérer plusieurs logiciels sans règles complexes (Priority: P2)

L'utilisateur ajoute d'autres applications et choisit laquelle a priorité lorsqu'elles fonctionnent ensemble.

**Why this priority**: plusieurs logiciels peuvent demander des plans différents.

**Independent Test**: configurer deux exécutables avec deux plans, inverser leur ordre, puis terminer le premier.

**Acceptance Scenarios**:

1. **Given** plusieurs règles actives, **When** leurs logiciels fonctionnent ensemble, **Then** la première règle de la liste gagne.
2. **Given** la règle gagnante se termine, **When** le délai de trois secondes expire, **Then** la prochaine règle encore applicable prend la main sans passage intermédiaire au plan de retour.
3. **Given** une règle supprimée, désactivée ou réordonnée par l'utilisateur, **When** la modification est enregistrée, **Then** le résultat est recalculé immédiatement et la nouvelle raison apparaît.
4. **Given** un logiciel déplacé, **When** son ancienne règle ne le trouve plus, **Then** l'utilisateur peut sélectionner le nouveau fichier sans perdre le plan associé.

### User Story 4 - Configurer et consulter l'outil sans expertise (Priority: P1)

L'utilisateur réalise le parcours depuis une fenêtre courte et une icône près de l'horloge, avec la souris, le clavier ou le Narrateur.

**Why this priority**: l'accessibilité et la simplicité sont des contraintes du premier produit.

**Independent Test**: ajouter une règle, choisir le retour, suspendre, reprendre et quitter sans souris, puis répéter avec le Narrateur.

**Acceptance Scenarios**:

1. **Given** le premier lancement, **When** l'utilisateur ajoute un logiciel, **Then** il peut choisir un fichier ou une application en cours, puis sélectionner un plan dans une liste lisible.
2. **Given** aucune configuration validée, **When** la fenêtre s'ouvre, **Then** aucun plan ne change et le plan de retour proposé doit être confirmé avant activation.
3. **Given** une configuration active, **When** l'utilisateur ferme la fenêtre, **Then** l'icône reste accessible et le suivi continue sans notification répétitive.
4. **Given** une fenêtre à 200 % ou un thème contrasté, **When** l'utilisateur parcourt les contrôles, **Then** les libellés, le focus et les actions restent lisibles et utilisables.
5. **Given** l'option de démarrage à la connexion désactivée, **When** l'utilisateur l'active puis la désactive, **Then** Windows reflète chacun de ces choix.

### Edge Cases

- Un processus apparaît et disparaît entre deux observations. Il peut ne pas déclencher de cycle.
- Le chemin d'un jeu protégé est inaccessible. L'application indique la limite et ne devine pas son identité.
- Une observation partielle ne prouve pas la fermeture du jeu.
- Le plan cible ou le plan de retour disparaît pendant le jeu.
- Une stratégie Windows empêche une bascule ou un outil constructeur modifie le plan.
- Le PC passe sur batterie, revient sur secteur, se met en veille ou reprend.
- Une autre session essaie de lancer un second contrôleur. Les deux instances ne doivent pas se battre.
- L'application, Windows ou le PC s'arrêtent brutalement.
- Une mise à jour du jeu conserve son chemin mais change son contenu. La règle reste attachée au chemin.
- Un changement externe simultané à une écriture peut ne pas être distingué. La protection est au mieux, sans garantie atomique.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: L'application DOIT afficher les plans installés avec leur nom Windows et le plan réellement actif. Aucun nom traduit ne sert d'identité.
- **FR-002**: L'utilisateur DOIT pouvoir choisir explicitement un plan de retour. « Utilisation normale » est proposé lorsqu'il existe. Aucune équivalence automatique avec le mode « Recommandé » des Paramètres n'est affirmée.
- **FR-003**: L'utilisateur DOIT pouvoir ajouter, modifier, activer, désactiver et supprimer une règle en sélectionnant un exécutable ou un processus accessible. La suppression n'est pas confirmée mais reste annulable tant que son message est affiché.
- **FR-004**: Une règle DOIT identifier le chemin complet du logiciel. Un nom identique ailleurs ne suffit pas. Le lanceur et le jeu restent des logiciels distincts.
- **FR-005**: L'application DOIT compter toutes les instances reconnues de la session contrôlée et ne jamais confondre deux incarnations d'un identifiant de processus réutilisé.
- **FR-006**: La première règle éligible dans l'ordre visible DOIT gagner. Les contrôles **Monter** et **Descendre** rendent cet ordre modifiable au clavier.
- **FR-007**: La perte naturelle de la règle gagnante DOIT attendre trois secondes avant recalcul. Une réapparition annule ce délai. Une action explicite sur les règles recalcule immédiatement.
- **FR-008**: La fin d'un cycle DOIT rétablir le plan de retour validé au début de ce cycle, y compris si le plan cible était déjà actif.
- **FR-009**: Tout changement externe distinct observé pendant un cycle DOIT suspendre les écritures jusqu'à sa fin ou une action **Reprendre l'automatisation**. La fin de cette suspension ne restaure pas le plan de retour.
- **FR-010**: Avant toute écriture ou restauration, l'application DOIT relire le plan actif et abandonner l'écriture si celui-ci diffère de son dernier état attendu. Après écriture, elle DOIT vérifier le résultat.
- **FR-011**: L'application DOIT enregistrer de quoi récupérer un cycle avant d'agir. Au redémarrage, elle conserve un plan externe distinct, réévalue les applications et ne restaure qu'un état compatible avec le cycle enregistré.
- **FR-012**: **Mettre en pause** DOIT rester active après redémarrage. **Reprendre** recalcule immédiatement les règles. **Quitter** tente la restauration protégée avant l'arrêt. Fermer la fenêtre laisse le suivi actif.
- **FR-013**: Un refus Windows ou une cible disparue DOIT produire une erreur persistante avec une action **Réessayer**. Aucun autre plan ne sera choisi silencieusement. Aucune tentative ne se répète à chaque observation.
- **FR-014**: Par défaut, les règles DOIVENT s'appliquer uniquement sur secteur. Sur batterie ou source inconnue, elles deviennent inéligibles et le cycle contrôlé se termine par un retour protégé. Une option globale permet l'usage sur batterie.
- **FR-015**: Au retour de veille, l'application DOIT relire le plan, la source d'alimentation et les processus avant d'écrire. En quittant la session interactive active, elle cesse le contrôle et tente un retour protégé.
- **FR-016**: Une seule instance DOIT contrôler la machine. Une seconde instance de la même session ouvre les réglages existants. Une autre session indique que le contrôle est indisponible. Aucun transfert automatique entre comptes n'est prévu.
- **FR-017**: Les règles et réglages DOIVENT survivre à un redémarrage. Un fichier endommagé ou incompatible est conservé, l'automatisation reste arrêtée et une erreur explique la récupération.
- **FR-018**: L'interface DOIT montrer le plan observé, le logiciel responsable, l'état du suivi et les actions utiles sans exposer d'identifiant technique dans le parcours principal.
- **FR-019**: Le parcours complet DOIT fonctionner au clavier et au Narrateur, avec noms et états accessibles, focus visible, thèmes clair, sombre et contrasté, et échelle de 100 à 200 %. Une action au clavier n'est jamais animée et le réglage Windows « Effets d'animation » supprime toute animation.
- **FR-020**: Le démarrage à la connexion DOIT être facultatif et désactivé à la première installation. L'application DOIT fonctionner sans compte, réseau ou télémétrie.
- **FR-021**: Les diagnostics DOIVENT être locaux et bornés à 5 Mio. Ils ne collectent ni arguments de lancement, ni liste exhaustive des applications, ni mémoire de jeu.
- **FR-022**: Le premier produit DOIT utiliser exclusivement les plans existants. Il ne change ni priorité de processus, ni affinité, ni réglages internes des plans, et ne promet aucun gain de FPS.
- **FR-023**: Un nouveau processus inaccessible ne DOIT jamais être reconnu par approximation. Une instance précédemment reconnue mais devenue indéterminée maintient l'état pendant au plus dix secondes, puis provoque une suspension visible et un retour protégé.
- **FR-024**: Modifier le plan de retour pendant un cycle DOIT affecter le cycle suivant. L'interface indique le retour déjà réservé pour le cycle courant.
- **FR-025**: L'activation initiale DOIT attendre la validation du plan de retour et d'au moins une règle. Être au repos sans cycle ne force jamais ce plan. Après cette activation, l'interface n'offre pas d'interrupteur global permanent : seule **Mettre en pause** arrête temporairement l'automatisation.
- **FR-026**: Les tests automatiques DOIVENT couvrir les décisions de bascule, les erreurs et la récupération. Les essais Windows réels DOIVENT vérifier séparément l'accessibilité, la consommation et le cycle d'un exécutable témoin léger. Battlefield 6 n'est pas lancé pendant ce développement.

### Key Entities *(include if feature involves data)*

- **Plan**. Identité Windows, nom affiché et disponibilité actuelle.
- **Règle**. Logiciel choisi, plan associé, activation et position dans la liste.
- **Instance de logiciel**. Identité d'une exécution, session, chemin vérifié et degré de certitude.
- **Configuration**. Règles, plan de retour, pause, option batterie et démarrage à la connexion.
- **Cycle**. Règles participantes, plan initial, retour réservé, plan appliqué et autorisation de continuer.
- **Journal de récupération**. Dernière intention durable, résultat confirmé et propriétaire du cycle.
- **État présenté**. Plan observé, raison, suspension ou erreur, et actions possibles.

## Success Criteria *(mandatory)*

### Measurable Outcomes

Ces critères sont des objectifs de livraison, non des mesures déjà obtenues.

- **SC-001**: Au moins quatre personnes sur cinq non spécialistes configurent une première règle et son retour en moins de deux minutes, sans aide verbale.
- **SC-002**: Sur 100 cycles d'un logiciel restant actif au moins dix secondes, 95 % des activations prennent au plus deux secondes depuis la création réelle du processus. 95 % des retours prennent au plus cinq secondes depuis la fermeture réelle de la dernière instance. Ces durées incluent détection, grâce éventuelle, écriture et confirmation du plan dans Windows.
- **SC-003**: Les scénarios de changement externe observé, instances multiples, plan supprimé et récupération ne produisent aucune écriture contraire aux règles de protection.
- **SC-004**: Sur la machine de référence, avec 20 règles et environ 200 processus, la moyenne CPU sur 30 minutes est au plus 0,1 % de la capacité totale de la machine et la mémoire privée au plus 40 Mio, fenêtre fermée après stabilisation. Fenêtre ouverte au repos, la mémoire privée est au plus 100 Mio.
- **SC-005**: Toutes les actions principales passent les parcours clavier et Narrateur à 100 %, 150 % et 200 %, en thèmes clair, sombre et contrasté.
- **SC-006**: Un exécutable témoin léger réussit l'activation, le maintien et le retour avec un lanceur témoin encore ouvert. Les essais couvrent plusieurs instances, un homonyme dans un autre dossier et un arrêt forcé. Ce résultat ne prétend pas valider l'anti-triche ou la compatibilité Battlefield 6.
- **SC-007**: Cent ouvertures puis fermetures des réglages ne laissent aucune fenêtre ou boucle de rendu permanente. Après stabilisation, mémoire privée et nombre de handles restent dans les budgets du protocole de validation.

## Assumptions

- Le premier produit vise Windows 11 x64. Windows 10, ARM64 et les modes d'alimentation modernes seront évalués dans d'autres spécifications.
- Demande de l'utilisateur du 2026-09-20 lors du démarrage de l'implémentation. Utiliser un exécutable personnalisé léger au lieu de lancer Battlefield 6. La validation propre au jeu reste différée sans bloquer la validation générique.
- Sur le PC inspecté, le plan normal porte le nom « Utilisation normale ». C'est la proposition initiale de retour pour le besoin « Recommandé ».
- Les chemins Windows sont sélectionnés depuis le système, sans demander à l'utilisateur de saisir celui de Battlefield. Le nom réel du processus du jeu reste à constater.
- Le produit suit la présence d'un processus, pas sa consommation ni le focus de sa fenêtre.
- Une seule session interactive contrôle l'outil. La collaboration entre comptes et le transfert de récupération d'un compte à un autre sont hors périmètre.
- Aucun composant indépendant ne surveille les crashs. Une fermeture forcée peut laisser le dernier plan actif jusqu'au redémarrage ou à une correction manuelle.
- Un jeu plus rapide est une hypothèse à mesurer. Le bénéfice garanti par la spécification est l'automatisation du choix de plan.
