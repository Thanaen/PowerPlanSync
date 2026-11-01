# Audit des droits, du réseau et des données (T043)

Audit du 1er novembre 2026, sur le commit `caf54ff` de `main`. Il couvre FR-020, FR-021,
FR-022 et le principe V de la constitution. Il porte sur `src/windows/`, `src/interface/`,
`src/diagnostics.rs`, `src/storage.rs`, `src/runtime.rs`, `src/tracker.rs`, `src/window_*.rs`,
les dépendances résolues pour Windows et le binaire release.

Environnement : Windows 11 Pro 10.0.26300.9711, compte non élevé, `rustc 1.98.1`,
`cargo 1.98.1`, VS Build Tools 2026 (`dumpbin` 14.51.36231). Le binaire examiné est
`target/release/power-plan-sync.exe`, construit par `cargo build --locked --release`
(29 135 872 octets, SHA-256 `D1717C6BF70F372A5ECC8EBCD5A6DBA61D08D5B6782AE01C6BEDDA57A04C7D78`).

## Conclusion

Aucune violation de FR-020, FR-021 ou FR-022 n'a été trouvée. Le binaire n'importe aucune
bibliothèque réseau Windows. Les dépendances ne contiennent aucun client HTTP, aucune pile TLS,
aucun envoi de rapports d'erreur et aucune vérification de mise à jour. Un essai de démonstration
de cinq minutes n'a ouvert aucun point de terminaison TCP ou UDP, ni dans le processus de suivi
ni dans le processus `--window`. Les diagnostics restent locaux et bornés à 5 Mio.
Ils ne contiennent ni chemin complet, ni argument de lancement, ni liste des applications.

Trois points sans violation sont notés à la fin : le code HTTP de GPUI compilé mais inactif,
les chemins de construction présents dans le binaire et la DACL par défaut du verrou global.

## 1. Droits demandés

### Niveau d'exécution

Le projet n'a pas de manifeste propre : `build.rs` compile seulement l'interface Slint.
Le manifeste vient de la fonctionnalité `windows-manifest` de `gpui-pre`
(`resources/windows/gpui.manifest.xml`). Le binaire le contient bien :
`<requestedExecutionLevel level="asInvoker" uiAccess="false" />`. Aucune élévation n'est demandée.
Aucun lancement n'utilise le verbe `runas`.

### Privilèges et accès aux autres processus

Les imports du binaire ne contiennent ni `AdjustTokenPrivileges`, ni `LookupPrivilegeValueW`,
ni `ReadProcessMemory`, `WriteProcessMemory`, `VirtualAllocEx`, `CreateRemoteThread`,
`SetWindowsHookExW`, `DebugActiveProcess`, `SetPriorityClass` ou `SetProcessAffinityMask`.
Aucune chaîne `Se…Privilege` n'apparaît dans le binaire. Il n'y a donc pas de privilège de débogage.

- `src/windows/session.rs` appelle `OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY)` pour lire
  le SID du compte courant, et rien d'autre.
- `src/windows/processes.rs` ouvre les processus avec `PROCESS_QUERY_LIMITED_INFORMATION`, puis lit
  seulement `QueryFullProcessImageNameW`, `GetProcessTimes` et la session. Il ne lit pas la mémoire
  des processus et ne fait aucune injection.
- `SetThreadPriority`, `PowerCreateRequest`, `PowerSetRequest` et `SetWindowDisplayAffinity`
  figurent dans les imports. Ils viennent de `gpui-pre-windows` (`spawn_realtime`,
  `prevent_idle_sleep`, protection du contenu) et de `winit`. Ils agissent sur les fils ou fenêtres
  de l'application, jamais sur un autre processus. Le code de PowerPlanSync n'appelle pas les API
  GPUI correspondantes.

### Plans d'alimentation (FR-022)

`powrprof.dll` fournit exactement quatre fonctions : `PowerEnumerate`, `PowerGetActiveScheme`,
`PowerReadFriendlyName` et `PowerSetActiveScheme`. Aucune fonction `PowerWrite…`,
`PowerCreatePossibleSetting`, `PowerDuplicateScheme` ou `PowerDeleteScheme` n'est importée.
Le produit choisit donc un plan existant sans modifier son contenu, ni la priorité ou
l'affinité d'un processus.

### Registre

| Clé | Accès | Quand |
| --- | --- | --- |
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, valeur `PowerPlanSync` | lecture, écriture, suppression | Écriture ou suppression seulement sur action de la personne (`SetStartAtLogin`). Refusée en démonstration et avec un `--data-dir` personnalisé. Une valeur qui appartient à une autre commande est conservée. Désactivée par défaut (`start_at_login: false`). |
| `HKCU\Software\Microsoft\Accessibility`, `TextScaleFactor` | lecture | Taille du texte Windows (`src/windows/preferences.rs`). |
| Disposition du clavier sous `HKLM` | lecture | `gpui-pre-windows` (`keyboard.rs`). |

`RegCreateKeyExW` et `RegSetValueExW` sont importés par `windows-registry` pour GPUI. GPUI ne
s'en sert que pour `HKCU\Software\Classes\AppUserModelId\…`, dans `show_system_notification`.
Cette fonction exige `App::set_app_identity`. PowerPlanSync n'appelle aucune des deux.
`CredReadW` et `CredWriteW` sont importés pour les fonctions `read_credentials` et
`write_credentials` de GPUI, que PowerPlanSync n'appelle pas.

L'essai de démonstration (section 2.3) a relevé la clé `Run` avant et après : elle est inchangée.

### Objets noyau (D07, D18)

| Objet | Sécurité |
| --- | --- |
| Mutex `Global\PowerPlanSync.Controller` | DACL par défaut du jeton, créé sans élévation. |
| Événement `Local\PowerPlanSync.Open.<session>.<SID>` | DACL protégée `D:P(A;;GA;;;<SID du compte>)`. |
| Canaux `\\.\pipe\PowerPlanSync.Window.<session>.<pid>.<n>.<sens>` | DACL protégée `D:P(A;;GA;;;<SID du compte>)`, `FILE_FLAG_FIRST_PIPE_INSTANCE`, une seule instance, `PIPE_REJECT_REMOTE_CLIENTS`. |

Les tests Windows de ces objets passent sur cette machine
(`cargo test --locked --lib -- window_channel startup session`, 17 tests), dont
`only_the_current_account_is_granted_access`, `a_connected_channel_refuses_any_other_client` et
`an_intruder_connected_first_aborts_the_channel`.

### Processus lancés

- Le processus de suivi lance son propre exécutable avec le seul argument `--window` (D18).
- Sur action de la personne, il lance `explorer.exe` avec un argument fixe :
  `ms-settings:powersleep`, le dossier de données, l'adresse du guide ou les notices de licence.

### Fichiers écrits

Seulement dans le dossier de données : `%LOCALAPPDATA%\PowerPlanSync`, ou le dossier donné par
`--data-dir`. Une démonstration écrit `demo.lock` dans ce dossier et le reste dans son sous-dossier
`simulation`. Aucun appel à `temp_dir` n'existe hors des tests. Le détail est en section 3.

## 2. Réseau absent

### 2.1 Dépendances

`cargo tree --locked -e normal --target x86_64-pc-windows-msvc` résout 361 crates, avec leurs
fonctionnalités dans [cargo-tree-windows.txt](privacy/cargo-tree-windows.txt). Aucune n'est
un client ou serveur réseau : ni `reqwest`, `ureq`, `hyper`, `isahc`, `curl`, `h2`, `quinn`,
`tokio`, `mio`, `socket2`, `rustls`, `native-tls`, `openssl`, ni `sentry`, `opentelemetry`,
crash reporter ou outil de mise à jour. Aucun journal n'est installé (`env_logger`,
`tracing-subscriber` ou autre) : les messages `log` et `tracing` des dépendances sont ignorés.
Les fonctionnalités `windows` et `windows-sys` activées ne contiennent aucun module
`Win32_Networking*`, `Win32_NetworkManagement*` ou `Web*`.

Une recherche dans les sources de ces 361 crates (`TcpStream`, `TcpListener`, `UdpSocket`,
`std::net::`, `WinHttp`, `WinINet`, `InternetOpen`, `ws2_32`, `WSAStartup`,
`URLDownloadToFile`, `sentry`, `telemetry`) donne seulement :

- les liaisons générées de `windows` et `windows-sys`, dont aucune n'est activée pour le réseau ;
- des exemples de documentation et des tests (`serde_json`, `slint`, `weezl`, `arrayvec`, `quote`) ;
- des types d'adresses IP sans socket (`log`, `schemars`, `url`) ;
- deux commentaires de `gpui-pre` qui emploient le mot « telemetry ».

`url::Url::socket_addrs` ferait une résolution DNS, mais elle n'est pas liée : le binaire
n'importe pas `ws2_32.dll` (section 2.2).

**Code HTTP compilé mais inactif.** `gpui-pre` dépend de `gpui-pre-http-client`. Cette crate
définit le trait `HttpClient`, des types (`http`, `url`) et des aides de téléchargement GitHub.
Elle ne contient aucune implémentation réseau. `Application::with_platform` installe
`NullHttpClient`, dont `send` répond toujours « No HttpClient available ». PowerPlanSync n'appelle
ni `with_http_client` ni `set_http_client`. Le seul appel interne de GPUI vient des images
`Resource::Uri`. L'application n'en affiche pas : ses icônes SVG sont intégrées par
`include_bytes!` (`src/interface/assets.rs`).

**Polices.** Les polices IBM Plex sont intégrées au binaire et chargées par
`text_system().add_fonts` (`src/interface/theme.rs`). Les polices système sont lues localement par
DirectWrite (`dwrite.dll`). Dans l'interface Slint, `fontdb` lit les dossiers de polices locaux.
Aucune police n'est téléchargée.

`tray-icon`, `muda` et `image` (décodeurs ICO, PNG et ceux activés par GPUI) n'ont aucune
dépendance réseau. Les icônes de la zone de notification sont construites en mémoire.

### 2.2 Binaire

`dumpbin /dependents` liste 39 entrées ([dumpbin-dependents.txt](privacy/dumpbin-dependents.txt)) :
`kernel32`, `advapi32`, `user32`, `shell32`, `gdi32`, `api-ms-win-shcore-scaling-l1-1-1`,
`comctl32`, `icuuc`, `shlwapi`, `uiautomationcore`, `uxtheme`, `ole32`,
`api-ms-win-core-synch-l1-2-0`, `powrprof`, `wtsapi32`, `comdlg32`, `dcomp`, `dwmapi`,
`combase`, `dwrite`, `winmm`, `imm32`, `ntdll`, `dxgi`, `d3d11`, `oleaut32`,
`api-ms-win-core-winrt-l1-1-0`, `api-ms-win-core-winrt-error-l1-1-0`, `bcryptprimitives`,
`userenv`, `VCRUNTIME140` et six `api-ms-win-crt-*` (`KERNEL32` et `USER32` apparaissent deux fois, en majuscules). Il n’y a aucun import différé.

Aucune bibliothèque réseau n'y figure : ni `ws2_32`, `mswsock`, `winhttp`, `wininet`, `urlmon`,
`dnsapi`, `iphlpapi`, `webio`. Dans [dumpbin-imports.txt](privacy/dumpbin-imports.txt), le seul nom
qui contient « connect » est `ConnectNamedPipe`, le canal local de D18.

Les noms de DLL présents comme chaînes, donc chargeables dynamiquement, sont les mêmes, plus
`dbghelp.dll` (traces de pile de la bibliothèque standard), `amd_ags_x64.dll` (informations du
pilote graphique AMD, dans `gpui-pre-windows`) et `shcore.dll`. Aucune n'est une bibliothèque réseau.

Les adresses `http(s)://` du binaire sont des espaces de noms XML et SVG, les mentions de
fonderie de la police IBM Plex, `https://slint.dev`, un lien vers un ticket `muda` dans un message
d'erreur, et les trois adresses GitHub de `src/runtime.rs` (guide, section « Réglages illisibles »,
notices). Ces trois adresses sont passées à `explorer.exe` (section 2.4).

### 2.3 Essai à l'exécution

L'essai s'est fait sous le verrou `D:\Dev\PowerPlanSync\.audit\live.lock`, avec le binaire
release, en démonstration et dans des dossiers temporaires isolés. Les points de terminaison ont
été relevés toutes les cinq secondes avec `Get-NetTCPConnection -OwningProcess` et
`Get-NetUDPEndpoint -OwningProcess`, pour le processus lancé et tous ses enfants.

| Lancement | Durée | Relevés | Processus suivis | TCP | UDP |
| --- | --- | --- | --- | --- | --- |
| `--demo --gpui --data-dir <temp>` | 180 s | 33 | suivi (pid 22188) et `power-plan-sync.exe --window` (pid 13984) | 0 | 0 |
| `--demo --data-dir <temp>` (Slint) | 120 s | 23 | suivi (pid 16452), aucun enfant | 0 | 0 |

Les deux processus ont été arrêtés de force à la fin, ce qui reste sans effet sur le plan en
démonstration. Le plan actif était « Haute performance » avant et après l'essai, et la clé `Run`
n'a pas changé. Le verrou a été retiré dans le bloc de nettoyage. Fichiers écrits par chaque
lancement : `demo.lock` (0 o) et `simulation\activity.log` (187 o), avec une seule ligne :

```text
1793532632	état	Automatisation désactivée · aucun plan ne sera changé | Équilibré (simulation) | Ajoutez un logiciel, confirmez le plan de retour, puis activez l’automatisation.
```

### 2.4 Navigateur ouvert par la personne

Les boutons « Ouvrir le guide » (page Aide et message de réglages illisibles) et « Voir les
licences » lancent `explorer.exe` avec une adresse `https://github.com/Thanaen/PowerPlanSync/…`.
« Voir les licences » ouvre d'abord le fichier de notices local s'il est présent à côté de
l'exécutable. Windows ouvre alors le navigateur par défaut, qui se connecte à GitHub.
Cette connexion appartient au navigateur, sur demande explicite. PowerPlanSync n'envoie rien
et ne lit pas la réponse. Aucun parcours ne dépend de ces pages, donc l'application fonctionne
sans réseau (FR-020).

## 3. Données enregistrées

Toutes restent dans le dossier de données. Aucune n'est transmise.

| Fichier | Contenu |
| --- | --- |
| `config.json` | `schema_version`, `rules` (`id`, `label` = nom de l'exécutable, `executable` = chemin complet en unités UTF-16, `plan` = GUID, `enabled`), `return_plan`, `automation_enabled`, `paused`, `allow_on_battery`, `start_at_login`. |
| `config.backup.json` | La configuration précédente, mêmes champs. |
| `recovery.json` | `schema_version`, `owner` (`user` = SID du compte, préfixé `demo:` en démonstration ; `session_id`), `cycle` (`id`, `initial`, `return_plan`, `last_confirmed` = GUID de plans, `winner` = identifiant de règle, `acquired`), `transaction`, `before`, `intended`, `phase`. Ou `null` hors cycle. |
| `*.tmp` | Copie temporaire de l'écriture durable, renommée aussitôt par `MoveFileExW`. |
| `demo.lock` | Fichier vide, verrou exclusif d'une démonstration. |
| `activity.log`, `activity.previous.log` | Diagnostics (section 4). |

Les seules données personnelles sont les chemins des logiciels choisis et le SID du compte.
Elles servent à reconnaître le logiciel exact (FR-023) et à refuser un journal d'un autre compte (D07).

Données en mémoire seulement :

- chaque seconde, le suivi ne garde que les processus dont le nom correspond à une règle, puis
  ceux dont le chemin complet est identique (`observe_paths`) ;
- la liste complète des applications de la session (`running_apps`) n'est lue qu'à l'ouverture
  du choix « Choisir parmi les logiciels ouverts ». Elle est envoyée à la fenêtre, puis remplacée
  au choix suivant. Elle n'est ni enregistrée ni journalisée ;
- le canal entre les deux processus transporte le modèle d'affichage et les actions de la fenêtre,
  sur la même machine et pour le même compte.

## 4. Diagnostics (FR-021)

### Contenu

`src/diagnostics.rs` écrit des lignes `secondes UNIX<TAB>événement<TAB>détail`. Il remplace
les caractères de contrôle par des espaces et coupe l'événement à 48 caractères et le détail à
2 048. Seul le processus de suivi détenteur du contrôle ouvre le journal. Il en existe trois
sortes, toutes dans `src/runtime.rs` :

| Événement | Détail |
| --- | --- |
| `état` | Ligne d'état, nom du plan observé et raison affichée. La raison peut contenir le libellé de la règle active, c'est-à-dire un nom d'exécutable comme `bf6.exe`. |
| `erreur` | Forme `Debug` de l'erreur du domaine : `Power(<message Windows>)`, `Storage(<message système ou JSON>)`, `MissingPlan(PlanId(<nombre>))`, `InvalidConfiguration(<message fixe>)`, `ObservationUncertain`, `ForeignOwner`, `StaleTransaction`. |
| `fenêtre` | Une des trois phrases fixes de `src/tracker.rs` sur la perte du processus de fenêtre. |

Les messages `std::io::Error` et `serde_json` ne contiennent pas de chemin : ils donnent le code
système, ou la ligne et la colonne. Aucun événement n'enregistre de chemin complet, d'argument
de lancement, de liste d'applications, de PID ou de mémoire de processus.

### Rotation et limite

`FILE_LIMIT` vaut 5 Mio / 2 = 2 621 440 octets par fichier. Deux fichiers seulement :

- avant une écriture qui dépasserait la limite, `activity.log` est copié sur
  `activity.previous.log`, puis vidé ;
- à l'ouverture, un `activity.log` trop grand est vidé sans copie, et un `activity.previous.log`
  trop grand est vidé.

Une ligne mesure au plus environ 8,2 Kio (2 048 caractères de 4 octets au plus, plus l’horodatage et l’événement). Les deux
fichiers totalisent donc au plus 5 Mio par dossier de données. Le test
`rotation_bounds_all_owned_logs_and_keeps_the_latest_event` écrit 3 001 lignes d'environ 4 Kio.
Il vérifie deux fichiers, au plus 5 Mio au total, et la dernière ligne intacte. Il passe sur cette
machine (`cargo test --locked --lib diagnostics`).

Les diagnostics ne sont jamais supprimés automatiquement. Ils restent dans le dossier de données,
que la personne peut ouvrir et supprimer.

## 5. Points notés sans violation

1. **Code HTTP inactif dans le binaire.** Les types de `gpui-pre-http-client` et de `url` sont
   compilés, sans implémentation réseau (section 2.1). Une future version de GPUI qui installerait
   un vrai client par défaut serait visible dans `cargo tree` et dans les imports. Il faut refaire
   ce contrôle à chaque mise à jour de `gpui-pre`.
2. **Chemins de construction dans le binaire.** Le binaire contient 1 703 occurrences du chemin
   `C:\Users\<compte>\.cargo\…` de la machine de construction, et deux chemins vers le dossier de
   travail. Ce sont des emplacements de panique, pas des données de l'utilisateur. Pour T044, une
   construction avec `--remap-path-prefix` (ou `trim-paths`) éviterait de publier le nom du compte
   de construction.
3. **Verrou global avec DACL par défaut.** Un autre compte local peut créer
   `Global\PowerPlanSync.Controller` avant PowerPlanSync. L'application refuse alors le contrôle
   avec un message (D07, FR-016), sans écrire de plan et sans élévation. C'est un refus de service
   local, conforme à la règle « un verrou inaccessible interdit les écritures ».

## Non exécuté

- Capture des paquets : non exécutée. Les relevés toutes les cinq secondes pourraient manquer une
  connexion très brève. L'absence de `ws2_32.dll` et des autres bibliothèques réseau couvre ce cas.
- Essai sans `--demo` : non exécuté, pour ne pas écrire de plan réel. Le code réseau, absent,
  est le même dans les deux modes.
- Clics sur « Ouvrir le guide » et « Voir les licences » : non exécutés. Leur comportement est lu
  dans `src/runtime.rs`.
- Écriture réelle de la clé `Run` : non exécutée dans cet audit. Elle est couverte par les tests de
  `src/windows/startup.rs` et par T033.
