# Contrat Rust retenu pour l'implémentation

## Problème

Un seul propriétaire doit ordonner les décisions, le journal et les écritures Windows.
Les prototypes doivent réutiliser les adaptateurs du produit sans créer un moteur complet prématuré.

## Usage et responsabilités

La sonde Windows expose inspection, détection du témoin et bascule contrôlée avec retour.
Elle est en lecture seule par défaut. `--allow-switch` autorise les seules bascules d'essai.
La sonde UI ne possède aucun accès à l'écriture des plans. `demo_target` dort pendant une durée
choisie et peut servir d'enfant d'un lanceur distinct. Il ne génère pas de charge CPU artificielle.

Le produit expose ensuite `Controller::open`, `handle` et `view` à la couche UI.
L'UI envoie des commandes typées. Elle n'appelle ni le stockage ni Windows directement.
Un thread synchrone unique traite chaque commande jusqu'à son résultat avant la suivante.
La boucle graphique reste dans son propre thread principal.

## Types et frontières

`PlanId` encapsule un GUID. `ExecutablePath` conserve le chemin Windows sans conversion UTF-8
destructive. `ProcessKey` réunit PID, date de création et session. Un snapshot distingue présence
vérifiée, absence confirmée et observation indéterminée. Les règles restent un vecteur ordonné.

Le cycle conserve son retour réservé, le plan initial, le dernier plan confirmé et le gagnant.
Une table par `ProcessKey` porte les échéances d'incertitude. Une échéance monotone de grâce
appartient à l'état `ExitGrace`. Ces informations ne sont pas reconstruites depuis le journal texte.

Le moteur pur calcule une transition et une intention depuis l'état, la configuration, le snapshot
et l'action. Le coordinateur transforme l'intention en un résultat typé confirmé, révoqué ou échoué.
Les adaptateurs système et de stockage ont des doubles d'essai qui injectent les coupures.
Les détails de sérialisation et les types Win32 restent à leurs frontières respectives.

La transaction suit cette séquence dans un seul appel du coordinateur.

1. Lire le plan et vérifier la valeur attendue.
2. Persister durablement l'intention avec son numéro de transaction.
3. Relire Windows après cette persistance, juste avant l'écriture.
4. Abandonner si un autre plan est observé.
5. Appliquer seulement si nécessaire, puis relire le résultat.
6. Persister la confirmation ou la révocation. Un échec conserve l'intention récupérable.

Le stockage vérifie l'identité du cycle et de la transaction quand il les finalise.
Une acquisition où le plan cible est déjà actif conserve tout de même un cycle et son retour.
Les états et la récupération suivent `data-model.md`. Aucun protocole public d'effets asynchrones,
bus d'événements générique ou acteur par règle n'est ajouté.

## Synthèse des variantes

Trois variantes ont été comparées avant le code. La variante A, coordinateur synchrone et décision
pure, est retenue pour sa petite interface et sa relecture après persistance. La variante B rend
chaque effet et acquittement explicites. Son contrôle du numéro de transaction et ses scénarios
de coupure sont repris, sans ajouter ce protocole à l'interface publique. La variante C apporte
la séparation du verrou de session et l'option explicite de bascule pour la sonde.

Le juge indépendant gpt-5.5 préférait initialement B pour l'ordre des effets encodé dans les types.
La décision de la racine conserve A car aucun effet n'est concurrent et les coupures sont testables
au coordinateur. La seconde relecture Windows traite en outre une fenêtre de course réelle.
Les scores de la racine sur exactitude, profondeur d'interface, tests et réutilisation sont
A 5/5/4/5, B 3/4/5/4 et C 3/4/4/4. Les candidats sont conservés dans la trace locale `.audit/architect/`.

Après lecture de cette synthèse, le juge gpt-5.5 confirme le choix A sans blocage pour les sondes.
Les tests du futur coordinateur devront couvrir le changement de plan entre préparation et écriture,
la lecture échouée après écriture et le refus d'une finalisation portant un ancien identifiant.

## Réconciliation pendant le développement

La première unité implémente seulement les identités et les adaptateurs nécessaires aux sondes.
Le moteur définitif vient après les preuves. Toute déviation acceptée est consignée ici avant
le passage à l'unité suivante. Les candidats ne font pas autorité en dehors de ce contrat spec-kit.

Le prototype conserve le handle du processus pendant la lecture du chemin, de la création et
de la session. La crainte initiale d'une réutilisation du PID entre ces lectures a été écartée
après vérification de la [durée de vie des identifiants Windows](https://devblogs.microsoft.com/oldnewthing/20150918-00/?p=91561).
Un handle ouvert conserve l'identité du processus. Le filtrage de session et le suivi des
observations incertaines sur plusieurs snapshots restent des travaux du produit.

## Couche d'interface GPUI

Décidée le 2026-10-25, voir D12 à D16 de `research.md`. Ce contrat ne change pas pour le moteur,
le coordinateur ni les adaptateurs.

- `src/interface/` est le seul module qui dépend de GPUI et de `tray-icon`. Le domaine, le moteur,
  le coordinateur et `src/presentation.rs` n'en dépendent pas.
- `src/presentation.rs` transforme `ViewState` en un modèle d'affichage sans type graphique,
  testé sans fenêtre. Les vues GPUI lisent ce modèle et envoient des commandes typées.
- Le thread principal exécute GPUI. Le coordinateur garde son thread. Ses états publiés arrivent
  par canal, lus dans une tâche GPUI. Les événements de l'icône et de son menu suivent le même
  chemin, comme dans Bientôt.
- La fenêtre est créée à la demande et détruite à sa fermeture. L'icône, le coordinateur et la
  fenêtre cachée des notifications Windows vivent sans elle.
- Depuis la décision du 2026-10-31 (D17, D18), GPUI ne s'exécute que dans le processus de fenêtre,
  `power-plan-sync.exe --window` (`src/window_process.rs`). Le processus de suivi (`src/tracker.rs`)
  pompe les messages Windows sur son thread principal pour l'icône de `src/interface/tray.rs`, qui
  n'utilise pas GPUI. Il lance le processus de fenêtre, lui transmet les `UiCommand` et rend ses
  `UiAction` au coordinateur, comme le canal de T056 le faisait dans un seul processus.
- `src/window_protocol.rs` sérialise ces messages en trames versionnées et refuse tout message
  inconnu. `src/windows/window_channel.rs` crée les canaux nommés. Le processus de fenêtre ne
  reçoit aucun accès au coordinateur, au stockage ni à Windows au-delà de sa fenêtre.
- `reduce_motion` est une donnée de GPUI, mise à jour depuis `src/windows/preferences.rs` à chaque
  `WM_SETTINGCHANGE`. Aucune vue ne lit le registre ni n'appelle Win32 directement.
- Jusqu'à T071, l'interface GPUI s'ouvre seulement avec `--gpui`. Les deux interfaces utilisent le
  même `Controller` et ne s'exécutent jamais ensemble. À partir de T077, `--gpui` lance le
  processus de suivi sans GPUI ; l'interface GPUI dans le même processus disparaît.
