# Modèle de menaces — pgpilot

## Portée

- Gestionnaire de clefs OpenPGP, application desktop single-user.
- Délègue toutes les opérations cryptographiques à GnuPG (binaire externe).
- Ne manipule jamais directement le matériel cryptographique (clefs privées) : pgpilot
  n'accède ni ne lit les clefs privées stockées dans `gpg-agent`.

---

## Actifs à protéger

| Actif | Description |
|---|---|
| **Clefs privées** | Stockées dans `gpg-agent`, jamais lues par pgpilot. |
| **Keyring GnuPG** | Ensemble des clefs publiques et de l'état de confiance (`TrustLevel`). |
| **Toile de confiance** | Niveaux Marginal / Full / Ultimate attribués par l'utilisateur ; pilote les décisions de chiffrement. |

---

## Menaces identifiées

### 1. Clef malveillante importée

**Description** : un attaquant contrôle le UID affiché dans pgpilot. Le champ
`signer_name` provient du token `GOODSIG` de GnuPG, qui n'est pas authentifié
cryptographiquement — seule la signature elle-même l'est. Un attaquant peut
donc créer une clef avec un UID imitant une identité connue.

**Statut : Atténuée**

- Le badge de confiance (`TrustLevel`) est affiché systématiquement à côté de
  chaque clef dans la liste et le panneau de détail.
- Les clefs dont le niveau est `Undefined` n'affichent pas de badge vert.
- L'utilisateur est responsable de vérifier l'empreinte par un canal secondaire
  avant d'élever la confiance.

---

### 2. MitM sur keyserver HTTP

**Description** : un attaquant positionné en homme-du-milieu substitue une
clef différente lors d'une importation depuis un serveur de clefs.

**Statut : Atténuée**

- HTTPS uniquement sur tous les appels réseau (`ureq` avec TLS activé par
  défaut).
- Les certificats TLS sont vérifiés par la pile de confiance du système
  d'exploitation.
- L'importation affiche toujours l'empreinte complète (40 hex) de la clef
  reçue, permettant une vérification manuelle.

---

### 3. OOM via réponse réseau surdimensionnée

**Description** : un serveur de clefs malveillant (ou compromis) retourne une
réponse de plusieurs gigaoctets, provoquant un épuisement de la mémoire vive
(`Out Of Memory`).

**Statut : Atténuée**

- Limite de 1 Mio appliquée sur toutes les lectures réseau dans `gpg/keyring.rs`
  (`read_to_string` limité via `take(1 << 20)`).
- Au-delà de cette limite, la lecture est interrompue et une erreur explicite
  est renvoyée à l'utilisateur.

---

### 4. Injection de paramètres HKP

**Description** : une requête vers un serveur HKP construite avec un
fingerprint ou une adresse e-mail non encodée permet à un attaquant de forger
l'URL et d'injecter des paramètres arbitraires.

**Statut : Atténuée**

- Validation stricte à l'entrée : 40 hex (fingerprint complet), 16 hex (key ID
  long) ou format e-mail reconnu.
- Encodage URL complet de tous les paramètres avant construction de la requête
  HTTP.
- Les valeurs non conformes sont rejetées avec un message d'erreur explicite,
  sans tentative de correction silencieuse.

---

### 5. Fingerprint court — attaque Evil32

**Description** : les identifiants courts sur 8 caractères (32 bits) sont
triviallement collisibles. Un attaquant peut générer une clef dont les 8
derniers caractères de l'empreinte coïncident avec une clef ciblée.

**Statut : Atténuée**

- Validation de 40 caractères hexadécimaux (160 bits) obligatoire sur toutes
  les fonctions du module `gpg/`.
- Les identifiants 16 hex (key ID long, 64 bits) sont acceptés uniquement pour
  la recherche sur keyserver, jamais pour les opérations cryptographiques
  directes.
- `key_id` dans `KeyInfo` et `SubkeyInfo` est systématiquement le long ID
  (16 derniers hex de l'empreinte).

---

### 6. Fichier forgé via drag-and-drop

**Description** : un fichier malveillant (p. ex. bombes de décompression,
contenu non sollicité) est déposé par l'utilisateur dans la zone de
chiffrement. pgpilot le transmet à GnuPG sans inspection préalable.

**Statut : Résiduelle**

- pgpilot chiffre ce que l'utilisateur dépose explicitement ; il n'interprète
  pas le contenu du fichier.
- La vérification de la légitimité du fichier avant chiffrement est à la charge
  de l'utilisateur.
- GnuPG ne décompresse ni n'exécute le contenu ; le risque est limité à
  l'exposition de données non souhaitées.

---

### 7. Presse-papiers exposé

**Description** : une clef publique armoriée, une URL `paste.rs` ou toute autre
donnée copiée dans le presse-papiers reste accessible à tous les processus de
la session X11 ou Wayland jusqu'à remplacement.

**Statut : Résiduelle**

- Sur Wayland, la durée de vie du presse-papiers est liée au focus de la
  fenêtre source ; un effacement automatique fiable n'est pas possible sans
  contrôle du compositeur.
- Sur X11, aucun mécanisme standard n'empêche un autre processus de lire le
  presse-papiers.
- pgpilot n'y copie jamais de matériel privé (clef secrète, passphrase).

---

### 8. Enregistrement d'écran (screen recording)

**Description** : un logiciel d'enregistrement d'écran (légitime ou malveillant)
capture les données affichées : empreintes, UIDs, statuts de confiance, contenu
de fichiers sélectionnés.

**Statut : Hors-scope**

- La protection contre l'enregistrement d'écran relève du niveau OS /
  compositeur.
- pgpilot n'affiche pas de matériel privé (clefs secrètes, passphrases).

---

### 9. LD_PRELOAD / injection d'environnement

**Description** : une bibliothèque malveillante injectée via `LD_PRELOAD` ou
une variable d'environnement altérée dans les sous-processus `gpg` pourrait
intercepter ou modifier les opérations cryptographiques.

**Statut : Atténuée**

- `env_clear()` est appelé sur chaque `Command` gpg, puis seules les variables
  nécessaires (`GNUPGHOME`, `PATH`) sont transmises explicitement.
- L'environnement hérité du processus parent (potentiellement compromis) est
  ainsi écarté.

---

### 10. GNUPGHOME non défini ou inattendu

**Description** : si `GNUPGHOME` n'est pas défini, GnuPG utilise le répertoire
par défaut (`~/.gnupg`), ce qui peut provoquer des opérations sur un keyring
inattendu dans des environnements de test ou de CI.

**Statut : Atténuée**

- La fonction `gnupg_dir()` (`gpg/mod.rs`) est appelée en tête de chaque
  opération gpg et retourne une erreur explicite si la variable n'est pas
  définie.
- Aucune opération ne se replie silencieusement sur un chemin par défaut.

---

## Hors-scope

Les menaces suivantes sont reconnues mais ne font pas partie du périmètre de
pgpilot :

- **Compromission de la machine locale** — OS, RAM, disque (keylogger, rootkit,
  accès physique).
- **Attaque physique sur YubiKey / smartcard** — extraction du matériel
  cryptographique par side-channel ou décapsulation.
- **Compromission de la chaîne TLS** — autorité de certification compromise ou
  attaque HTTPS à grande échelle.
- **Attaques par canal auxiliaire sur le matériel** — timing, consommation
  électrique, rayonnement électromagnétique.

---

## Hypothèses de sécurité

Le modèle de menaces repose sur les hypothèses suivantes. Si l'une d'elles est
violée, les atténuations décrites peuvent devenir inefficaces.

| Hypothèse | Justification |
|---|---|
| Le binaire `gpg` dans `PATH` est de confiance | pgpilot délègue toute la cryptographie à GnuPG. |
| `gpg-agent` n'est pas compromis | Les clefs privées y sont stockées et protégées. |
| `pinentry` fonctionne correctement et n'est pas remplacé | La saisie de passphrase doit être isolée du reste de la session. |
| L'OS n'est pas compromis au moment de l'exécution | Les primitives de sécurité (mémoire, processus, filesystem) doivent être fiables. |

---

## Chat PGP (v0.6.0)

La v0.6.0 introduit un sous-système de chat chiffré PGP de bout en bout transporté
via MQTT (TLS). Les messages sont éphémères par conception : seuls les salons
sont persistés (`~/.config/pgpilot/rooms.yaml`), jamais le contenu des messages.

### Nouveaux actifs à protéger

| Actif | Description |
|---|---|
| **`rooms.yaml`** | Liste des salons et de leurs participants (fingerprints, dates de jointure, relay MQTT, identité locale `my_fp`). N'inclut aucun contenu de message. Plafonné à 1 Mio à la lecture. |
| **Messages en RAM** | `chat_messages: HashMap<RoomId, VecDeque<ChatMessage>>` — déchiffrés, bornés à 500 entrées par salon (FIFO), perdus à la fermeture de l'application. |
| **Connexion MQTT over TLS** | Socket persistante vers le broker (`mqtts://`). Transporte les blobs PGP chiffrés et les payloads de présence/ACK. |
| **`my_fp` par room** | Identité PGP locale utilisée dans chaque salon. Un utilisateur multi-clef peut séparer identité pro et perso par salon. |
| **Join codes** | Invitations partagées hors-bande (`pgpilot:join:<base64url>`). Contiennent `room_id`, `relay`, `invited_by` et une signature PGP de l'invitant. |
| **`ChatCryptoCtx`** | Contexte crypto résident en RAM pour la durée de la session : `homedir` GPG + `local_fp`. Toutes les opérations passent par des subprocesses `gpg`. |

### Nouvelles menaces et contre-mesures

| Menace | Scénario | Contre-mesure | Résiduel |
|---|---|---|---|
| **Lecture des messages par le broker** | Un broker compromis ou malveillant inspecte les payloads transitant sur le topic chat. | Chiffrement E2E PGP multi-destinataires (`gpg --encrypt --sign --armor`). Le broker ne voit que des blobs PGP armored. | Le broker observe les timestamps de publication, les topics, et la cardinalité des participants. |
| **Usurpation d'identité émetteur** | Un attaquant modifie `wire.sender` dans le JSON pour faire passer son message pour un autre. | `decrypt_message` exige le token `[GNUPG:] VALIDSIG <fp40>` et compare strictement le fingerprint extrait avec ce qui est attendu côté handler. La signature étant intégrée au message PGP, modifier `wire.sender` ne change pas le résultat de la vérification. | Limité aux clefs présentes dans le keyring local : un signataire inconnu produit `ChatError::SignatureInvalid` et le message est ignoré. |
| **Injection de messages par tiers** | Un participant externe (qui connaît le topic) publie un message chiffré pour un destinataire valide du salon. | À la réception, `signer_fp` (extrait de `VALIDSIG`) est comparé à `room.participants` avant affichage. Tout signataire hors liste est rejeté silencieusement. | Quiconque connaît le topic peut publier ; la confidentialité reste assurée car seuls les destinataires listés peuvent déchiffrer. |
| **Fausse invitation (redirection broker)** | Un join code forgé pointe vers un broker contrôlé par l'attaquant pour intercepter ou bloquer le trafic. | `JoinCode::verify()` vérifie la signature PGP de l'invitant via `gpg --batch --verify` sur la canonicalisation `room_id ‖ \x00 ‖ relay ‖ \x00 ‖ invited_by`. Le relay doit commencer par `mqtts://` (sauf `mqtt://localhost` / `mqtt://127.x.x.x` pour les tests locaux) — sinon `ChatError::InvalidJoinCode`. | L'invitant doit déjà être présent dans le keyring local ; sinon l'invitation est rejetée (`ChatError::JoinCodeInviterUnknown`). L'utilisateur doit importer la clef de l'invitant avant d'accepter. |
| **Usurpation de présence** | Un attaquant publie un payload `online` ou `offline` pour le compte d'un fingerprint tiers. | Aucune en v0.6.0 : la présence n'est pas signée. Limitation explicitement documentée. | Faux statut Online/Offline possible par broker malveillant ou tiers connaissant le topic présence. Sans impact sur la confidentialité ni l'authenticité des messages. |
| **Replay de messages** | Un attaquant rejoue un ancien `WireMessage` capturé pour faire croire qu'un message est récent. | Validation à la réception : `|wire.ts - now| ≤ 86400 s` (fenêtre de 24 h). La signature PGP intégrée couvre `id`, `sender`, `ts`, `payload` (canonicalisation `SIGN_CANONICAL_PREFIX ‖ id ‖ \x00 ‖ sender ‖ \x00 ‖ ts ‖ \x00 ‖ payload`) — modifier `ts` casse la signature. | Replay possible à l'intérieur de la fenêtre 24 h si le `msg_id` n'a pas déjà été vu (dédup côté récepteur). |
| **DoS via gros messages** | Un attaquant publie un payload de plusieurs Mio sur le topic chat pour saturer la mémoire du client. | `mqtt_task` rejette les payloads > 64 Kio (`MAX_WIRE_MESSAGE_BYTES`) avant copie en mémoire. Validation appliquée côté émetteur ET récepteur. | Dépend du broker : un broker mal configuré pourrait accepter des payloads plus larges, mais le client les jette avant déchiffrement. |
| **Leak `room_id` via topic** | Un observateur déduit le nom ou l'identité d'un salon depuis le topic MQTT. | Le topic chat est dérivé via `sha256(room_id)[0..8]` en hex (16 caractères) — opaque. Le nom du salon n'apparaît jamais sur le wire. | 64 bits du hash de `room_id` exposés sur le broker ; cardinalité et corrélation temporelle restent observables. |
| **Surveillance des métadonnées** | Le broker observe qui publie, quand, et sur quels topics. | Topics opaques (hash tronqué). Fingerprints des fingerprints tronqués à 16 hex (64 bits) dans les topics présence/ACK. Pas de liste de destinataires sur le wire — implicite dans les session keys PGP. | Le broker voit le timing et la cardinalité de chaque salon. C'est une limite inhérente à MQTT public. |
| **`rooms.yaml` malveillant** | Un fichier `rooms.yaml` corrompu ou trop volumineux (rempli par un autre processus) cause une consommation mémoire excessive. | Vérification de taille via `metadata().len()` avant lecture : rejet au-delà de 1 Mio (`MAX_ROOMS_YAML_BYTES`) avec `ChatError::RoomsYamlLoadFailed`. Validation des champs `JoinCode` après désérialisation (UUID, fingerprint 40-hex, schéma relay, taille `room_name`). | Corruption partielle possible : un salon malformé n'empêche pas le chargement du reste, mais provoque l'échec global de `serde_yaml::from_str`. |
| **Forward secrecy absente** | Un attaquant capture les blobs PGP en transit, puis compromet la clef privée plus tard ; il peut alors déchiffrer rétrospectivement les messages. | Aucune en v0.6.0 — documentée hors-scope. Mitigation indirecte : le broker ne persiste rien (QoS 1, pas de retain sur le topic chat), donc la fenêtre d'enregistrement de l'attaquant est limitée à ce qu'il a pu intercepter en temps réel. | Pas de PFS (Perfect Forward Secrecy) en v0.6.0. La compromission d'une clef privée expose tous les messages capturés et déchiffrables. |

### Hors-scope explicite v0.6.0

Les éléments suivants sont reconnus mais ne font pas partie du périmètre de la v0.6.0 :

- **Forward secrecy** : pas de Double Ratchet, X3DH ou per-message ephemeral keys. La compromission ultérieure d'une clef privée permet de déchiffrer les messages capturés en temps réel.
- **Signature des payloads de présence** : les statuts Online/Offline ne sont pas signés. Un broker malveillant peut publier de faux statuts.
- **YubiKey et touch policy** : les opérations chat (chiffrement, signature, déchiffrement) passent par des subprocesses `gpg` qui délèguent au `gpg-agent` — les clefs YubiKey fonctionnent donc normalement. Limitation pratique : une carte configurée en "touch requis par opération" demandera un toucher physique pour chaque message envoyé et chaque message reçu. Recommandation : utiliser la politique "touch une fois par session" ou une clef logicielle dédiée au chat.
- **Persistance chiffrée des messages** : conformément à l'exigence "éphémère par conception". Aucun message n'est jamais écrit sur disque. Reporté à v0.7+ avec chiffrement local SQLite + sequoia.
- **Authentification broker** : pas de support `user`/`password` ni de mTLS client cert dans `MqttConfig`. La v0.6.0 cible des brokers publics ou privés sans authentification.
- **Multi-device** : un même utilisateur ne peut pas synchroniser ses salons et ses messages entre deux instances pgpilot.
- **Zeroize de `ChatCryptoCtx`** : pas de wipe explicite de la mémoire au quit. Mitigation OS-level uniquement (mémoire libérée à la fermeture, pas de swap recommandé sur les machines sensibles).
- **Modération / kick / ban** : un participant retiré reste capable de lire les messages capturés ; la révocation effective nécessiterait un re-keying complet du salon (non implémenté).
