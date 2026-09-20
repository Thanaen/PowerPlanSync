# Premier lot de faisabilité

Date : 20 septembre 2026. Code intégré sur `main` dans `114ae26` puis `9fc0ff4`.
Cette livraison contient des sondes et des adaptateurs. Elle ne contient pas encore le moteur
de règles, la configuration persistante ni le journal de récupération du produit.

## Vérifications exécutées

- Build release des trois exemples, formatage, Clippy sans avertissement et quatre tests Rust réussis.
- Énumération des plans, détection du témoin, instances multiples, homonyme et lanceur restant ouvert.
- Trois cycles réels Haute performance puis Utilisation normale. Réglage initial restauré.
- Refus d'un second contrôleur dans un autre processus du même compte.
- Inspection native de l'interface, saisie, Tabulation, case activée avec Espace et sélection de plan avec les flèches puis Entrée.
- Captures à facteurs Slint 1, 1,5 et 2, sans changer les paramètres d'affichage Windows.
- Masquage de la fenêtre avec maintien du processus, puis mesure au repos dans les deux états.

Les quatre tests Rust couvrent les conversions d'identité et les comparaisons de noms/chemins.
Ils ne remplacent pas les futurs tests du moteur et de récupération. La racine a réexécuté les
contrôles dans le worktree de construction après son transfert, puis vérifié l'identité des
sources avec le checkout principal. Les sorties sont dans [validation/](validation/).

Les premiers écrans débordaient horizontalement. Le titre, la largeur du contenu et les boutons
ont été corrigés, puis vérifiés dans Windows. La hauteur initiale est passée à 520 unités.
Voir [avant correction](accessibility/prototype-v1-before.png),
[écran corrigé](accessibility/prototype-v3-normal.png) et [arbre accessible](accessibility/prototype-v3-tree.txt).

## Première mesure de consommation

Machine Ryzen 7 9800X3D, 16 processeurs logiques, Windows 11 25H2 build 26200.9457.
Mesure de 60 secondes par état, échantillon chaque seconde, CPU normalisé sur toute la machine.
L'interface est seule ; aucun moteur de surveillance n'existe encore dans ce processus.

| État | CPU moyen total machine | Mémoire privée maximale | Working set maximal |
| --- | --- | --- | --- |
| Fenêtre ouverte | 0,0469 % | 5,406 Mio | 26,500 Mio |
| Fenêtre détruite, icône active | 0,0000 % mesuré | 3,691 Mio | 25,230 Mio |

Les données et rapports sont dans [performance/prototype-window/](performance/prototype-window/)
et [performance/prototype-tray/](performance/prototype-tray/).
Zéro mesuré est un résultat arrondi sur cette minute, pas une garantie d'absence de travail.
Les campagnes de trente minutes, vingt règles et cent cycles de fenêtre restent à faire sur le produit.

## Critères encore ouverts

- Narrateur, annonces de statut et focus accessible. Le focus visuel se déplace, mais le pilote
  d'inspection continue à signaler le champ de saisie. Le défaut peut provenir de l'extracteur
  ou de l'intégration accessible. Aucun succès Narrateur n'est déclaré.
- Thème clair, thème à contraste élevé, texte agrandi et changement d'écran/DPI Windows réel.
- Réouverture par le menu de l'icône. Le pilote n'expose pas la barre des tâches dans cette session.
  Le masquage a été exercé ; la réouverture par l'utilisateur reste à vérifier.
- Mesures prolongées, ETW, sessions multiples et reprise de veille.

Les captures [100 %](accessibility/prototype-scale-100.png),
[150 %](accessibility/prototype-scale-150.png) et [200 %](accessibility/prototype-scale-200.png)
prouvent le rendu à ces facteurs propres au processus, pas le passage entre écrans Windows.
Tous les processus créés pour les essais ont été arrêtés.

## Décision

Les appels Windows et la détection du témoin sont faisables sur cette machine sans élévation.
Slint reste le candidat retenu. Son accessibilité complète n'est pas encore validée.
T001, T002, T003 et T005 sont terminées. T004 et T006 restent ouvertes ; le passage au moteur
complet n'est pas déclaré acquis. Les réserves ne sont pas effacées pour annoncer une version distribuable.

Binaires inspectés et exécutés, SHA-256 :

```text
ui_probe.exe       98FA45B0DD0AC07457C6D0690547A476C4499041F2A599D9C3B0B4FD70181EDC
windows_probe.exe  07A88A5107CAA9F9571FA20AEE65948FEAC6179252A36BB3A38764B6CA82BA08
demo_target.exe    DD74F02F001D2FDACD727174EED5D8F7703037B4CC8F30CD0905C0E6C860438C
```
