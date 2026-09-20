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

## Réconciliation pendant le développement

La première unité implémente seulement les identités et les adaptateurs nécessaires aux sondes.
Le moteur définitif vient après les preuves. Toute déviation acceptée est consignée ici avant
le passage à l'unité suivante. Les candidats ne font pas autorité en dehors de ce contrat spec-kit.
