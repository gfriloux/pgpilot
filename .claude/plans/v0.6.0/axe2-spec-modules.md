# Axe 2 — Spec technique des modules chat (T2.1)

> Document de conception. Aucun code n'est écrit dans cet axe. Cette spec définit
> le layout, les types, les contrats internes et les points d'intégration des
> nouveaux modules `chat` dans pgpilot. Elle sert de référence pour l'axe 2.2
> (design API) et la fusion T2.3, puis pour l'implémentation des axes 3–8.

---

## 1. Layout des nouveaux modules

```
src/
├── chat/
│   ├── mod.rs        — re-exports publics + types partagés (ChatError, ChatId, etc.)
│   ├── rooms.rs      — Room, RoomStore (load/save rooms.yaml), CRUD, JoinCode encode/decode
│   ├── mqtt.rs       — MqttHandle, ChatTransport trait, run_client(), reconnect loop
│   ├── crypto.rs     — encrypt_for_room(), decrypt_message(), sign(), verify_sender()
│   └── presence.rs   — PresenceStatus, PresenceTracker, build_lwt(), encode/decode payloads
```

Une nouvelle racine `src/chat/` est ajoutée à `src/lib.rs`/`src/main.rs` (déclaration
`pub mod chat;`). Elle est volontairement séparée de `src/gpg/` : la couche `gpg`
reste dédiée aux opérations sur le keyring local (subprocess `gpg`) ; `chat`
combine sequoia (chiffrement in-process) + MQTT.

### 1.1 `src/chat/mod.rs`

Rôle : façade publique + types partagés. Suit le pattern de `src/gpg/mod.rs`
(re-exports en bas, helpers `pub(crate)` en haut).

**Types publics** :

```rust
pub use rooms::{Room, RoomStore, JoinCode};
pub use mqtt::{MqttHandle, MqttEvent, MqttConfig, ChatTransport};
pub use crypto::{ChatPayload, VerifiedMessage};
pub use presence::{PresenceStatus, PresenceUpdate, PresenceTracker};

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
  #[error("MQTT non connecté")]
  MqttNotConnected,
  #[error("Échec chiffrement : {0}")]
  EncryptFailed(String),
  #[error("Échec déchiffrement : {0}")]
  DecryptFailed(String),
  #[error("Signature invalide")]
  SignatureInvalid,
  #[error("Émetteur inconnu : {0}")]
  UnknownSender(String),
  #[error("Salon introuvable : {0}")]
  RoomNotFound(String),
  #[error("Code d'invitation invalide")]
  InvalidJoinCode,
  #[error("Configuration invalide : {0}")]
  InvalidConfig(String),
}

pub type ChatResult<T> = std::result::Result<T, ChatError>;

/// Identifiant de salon (UUID v4 sous forme String). Newtype optionnel pour
/// éviter les confusions avec d'autres String.
pub type RoomId = String;

/// Fingerprint 40 hex (réutilise validate_fp de gpg::keyring).
pub type Fingerprint = String;
```

**Helpers `pub(crate)`** : aucun pour l'instant — chaque sous-module expose
ses helpers nécessaires.

**Dépendances externes ajoutées au Cargo.toml** (livrées par axe 3) :
- `rumqttc = "0.24"` (client MQTT async, tokio-friendly, déjà compatible avec
  notre runtime tokio)
- `uuid = { version = "1", features = ["v4", "serde"] }`
- `base64 = "0.22"` pour join codes
- `thiserror = "1"` pour `ChatError`
- `serde_json = "1"` pour le wire format

### 1.2 `src/chat/rooms.rs`

Rôle : persistance YAML des salons et codes d'invitation. Calque le pattern de
`src/config/mod.rs` (load/save via `serde_yaml`).

**Types publics** :

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
  pub id: RoomId,                         // UUID v4
  pub name: String,                       // libellé local (jamais transmis)
  pub relay: String,                      // "mqtts://broker:8883"
  pub participants: Vec<Fingerprint>,     // 40-hex chacun
  pub created_at: chrono::DateTime<chrono::Utc>,
  #[serde(default)]
  pub joined_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoomStore {
  pub rooms: Vec<Room>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinCode {
  pub room_id: RoomId,
  pub relay: String,
  pub invited_by: Fingerprint,
  pub room_name: Option<String>,
}
```

**Fonctions publiques** :

```rust
impl RoomStore {
  pub fn path() -> std::path::PathBuf;          // ~/.config/pgpilot/rooms.yaml
  pub fn load() -> ChatResult<Self>;            // tolère fichier absent → Self::default()
  pub fn save(&self) -> ChatResult<()>;         // crée le dossier si besoin (fs::create_dir_all)
  pub fn get(&self, id: &str) -> Option<&Room>;
  pub fn upsert(&mut self, room: Room);
  pub fn remove(&mut self, id: &str) -> Option<Room>;
}

impl Room {
  pub fn new(name: String, relay: String, participants: Vec<Fingerprint>) -> Self;
  /// Topic MQTT principal du salon (cf. axe 2.2 / axe 3).
  pub fn chat_topic(&self) -> String;           // pgpilot/chat/{sha256(id)[0..16]}
}

impl JoinCode {
  pub fn encode(&self) -> ChatResult<String>;   // pgpilot:join:<base64url(json)>
  pub fn decode(s: &str) -> ChatResult<Self>;   // strip prefix → base64 → json
  pub fn into_room(self, name_override: Option<String>) -> Room;
}
```

**Dépendances internes** :
- `chat::mod` (ChatError, RoomId, Fingerprint)
- `crypto::keyring::validate_fp` (réexporté via `gpg::validate_fp` ou copié si la
  visibilité l'interdit) pour valider les fingerprints au load.

### 1.3 `src/chat/mqtt.rs`

Rôle : connexion MQTT, boucle `eventloop`, publication/abonnement, reconnect.
**Toutes les opérations réseau sont async** (tokio). Pas de `blocking_task` ici.

**Types publics** :

```rust
pub struct MqttConfig {
  pub relay: String,        // "mqtts://host:8883" ou "mqtt://host:1883"
  pub client_id: String,    // dérivé du fingerprint local (16 hex)
  pub keepalive_secs: u16,  // défaut 30
  pub will: Option<LastWill>, // payload présence offline (rétention)
}

pub struct LastWill {
  pub topic: String,
  pub payload: Vec<u8>,
  pub retain: bool,
  pub qos: u8,
}

/// Évènements émis par la tâche de fond MQTT vers iced.
#[derive(Debug, Clone)]
pub enum MqttEvent {
  Connected,
  Disconnected(String),                  // raison
  Reconnecting { attempt: u32 },
  Message { topic: String, payload: Vec<u8>, retained: bool },
  PublishAck { packet_id: u16 },         // utilisé pour ACK applicatif (cf. axe 6)
  Error(String),
}

/// Handle thread-safe vers la tâche MQTT, partagé via Arc dans App.
#[derive(Clone)]
pub struct MqttHandle {
  cmd_tx: tokio::sync::mpsc::Sender<MqttCmd>,
  // PAS de référence directe au client rumqttc — tout passe par cmd_tx
}

#[derive(Debug)]
pub(crate) enum MqttCmd {
  Subscribe { topic: String, qos: u8 },
  Unsubscribe { topic: String },
  Publish { topic: String, payload: Vec<u8>, qos: u8, retain: bool },
  Shutdown,
}

/// Trait pour permettre l'injection d'un transport mock dans les tests.
#[async_trait::async_trait]
pub trait ChatTransport: Send + Sync {
  async fn subscribe(&self, topic: &str, qos: u8) -> ChatResult<()>;
  async fn unsubscribe(&self, topic: &str) -> ChatResult<()>;
  async fn publish(&self, topic: &str, payload: Vec<u8>, qos: u8, retain: bool) -> ChatResult<()>;
}
```

**Fonctions publiques** :

```rust
/// Démarre la boucle MQTT en arrière-plan. Renvoie un handle pour piloter le
/// client + un Stream d'évènements à brancher sur l'iced subscription.
pub fn spawn(
  config: MqttConfig,
) -> (MqttHandle, impl futures_core::Stream<Item = MqttEvent> + Unpin + Send + 'static);

impl MqttHandle {
  pub async fn subscribe(&self, topic: &str, qos: u8) -> ChatResult<()>;
  pub async fn unsubscribe(&self, topic: &str) -> ChatResult<()>;
  pub async fn publish(&self, topic: &str, payload: Vec<u8>, qos: u8, retain: bool) -> ChatResult<()>;
  pub async fn shutdown(self);
}

#[async_trait::async_trait]
impl ChatTransport for MqttHandle { /* delegate to channel */ }
```

**Implémentation interne** :
- `tokio::spawn` une tâche qui possède le `rumqttc::AsyncClient` et le
  `EventLoop`.
- Boucle : `select!` entre `event_loop.poll()` (réception) et `cmd_rx.recv()`
  (commandes UI). Sur `Incoming::Publish`, on émet `MqttEvent::Message` via le
  stream sortant.
- Reconnect : sur `Outgoing::Disconnect` ou erreur de poll, émet
  `Disconnected` puis `Reconnecting { attempt }` avec backoff exponentiel
  borné (1 s → 30 s, jamais d'arrêt). rumqttc gère déjà le retry implicite ;
  on ajoute juste l'émission d'évènements et un cap.
- Ne touche **jamais** au keyring ni à `App` — pure couche transport.

**Dépendances internes** :
- `chat::mod::{ChatError, ChatResult}`
- `tokio::sync::mpsc` pour le canal commandes
- `tokio::sync::broadcast` ou `mpsc` pour le stream sortant (préférence : `mpsc`
  illimité côté UI, ou `broadcast` si on veut plusieurs consommateurs)

### 1.4 `src/chat/crypto.rs`

Rôle : chiffrement/déchiffrement/signature des messages chat **en
process** via sequoia, sans appeler le binaire `gpg`. C'est la différence
fondamentale avec `src/gpg/keyring.rs` : pas de subprocess, pas de pinentry, on
travaille à partir des Cert sequoia chargés depuis le keyring.

**Justification** : un appel `gpg --encrypt` par message serait inacceptable en
latence (200–500 ms par opération), et ferait apparaître pinentry pour la
signature. On charge la clé secrète **une seule fois** par session (au démarrage
du chat) en mémoire, et on l'utilise pour signer chaque message.

**Types publics** :

```rust
/// Charge utile binaire prête à être encapsulée dans un WireMessage.
/// Contient le PGP message ASCII-armored.
#[derive(Debug, Clone)]
pub struct ChatPayload {
  pub ciphertext_armored: String,  // -----BEGIN PGP MESSAGE-----
  pub signature_armored: String,   // -----BEGIN PGP SIGNATURE-----
}

#[derive(Debug, Clone)]
pub struct VerifiedMessage {
  pub plaintext: String,
  pub signer_fp: Fingerprint,
  pub signed_at: chrono::DateTime<chrono::Utc>,
}

/// Cache des Cert chargés depuis le keyring, partagé pour toute la session.
/// Construit une fois au démarrage du chat.
pub struct ChatCryptoCtx {
  /// Clé secrète locale (déchiffrée si pas de passphrase, sinon nécessite
  /// pinentry une fois — cf. note plus bas).
  pub local_cert: sequoia_openpgp::Cert,
  pub local_fp: Fingerprint,
  /// Map fp → Cert pour les destinataires (publics).
  pub peers: HashMap<Fingerprint, sequoia_openpgp::Cert>,
}
```

**Fonctions publiques** :

```rust
impl ChatCryptoCtx {
  /// Charge la clé secrète locale + les certs publics des participants depuis
  /// le keyring gpg. Réutilise gpg::gnupg_dir() pour localiser le homedir et
  /// `gpg --export-secret-keys --armor <fp>` (subprocess, une fois) pour
  /// récupérer la clé secrète, puis la parse via sequoia.
  pub fn load(local_fp: &str, peers: &[Fingerprint]) -> ChatResult<Self>;

  /// Chiffre + signe un message texte pour une room donnée. La signature est
  /// detached, sur le ciphertext (cf. T2.2 — recommandation : signer le tuple
  /// {id, sender, ts, payload}, à confirmer dans la spec finale).
  pub fn encrypt_for_room(
    &self,
    plaintext: &str,
    recipients: &[Fingerprint],
  ) -> ChatResult<ChatPayload>;

  /// Vérifie la signature, déchiffre le message, et renvoie le plaintext +
  /// metadata signataire. Échoue avec UnknownSender si le fingerprint signataire
  /// n'est pas dans `peers`.
  pub fn decrypt_message(&self, payload: &ChatPayload) -> ChatResult<VerifiedMessage>;
}
```

**Note sur la passphrase** : si la clé secrète est protégée par passphrase
(comportement gpg standard), `gpg --export-secret-keys` déclenche pinentry. On
demande donc à l'utilisateur **une seule fois** au démarrage du chat. Pour les
clés sur YubiKey, ce schéma ne fonctionne pas (la clé secrète n'est pas
exportable) : axe 7 doit définir le comportement (refus + message clair, ou
fallback `gpg --encrypt` lent). Recommandation : refus initial, le chat n'est
disponible qu'avec une clé secrète locale (pas YubiKey) en v0.6.0.

**Dépendances internes** :
- `chat::mod::{ChatError, ChatResult, Fingerprint}`
- `gpg::gnupg_dir`, `gpg::gpg_command` (réutilisés pour exporter une fois la
  clé secrète) — nécessite de promouvoir `gnupg_dir` et `gpg_command` à
  `pub(crate)` dans `gpg/mod.rs` (ils le sont déjà).
- `sequoia_openpgp` (déjà dans Cargo.toml)

### 1.5 `src/chat/presence.rs`

Rôle : encodage/décodage des messages de présence, gestion de l'état local.

**Types publics** :

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceStatus {
  Online,
  Offline,
  Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
  pub fingerprint: Fingerprint,
  pub status: String,                  // "online" | "offline"
  pub ts: i64,                         // unix secondes
}

#[derive(Default, Debug, Clone)]
pub struct PresenceTracker {
  /// Dernier statut connu par fingerprint.
  states: HashMap<Fingerprint, (PresenceStatus, chrono::DateTime<chrono::Utc>)>,
}
```

**Fonctions publiques** :

```rust
impl PresenceTracker {
  pub fn new() -> Self;
  pub fn apply(&mut self, update: PresenceUpdate) -> Option<PresenceStatus>;
  pub fn status(&self, fp: &str) -> PresenceStatus;
  pub fn last_seen(&self, fp: &str) -> Option<chrono::DateTime<chrono::Utc>>;
}

/// Topic présence pour un fingerprint.
pub fn presence_topic(fp: &str) -> String;          // pgpilot/presence/{fp[0..16]}

/// Construit le payload "online" à publier au connect.
pub fn online_payload(fp: &str) -> Vec<u8>;

/// Construit le LastWill MQTT (statut "offline" rétention) à fournir au broker.
pub fn build_lwt(fp: &str) -> LastWill;             // dans mqtt::LastWill

/// Décode un payload reçu sur un topic présence.
pub fn decode_payload(bytes: &[u8]) -> ChatResult<PresenceUpdate>;
```

**Dépendances internes** :
- `chat::mod::{ChatError, ChatResult, Fingerprint}`
- `chat::mqtt::LastWill`

### 1.6 Diagramme des dépendances internes

```
                ┌──────────────┐
                │   chat/mod   │  ← types partagés (ChatError, RoomId, …)
                └──┬─────┬──┬──┘
                   │     │  │
        ┌──────────┘     │  └────────────┐
        │                │               │
   ┌────▼────┐    ┌──────▼─────┐    ┌────▼────┐
   │  rooms  │    │   crypto   │    │ presence│
   └────┬────┘    └──────┬─────┘    └────┬────┘
        │                │               │
        │            (gpg::gnupg_dir,    │
        │             gpg::gpg_command,  │
        │             sequoia)           │
        │                                │
        │     ┌──────────────────────────┘
        │     │  (LastWill type vit dans mqtt)
        ▼     ▼
       ┌─────────┐
       │  mqtt   │   ← async, possède rumqttc::AsyncClient
       └─────────┘
```

`rooms`, `crypto`, `presence` sont indépendants entre eux. `mqtt` est
indépendant de tous les autres (pas de circular dep). L'orchestration vit dans
`app/chat.rs`.

---

## 2. Intégration dans `App`

Cette section liste les champs ajoutés à `struct App` (`src/app/mod.rs`) et
leur justification.

### 2.1 Champs ajoutés

```rust
pub struct App {
  // … champs existants …

  // --- chat (axe 2 / v0.6.0) ---

  /// Salons persistés. Chargés au démarrage depuis ~/.config/pgpilot/rooms.yaml.
  pub rooms: Vec<crate::chat::Room>,

  /// Salon actif (room_id). None = vue ChatList ou pas dans la section chat.
  pub active_room: Option<String>,

  /// Messages en RAM par room_id. JAMAIS persistés (cf. exigence éphémère).
  /// Borné par room (cf. §7.3) à 200 derniers messages.
  pub chat_messages: std::collections::HashMap<String, std::collections::VecDeque<crate::chat::ChatMessage>>,

  /// État de présence agrégé pour tous les fingerprints connus.
  pub presence: crate::chat::PresenceTracker,

  /// État de connexion MQTT (Disconnected | Connecting | Connected | Reconnecting{attempt}).
  pub mqtt_state: MqttState,

  /// Handle vers le client MQTT (None tant que pas connecté). Cloneable et
  /// thread-safe (mpsc::Sender interne).
  pub mqtt: Option<crate::chat::MqttHandle>,

  /// Saisie en cours dans la room active (équivalent à create_form).
  pub chat_input: String,

  /// Contexte crypto chargé une fois au démarrage du chat (Cert local + peers).
  /// Wrapped dans Arc pour passage thread-safe vers blocking_task sans clone
  /// coûteux des Cert.
  pub chat_crypto: Option<std::sync::Arc<crate::chat::ChatCryptoCtx>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum MqttState {
  #[default]
  Disconnected,
  Connecting,
  Connected,
  Reconnecting { attempt: u32 },
  Failed(String),
}
```

### 2.2 Justification des choix

**`rooms: Vec<Room>` (et pas `Option<RoomStore>`)** : on veut accéder à la
liste sans pattern-match systématique. Le `RoomStore` n'existe que comme outil
de sérialisation (load/save) et ses méthodes sont de simples helpers.

**`active_room: Option<String>` (fingerprint pattern)** : reproduit le pattern
`selected: Option<String>` pour les clés. Identifiant stable (UUID), survit
aux rechargements. Le composant UI résout `active_room` → `&Room` via une
recherche linéaire (acceptable pour <100 rooms) ou un futur `room_by_id()`.

**`chat_messages: HashMap<String, VecDeque<ChatMessage>>`** : la `VecDeque`
permet le bornage (pop_front au-delà de N) en O(1). HashMap clé = room_id pour
isoler les conversations. **Pas de sérialisation** : conforme à l'exigence
éphémère (cf. axe 7 sécurité) et au feedback "chat éphémère PGP/MQTT" du plan
v0.6.0. Borné à 200 messages par room (cf. §7.3).

**`presence: PresenceTracker`** : encapsule la HashMap interne pour exposer
des méthodes typées (`status()`, `last_seen()`) plutôt qu'un accès brut. Évite
qu'un appelant indexe par String et obtienne `Online` par défaut au lieu de
`Unknown`.

**`mqtt_state: MqttState` (enum, pas `bool`)** : le pattern existant pour
`KeyserverStatus` montre qu'un enum à 4 états est plus expressif qu'un bool +
optionnels. Permet d'afficher "Reconnecting (attempt 3)" dans la status bar.

**`mqtt: Option<MqttHandle>`** : `Option` car la connexion peut échouer ou ne
pas être démarrée (utilisateur n'a pas encore ouvert de room). `MqttHandle`
est `Clone` (contient un `mpsc::Sender`), donc thread-safe pour passage dans
`Task::perform`.

**`chat_input: String`** : champ plat, suit le pattern `create_form.name` /
`import_form.url`. Si la complexité grandit (ex : drafts par room, fichiers
joints), promouvoir à `ChatForm { input, attachments, … }`.

**`chat_crypto: Option<Arc<ChatCryptoCtx>>`** : partagée entre le thread UI et
les `blocking_task` qui chiffrent/déchiffrent. `Arc` pour ne pas cloner les
Cert (volumineux). `None` jusqu'au premier accès au chat → lazy load via
`blocking_task` car `gpg --export-secret-keys` est bloquant.

### 2.3 Initialisation dans `App::new`

```rust
pub fn new() -> (Self, Task<Message>) {
  let config = Config::load().unwrap_or_default();
  // … existant …
  let rooms = crate::chat::RoomStore::load().map(|s| s.rooms).unwrap_or_default();
  let initial_keys_task = Task::perform(blocking_task(crate::gpg::list_keys), Message::KeysLoaded);

  (
    Self {
      // … existant …
      rooms,
      active_room: None,
      chat_messages: HashMap::new(),
      presence: crate::chat::PresenceTracker::new(),
      mqtt_state: MqttState::Disconnected,
      mqtt: None,
      chat_input: String::new(),
      chat_crypto: None,
    },
    initial_keys_task,  // PAS de connexion MQTT ici — démarrée à la 1ère ouverture de room
  )
}
```

**Décision** : on **ne démarre pas MQTT au lancement**. La connexion est
établie quand l'utilisateur sélectionne une room pour la première fois. Évite
des connexions inutiles, et permet à pgpilot de fonctionner sans réseau pour
les opérations clé/chiffrement classique.

---

## 3. Nouveaux variants `View`

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum View {
  #[default]
  MyKeys,
  PublicKeys,
  CreateKey,
  Import,
  Health,
  Encrypt,
  Decrypt,
  Sign,
  Verify,
  Settings,
  // --- nouveau (v0.6.0) ---
  ChatList,
  ChatRoom(String),   // room_id actif
}
```

### 3.1 Cohérence avec `previous_view`

Le code existant (`on_nav_changed`, `nav.rs:53`) ne mémorise `previous_view`
que pour `View::CreateKey | View::Import`. **Étendre la liste** :

```rust
if matches!(view, View::CreateKey | View::Import | View::ChatRoom(_)) {
  self.previous_view = Some(self.view.clone());
}
```

Justification : depuis une `ChatRoom`, le bouton "retour" doit ramener à
`ChatList`. C'est la même sémantique que CreateKey → précédente vue.

### 3.2 Cohérence avec `on_nav_changed`

Le handler actuel reset `self.selected = None`, `reset_pending_ops()`, et le
formulaire decrypt. Pour ChatRoom, **ne PAS** reset `chat_input` lors d'une
nav inter-rooms (sinon perte du draft) — sauf si on définit que les drafts
sont par-room (auquel cas il faudrait `chat_drafts: HashMap<RoomId, String>`).

**Recommandation v0.6.0** : un seul `chat_input` global, vidé à chaque
changement de room (KISS). Promotion à HashMap si demande utilisateur.

À ajouter dans `on_nav_changed` :

```rust
match &view {
  View::ChatRoom(room_id) => {
    self.active_room = Some(room_id.clone());
    self.chat_input.clear();
    return self.ensure_chat_started(room_id.clone());  // démarre MQTT si absent + subscribe
  }
  View::ChatList => {
    self.active_room = None;
  }
  _ => {
    self.active_room = None;
  }
}
```

`ensure_chat_started` est défini en §5.

### 3.3 Pourquoi `ChatRoom(String)` plutôt qu'un champ séparé ?

Iced compare l'enum `View` pour décider du re-render. Si on stocke `room_id`
hors de View, changer de room sans changer de View ne déclencherait pas la
mise en évidence de la sidebar. Avec `ChatRoom(String)`, chaque room est une
"vue distincte" ce qui simplifie le routage UI.

**Conséquence** : la sidebar affiche un bouton "Chat" (= `View::ChatList`).
Les rooms ne figurent pas dans la sidebar — elles vivent dans la liste à
gauche du master-detail (réutilise le pattern `key_list` cf. §5.4).

---

## 4. Nouveaux variants `Message`

Liste **exhaustive** des messages chat. Suit le naming `<Domain><Action>` du
codebase (pas de `ChatRoomCreated` pour distinguer "create" lancé vs "created"
revenu d'async).

```rust
#[derive(Debug, Clone)]
pub enum Message {
  // … existant …

  // --- Chat — création / jointure de salons ---
  ChatRoomCreate,                                    // bouton "Nouveau salon"
  ChatRoomNameChanged(String),                       // saisie nom
  ChatRoomCreated(Result<crate::chat::Room, String>),
  ChatJoinCodeChanged(String),                       // saisie code
  ChatRoomJoin,                                      // bouton "Rejoindre"
  ChatRoomJoined(Result<crate::chat::Room, String>),
  ChatRoomSelected(String),                          // room_id depuis la liste
  ChatRoomLeave(String),                             // bouton quitter
  ChatRoomLeft(Result<String, String>),              // room_id

  // --- Chat — envoi / réception ---
  ChatInputChanged(String),
  ChatSend,                                          // bouton "Envoyer" / Enter
  ChatSent(Result<crate::chat::ChatMessage, String>),
  ChatReceived(String, crate::chat::ChatMessage),    // room_id, message déchiffré
  ChatDecryptFailed(String, String),                 // room_id, raison

  // --- Chat — copie/partage du join code ---
  ChatJoinCodeCopy(String),                          // room_id → encode + clipboard
  ChatJoinCodeCopied(Result<String, String>),

  // --- MQTT infra ---
  MqttEvent(crate::chat::MqttEvent),                 // évènement venant du stream
  MqttCryptoLoaded(Result<std::sync::Arc<crate::chat::ChatCryptoCtx>, String>),

  // --- Présence ---
  PresenceUpdated(crate::chat::PresenceUpdate),

  // --- ACK applicatif (cf. axe 6) ---
  ChatAckSent(Result<(), String>),                   // confirmation publication ACK
  ChatAckReceived(String, String, String),           // room_id, msg_id, sender_fp
}
```

### 4.1 Justifications par groupe

**Création/jointure** : on sépare le clic UI (`ChatRoomCreate`) du retour async
(`ChatRoomCreated`) — pattern identique à `CreateKeySubmit` / `CreateKeyDone`.
Pas de paramètre dans `ChatRoomCreate` car le nom est dans `chat_input` /
forme dédiée (à choisir : `ChatNewRoomForm { name, participants }`). Décision
v0.6.0 : forme dédiée pour éviter de partager `chat_input` (qui sert à l'envoi
de messages).

**Envoi/réception** : `ChatSend` ne porte pas le texte ni le room — il les lit
depuis `self.active_room` et `self.chat_input`. `ChatSent` revient avec le
`ChatMessage` complet (incluant `id` UUID généré côté lib) pour l'insérer en
RAM avec direction = Sent. `ChatReceived` est dispatché depuis le handler
`MqttEvent` qui parse + déchiffre via `blocking_task`.

**`ChatDecryptFailed(String, String)`** : on logue mais on **n'affiche pas un
status bar rouge** pour chaque message corrompu (spam potentiel). Au lieu de
ça, on incrémente un compteur dans `ChatMessage::Error` placeholder.
Décision finale en axe 5 (UI).

**`MqttEvent(MqttEvent)`** : un seul variant porte tous les évènements MQTT.
Le handler `on_mqtt_event` discrimine en interne. Évite l'explosion à 6+
variants pour ce qui est essentiellement le même flux.

**`MqttCryptoLoaded`** : le chargement du `ChatCryptoCtx` est async (export
secret + parse). Distinct de la connexion MQTT.

**`PresenceUpdated`** : porte la struct `PresenceUpdate` complète (fp + status
+ ts) plutôt qu'un tuple — meilleure évolutivité.

**`ChatAckReceived`** : 3 String = room_id, msg_id, sender_fp. Le handler
trouve le message dans `chat_messages[room_id]` par msg_id et marque l'ack
pour ce sender_fp. Si le message n'existe plus (purge), ignore silencieusement.

### 4.2 Routing dans `update()`

Le router conserve son style "une ligne par message". Exemple :

```rust
Message::ChatRoomCreate => self.on_chat_room_create(),
Message::ChatRoomCreated(r) => self.on_chat_room_created(r),
Message::ChatSend => self.on_chat_send(),
Message::ChatSent(r) => self.on_chat_sent(r),
Message::ChatReceived(room_id, msg) => self.on_chat_received(room_id, msg),
Message::MqttEvent(ev) => self.on_mqtt_event(ev),
// etc.
```

Les changements triviaux restent inline :

```rust
Message::ChatInputChanged(v) => { self.chat_input = v; Task::none() }
Message::ChatJoinCodeChanged(v) => { self.chat_new_form.join_code = v; Task::none() }
```

---

## 5. Découpage handlers dans `app/`

Ajout d'**un seul** module : `src/app/chat.rs`. Pas de fragmentation
prématurée — on splittera si le fichier dépasse ~400 lignes (taille typique
des modules existants).

### 5.1 Ajout dans `src/app/mod.rs`

```rust
mod card;
mod chat;          // ← nouveau
mod create;
mod decrypt;
// …
```

### 5.2 Signatures des handlers (`src/app/chat.rs`)

```rust
use iced::Task;
use std::sync::Arc;

use crate::chat::{ChatCryptoCtx, ChatMessage, MqttEvent, MqttHandle, Room};
use super::{blocking_task, App, Message, MqttState, StatusKind, View};

impl App {
  // --- Cycle de vie chat ---

  /// Démarre la connexion MQTT + charge le crypto ctx si pas encore fait.
  /// Idempotent : safe à appeler à chaque entrée dans une room.
  pub(super) fn ensure_chat_started(&mut self, room_id: String) -> Task<Message>;

  // --- Création / jointure ---

  pub(super) fn on_chat_room_create(&mut self) -> Task<Message>;
  pub(super) fn on_chat_room_created(&mut self, r: Result<Room, String>) -> Task<Message>;
  pub(super) fn on_chat_room_join(&mut self) -> Task<Message>;
  pub(super) fn on_chat_room_joined(&mut self, r: Result<Room, String>) -> Task<Message>;
  pub(super) fn on_chat_room_leave(&mut self, room_id: String) -> Task<Message>;
  pub(super) fn on_chat_room_left(&mut self, r: Result<String, String>) -> Task<Message>;
  pub(super) fn on_chat_room_selected(&mut self, room_id: String) -> Task<Message>;

  // --- Envoi / réception ---

  pub(super) fn on_chat_send(&mut self) -> Task<Message>;
  pub(super) fn on_chat_sent(&mut self, r: Result<ChatMessage, String>) -> Task<Message>;
  pub(super) fn on_chat_received(&mut self, room_id: String, msg: ChatMessage) -> Task<Message>;
  pub(super) fn on_chat_decrypt_failed(&mut self, room_id: String, reason: String) -> Task<Message>;

  // --- Join code ---

  pub(super) fn on_chat_join_code_copy(&mut self, room_id: String) -> Task<Message>;
  pub(super) fn on_chat_join_code_copied(&mut self, r: Result<String, String>) -> Task<Message>;

  // --- MQTT infra ---

  pub(super) fn on_mqtt_event(&mut self, event: MqttEvent) -> Task<Message>;
  pub(super) fn on_mqtt_crypto_loaded(&mut self, r: Result<Arc<ChatCryptoCtx>, String>) -> Task<Message>;

  // --- Présence ---

  pub(super) fn on_presence_updated(&mut self, update: crate::chat::PresenceUpdate) -> Task<Message>;

  // --- ACK ---

  pub(super) fn on_chat_ack_received(&mut self, room_id: String, msg_id: String, sender_fp: String) -> Task<Message>;
  pub(super) fn on_chat_ack_sent(&mut self, r: Result<(), String>) -> Task<Message>;

  // --- Helpers privés ---

  /// Insère un message en RAM, applique le bornage à 200 messages/room.
  fn push_chat_message(&mut self, room_id: &str, msg: ChatMessage);

  /// Cherche une room par id (helper équivalent à key_by_fp).
  fn room_by_id(&self, id: &str) -> Option<&Room>;
}
```

### 5.3 Logique clé : `on_mqtt_event`

Centralise la dispatch des évènements MQTT. C'est le point d'entrée applicatif
de tout ce qui vient du broker.

```rust
pub(super) fn on_mqtt_event(&mut self, event: MqttEvent) -> Task<Message> {
  match event {
    MqttEvent::Connected => {
      self.mqtt_state = MqttState::Connected;
      // Subscribe à tous les topics de toutes les rooms + présence.
      self.subscribe_all_known_topics()
    }
    MqttEvent::Disconnected(reason) => {
      self.mqtt_state = MqttState::Disconnected;
      self.set_status(StatusKind::Error, format!("MQTT déconnecté : {reason}"))
    }
    MqttEvent::Reconnecting { attempt } => {
      self.mqtt_state = MqttState::Reconnecting { attempt };
      Task::none()
    }
    MqttEvent::Message { topic, payload, retained: _ } => {
      // Dispatch selon le préfixe topic : chat / presence / ack
      self.dispatch_mqtt_payload(topic, payload)
    }
    MqttEvent::PublishAck { .. } => Task::none(),
    MqttEvent::Error(e) => self.set_status(StatusKind::Error, format!("MQTT : {e}")),
  }
}
```

`dispatch_mqtt_payload` lance un `blocking_task` pour décoder + déchiffrer
(opération CPU-bound via sequoia) et retourne `ChatReceived` ou
`PresenceUpdated`.

### 5.4 Réutilisation de `key_list.rs` pour `ChatList`

Le master-detail (liste à 320px à gauche, détail à droite) est exactement le
même pattern. **Recommandation** : créer `src/ui/chat_list.rs` qui copie le
squelette mais ne factorise pas avec `key_list.rs` à ce stade — les types
(`Room` vs `KeyInfo`) et messages diffèrent assez pour qu'une abstraction
prématurée nuise. Refacto possible en v0.7+.

---

## 6. Réutilisation du code existant

### 6.1 `blocking_task()` pour encrypt/decrypt

Oui, **utiliser `blocking_task` partout pour la crypto sequoia**, même
in-process. Justification : sequoia parse/chiffre en ~10–50 ms ; pour un
message c'est tolérable, mais `Task::perform` exige que le futur soit Send +
'static. Encapsuler dans `blocking_task` permet aussi de partager le pattern
de gestion d'erreurs `Result<T, String>`.

Exemple :

```rust
let crypto = self.chat_crypto.clone().expect("ensure_chat_started garantit le chargement");
let recipients: Vec<String> = room.participants.clone();
let plaintext = std::mem::take(&mut self.chat_input);
let room_id = room.id.clone();

Task::perform(
  blocking_task(move || {
    let payload = crypto.encrypt_for_room(&plaintext, &recipients)?;
    let wire = build_wire_message(&payload, &crypto.local_fp, &plaintext);
    Ok(wire)
  }),
  |r| match r {
    Ok(wire) => Message::ChatSent(Ok(wire_to_chat_message(wire))),
    Err(e) => Message::ChatSent(Err(e)),
  },
)
```

### 6.2 Étendre `Config` pour `mqtt_relay`

**Recommandation** : oui, ajouter un champ `mqtt_default_relay: Option<String>`
au `Config`. Permet à l'utilisateur de définir un relais par défaut pour les
nouvelles rooms (sinon, demandé à chaque création).

```rust
pub struct Config {
  pub language: Language,
  pub scale_factor: f64,
  pub theme: ThemeVariant,
  // --- v0.6.0 ---
  #[serde(default)]
  pub mqtt_default_relay: Option<String>,
  /// Local fingerprint utilisé pour le chat (pour le client_id et la
  /// signature). Si None → première clé secrète disponible.
  #[serde(default)]
  pub chat_local_fp: Option<String>,
}
```

Le pattern `serde(default)` garantit la rétro-compatibilité avec les configs
v0.5.x existantes.

### 6.3 `gnupg_dir()` réutilisable dans `chat/crypto.rs`

Oui, **directement**. Déjà `pub(crate)` dans `gpg/mod.rs`. Permet à
`ChatCryptoCtx::load` de localiser le keyring sans dupliquer la logique.

```rust
// dans chat/crypto.rs
let homedir = crate::gpg::gnupg_dir().map_err(|e| ChatError::InvalidConfig(e.to_string()))?;
```

Idem pour `gpg_command` : on l'utilise une seule fois au load
(`gpg --export-secret-keys --armor <fp>`) avant de basculer vers sequoia.

### 6.4 Sequoia déjà disponible

Oui — `sequoia-openpgp = "2"` est déjà dans Cargo.toml. Les modules à
importer :

```rust
use sequoia_openpgp::{
  Cert,
  parse::{Parse, stream::*},
  serialize::stream::*,
  policy::{StandardPolicy, NullPolicy},
  cert::amalgamation::ValidAmalgamation,
};
```

À noter : `NullPolicy::new()` est `unsafe` en sequoia 2 (déjà commenté dans
CLAUDE.md) — on utilise `StandardPolicy` pour le chat (les clés sont
fraîchement créées et conformes).

---

## 7. Risques architecturaux

### 7.1 Intégration MQTT avec iced — Option A vs B

**Option A : `App::subscription` retourne un Stream MQTT**
- L'`MqttEvent` stream est exposé via `iced::Subscription::run` (qui accepte
  un Stream).
- Iced gère naturellement le branchement → chaque évènement devient un
  `Message::MqttEvent`.

**Option B : channel tokio + Task::perform en boucle**
- Une `tokio::sync::mpsc` est consommée par un `Task::perform` qui re-poste
  un nouveau Task à chaque message reçu.
- Plus complexe, demande de gérer la re-souscription manuellement.

**Recommandation : Option A.** Iced 0.14 a précisément été conçu pour
intégrer des streams externes via `Subscription::run_with_id` (pattern
identique à un WebSocket dans une app iced). C'est l'API idiomatique.

Squelette :

```rust
pub fn subscription(&self) -> iced::Subscription<Message> {
  let file_drop = iced::event::listen_with(|event, _, _| match event {
    iced::Event::Window(iced::window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
    _ => None,
  });

  let mut subs = vec![file_drop];

  if let Some(handle) = &self.mqtt {
    // Le stream a été stocké dans un `Mutex<Option<Stream>>` au spawn,
    // récupéré ici via une factory `mqtt::take_stream()` ou un Subscription
    // basé sur l'ID du handle.
    subs.push(crate::chat::mqtt::subscription(handle.clone()));
  }

  iced::Subscription::batch(subs)
}
```

**Précision technique** : `Subscription::run_with_id` exige une closure
`Fn() -> Stream`. Le Stream MQTT est créé une seule fois au spawn, donc on le
stocke dans un `Arc<Mutex<Option<Stream>>>` interne au handle, et la closure
le `take()` au premier appel. Iced ne ré-appelle pas la factory tant que
l'`id` ne change pas.

### 7.2 Reconnexion automatique sans bloquer l'UI

`rumqttc` gère déjà la reconnexion en interne (boucle `event_loop.poll()`
qui retry sur erreur). On n'a **rien à faire dans l'UI thread** : la tâche
tokio dédiée tourne indépendamment. L'UI reçoit `Reconnecting { attempt }`
et `Connected` purement informatifs.

**Bornage du backoff** : `rumqttc` n'a pas de cap natif robuste — on impose
manuellement min 1 s, max 30 s, exponentiel ×2. Implémenté dans la tâche
spawn de `mqtt::spawn`.

**Risque** : reconnexion infinie si le broker est down → consommation CPU
négligeable mais bruit potentiel sur les status messages. **Mitigation** :
n'émettre `MqttEvent::Reconnecting` qu'une fois toutes les 5 tentatives, ou
silencieusement après les 3 premières.

### 7.3 Bornage de la HashMap messages en RAM

`HashMap<RoomId, VecDeque<ChatMessage>>` avec **N = 200 messages par room**.
Justification : 200 messages × 200 octets moyens × 50 rooms = 2 MB en RAM →
acceptable. L'application est un client GUI, pas un serveur.

Implémentation dans `push_chat_message` :

```rust
fn push_chat_message(&mut self, room_id: &str, msg: ChatMessage) {
  let queue = self.chat_messages.entry(room_id.to_string()).or_insert_with(VecDeque::new);
  if queue.len() >= 200 {
    queue.pop_front();
  }
  queue.push_back(msg);
}
```

**Conformité éphémère** : aucune sérialisation sur disque. Au quit de
l'application, tout disparaît. Si l'utilisateur veut conserver, il faudra un
export explicite (hors scope v0.6.0).

### 7.4 Thread-safety du client MQTT avec iced/tokio

**Pattern choisi : commande via `mpsc::Sender`**. Le `MqttHandle` ne contient
**aucune référence directe** à `rumqttc::AsyncClient`. Toutes les opérations
passent par un canal. Conséquences :
- `MqttHandle` est trivialement `Clone + Send + Sync` (un Sender l'est).
- L'unique propriétaire du `AsyncClient` et du `EventLoop` est la tâche
  tokio, qui ne quitte qu'au `Shutdown`.
- Iced peut clôner `MqttHandle` autant qu'il veut dans des `Task::perform`
  sans risque de data race.

**Risque alternatif rejeté** : exposer `Arc<Mutex<AsyncClient>>` dans `App`.
Crée des deadlocks potentiels (l'eventloop bloque sur le mutex pendant qu'un
`Task::perform` essaie de publier). Le pattern channel l'élimine par
construction.

### 7.5 Risques transverses non bloquants

- **Passphrase pinentry au load** : géré par `gpg --export-secret-keys` (cf.
  §1.4). UX acceptable une fois par session.
- **Clé secrète en RAM** : `ChatCryptoCtx` contient la `Cert` complète. Elle
  vit tant que l'app tourne. **Pas de zeroize** en v0.6.0 → à documenter
  dans l'axe 7 sécurité comme limite assumée.
- **Topic enumeration** : SHA256(room_id)[0..16] hex offre 64 bits de
  collision-resistance ; un attaquant qui scanne les topics ne peut pas
  identifier les rooms. Validation détaillée en T2.2 / axe 7.
- **Capacité du stream MQTT vers iced** : si l'UI freeze, un `mpsc::Sender`
  illimité accumule la mémoire. **Recommandation** : `mpsc::channel(256)`
  borné, drop des évènements anciens si saturé (avec log error). Détails
  d'impl en axe 3.

---

## 8. Synthèse des décisions

| Sujet | Décision | Raison principale |
|---|---|---|
| Layout modules | 5 fichiers (`mod`, `rooms`, `mqtt`, `crypto`, `presence`) | Séparation claire des responsabilités, pas de circular dep |
| Type d'erreur | `ChatError` enum via `thiserror` | Cohérent avec les `Result<T, String>` existants tout en restant typé en interne |
| Champ `App.rooms` | `Vec<Room>` (pas `RoomStore`) | Accès direct, store reste outil de sérialisation |
| Champ `App.chat_messages` | `HashMap<RoomId, VecDeque<ChatMessage>>` borné à 200 | RAM modeste, conforme éphémère, bornage O(1) |
| Champ `App.mqtt` | `Option<MqttHandle>` cloneable | Thread-safe via mpsc::Sender interne |
| Champ `App.chat_crypto` | `Option<Arc<ChatCryptoCtx>>` | Évite cloner la Cert, partageable thread |
| Connexion MQTT | Démarrée à la 1ère ouverture de room | Évite connexion inutile au lancement |
| `View::ChatRoom(String)` | Variant porte le `room_id` | Re-render naturel d'iced à chaque changement de room |
| `previous_view` | Étendu à `ChatRoom(_)` | "Retour" depuis room → ChatList |
| Découpage handlers | Un seul `app/chat.rs` | Pas de fragmentation prématurée |
| Crypto chat | sequoia in-process (pas subprocess gpg) | Latence acceptable, pas de pinentry par message |
| Réutilisation `Config` | Ajout `mqtt_default_relay`, `chat_local_fp` | `serde(default)` garantit la rétro-compat |
| Intégration iced ↔ MQTT | `Subscription::run` autour d'un Stream | API idiomatique iced 0.14 |
| Thread-safety MQTT | `mpsc::Sender` dans `MqttHandle`, jamais d'`Arc<Mutex<Client>>` | Élimine risques de deadlock |
| YubiKey | Refus en v0.6.0 (pas de clé secrète exportable) | Le chat n'est dispo qu'avec clé locale |

---

## 9. Hors scope v0.6.0 (notes pour v0.7+)

- **Persistance optionnelle** des messages chat (chiffrée localement).
- **Drafts par room** (HashMap au lieu de String unique).
- **Multi-device** (synchronisation des messages entre 2 instances pgpilot
  d'un même utilisateur).
- **Pièces jointes** (fichiers chiffrés transmis via MQTT ou hors-bande).
- **YubiKey support** (nécessite signature in-card via `gpg --sign` sans
  export → architecture différente).
- **Refacto master-detail** entre `key_list` et `chat_list` si patterns
  divergent peu.

---

## 10. Prochaines étapes

Cette spec alimente :
- **T2.2** (axe 2 — design API) : doit confirmer/raffiner les types
  `WireMessage`, `JoinCode`, `ChatMessage` détaillés ici de façon synthétique.
- **T2.3** (axe 2 — fusion) : produit `axe2-spec-finale.md` qui répond aux
  4 questions ouvertes (intégration iced, vie des clés, broker injoignable,
  tests sans broker).
- **Axe 3** : implémentation `mqtt.rs` + ajout dépendances Cargo.
- **Axe 4** : implémentation `rooms.rs` + `crypto.rs` + `presence.rs` + tous
  les handlers `app/chat.rs`.
- **Axe 5** : UI `ui/chat_list.rs` + `ui/chat_room.rs` + intégration sidebar.
