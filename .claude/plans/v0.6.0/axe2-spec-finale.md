# Axe 2 — Spec finale chat (T2.3)

> Document de référence unique pour l'implémentation des axes 3 à 8. Fusionne et arbitre `axe2-spec-modules.md` (T2.1, architecture) et `axe2-spec-api.md` (T2.2, contrats de données). Toute contradiction entre ces deux sources est tranchée ici, avec la décision retenue documentée explicitement.
>
> Aucun code n'est écrit dans ce document. Les axes 3–8 implémenteront ce qui est spécifié ici.

---

## 1. Vue d'ensemble et périmètre

### 1.1 Objectif fonctionnel v0.6.0

Permettre à plusieurs utilisateurs PGPilot d'échanger des messages texte chiffrés et signés en temps quasi-réel, via un broker MQTT public ou privé, **sans persistance des messages** (éphémère par conception).

### 1.2 Modèle de menace résumé

- **Confidentialité** : assurée par PGP (chiffrement multi-destinataires via clefs publiques). Le broker ne voit que des blobs PGP armored.
- **Authenticité** : signature PGP détachée par message, vérifiée à la réception ; messages non vérifiables ignorés silencieusement.
- **Métadonnées** : minimisées (topics dérivés via SHA256, pas de liste de destinataires sur le wire, fingerprints tronqués à 16 hex dans les topics). Le broker peut toujours observer `qui publie sur quel topic`, ce qui révèle la cardinalité d'un salon.
- **Disponibilité** : best-effort. Pas de file d'attente persistante : un message manqué pendant une déconnexion est perdu.
- **Forward secrecy** : aucune en v0.6.0. Une clef privée compromise plus tard permet de déchiffrer les messages capturés (mais comme rien n'est persisté côté broker, la fenêtre est limitée à ce qu'un attaquant a pu enregistrer en temps réel).

### 1.3 Ce qui est inclus en v0.6.0

- Création / jointure de salons via join code signé.
- Salons multi-participants (N participants, chaque message chiffré pour les N).
- Présence Online/Offline (LWT MQTT + heartbeat).
- ACK applicatif de réception (non signé, best-effort).
- Persistance YAML des salons uniquement (pas des messages).
- Reconnexion automatique avec backoff borné.
- UI master-detail (liste de salons à gauche, conversation à droite) intégrée dans la sidebar PGPilot existante.

### 1.4 Ce qui est explicitement hors scope (cf. §12)

YubiKey, multi-device, pièces jointes, persistance chiffrée, drafts par room, forward secrecy, modération.

---

## 2. Layout des modules `src/chat/`

```
src/
├── chat/
│   ├── mod.rs        — re-exports publics, types partagés (ChatError, RoomId, Fingerprint), constantes
│   ├── rooms.rs      — Room, RoomParticipant, RoomStore (load/save rooms.yaml), JoinCode encode/decode/verify
│   ├── mqtt.rs       — MqttHandle, MqttEvent, MqttConfig, ChatTransport trait, spawn(), reconnect loop
│   ├── crypto.rs     — ChatCryptoCtx, ChatPayload, VerifiedMessage, encrypt_for_room, decrypt_message
│   ├── presence.rs   — PresenceStatus, PresenceUpdate, PresenceTracker, build_lwt, presence_topic
│   └── wire.rs       — WireMessage, WireAck, sérialisation/désérialisation, validation de taille
```

> **Précision vs T2.1** : T2.1 prévoyait 5 fichiers et plaçait le wire format implicitement dans `crypto.rs`. T2.2 a formalisé `WireMessage`/`WireAck` comme types séparés avec validation de taille et règles de canonicalisation. **Décision : ajouter un 6e fichier `wire.rs`** pour isoler les contrats de sérialisation (ils n'appartiennent ni à crypto, ni à mqtt — ils sont la frontière entre les deux).

### 2.1 Responsabilités par fichier

| Fichier | Responsabilité | Dépendances internes |
|---|---|---|
| `chat/mod.rs` | Façade publique, `ChatError`, alias de types, constantes globales | aucune |
| `chat/rooms.rs` | Persistance YAML, CRUD des salons, encode/decode/vérification du `JoinCode` | `mod`, `crypto` (pour signer/vérifier les join codes), `gpg::validate_fp` |
| `chat/mqtt.rs` | Connexion MQTT, eventloop, reconnect, canal de commandes, exposition d'un Stream pour iced | `mod`, `tokio`, `rumqttc` |
| `chat/crypto.rs` | Chargement des clefs (sequoia in-process), chiffrement multi-destinataires, signature, déchiffrement, vérification | `mod`, `gpg::gnupg_dir`, `gpg::gpg_command`, `sequoia_openpgp` |
| `chat/presence.rs` | Tracking présence en RAM, encodage/décodage des payloads présence, génération du LWT | `mod`, `chat::mqtt::LastWill` |
| `chat/wire.rs` | `WireMessage`, `WireAck`, sérialisation JSON, canonicalisation pour signature, validation taille | `mod`, `serde`, `serde_json` |

### 2.2 Diagramme de dépendances internes

```
                     ┌─────────────┐
                     │  chat/mod   │  types partagés, constantes
                     └─┬──┬──┬──┬──┘
                       │  │  │  │
        ┌──────────────┘  │  │  └────────────────────┐
        │                 │  │                       │
   ┌────▼────┐    ┌───────▼──▼─┐    ┌───────────┐  ┌─▼──────┐
   │  rooms  │───►│   crypto   │    │  presence │  │  wire  │
   └─────────┘    └────────────┘    └───────────┘  └────────┘
                       │                  │             │
                       │             (LastWill type     │
                       │              vit dans mqtt)    │
                       ▼                  ▼             │
                 (gpg::gnupg_dir,   ┌──────────┐        │
                  gpg::gpg_command, │   mqtt   │◄───────┘
                  sequoia)          └──────────┘  (publish bytes JSON)
```

`rooms` dépend de `crypto` uniquement pour signer/vérifier les join codes (réutilise `ChatCryptoCtx`). Pas de cycle. L'orchestration (qui appelle quoi quand) vit exclusivement dans `app/chat.rs`.

### 2.3 Dépendances Cargo à ajouter (livrées par axe 3)

```toml
rumqttc        = "0.24"                              # client MQTT async tokio
uuid           = { version = "1", features = ["v4", "serde"] }
base64         = "0.22"                              # join codes (URL-safe, no padding)
thiserror      = "1"                                 # ChatError
serde_json     = "1"                                 # wire format
async-trait    = "0.1"                               # ChatTransport trait
futures        = "0.3"                               # Stream pour Subscription::run
```

`sequoia-openpgp = "2"` est déjà présent. `tokio`, `serde`, `serde_yaml`, `chrono`, `dirs` aussi.

---

## 3. Champs ajoutés à `App` (avec types exacts)

```rust
pub struct App {
  // … champs existants v0.5.x conservés inchangés …

  // --- Chat (v0.6.0) ---

  /// Salons persistés, chargés au démarrage depuis ~/.config/pgpilot/rooms.yaml.
  pub rooms: Vec<crate::chat::Room>,

  /// Salon actif (room_id UUID). None = vue ChatList ou hors section chat.
  pub active_room: Option<String>,

  /// Messages en RAM par room_id. JAMAIS persistés. Borné à 500 messages/room (FIFO).
  pub chat_messages: std::collections::HashMap<String, std::collections::VecDeque<crate::chat::ChatMessage>>,

  /// Tracker de présence agrégé pour tous les fingerprints connus.
  pub presence: crate::chat::PresenceTracker,

  /// État de connexion MQTT.
  pub mqtt_state: MqttState,

  /// Handle vers le client MQTT (None tant que pas démarré). Cloneable, thread-safe.
  pub mqtt: Option<crate::chat::MqttHandle>,

  /// Saisie courante dans la room active. Vidé à chaque changement de room.
  pub chat_input: String,

  /// Formulaire dédié pour création/jointure (séparé de `chat_input` pour ne pas
  /// écraser un draft en cours quand l'utilisateur ouvre un dialogue de création).
  pub chat_new_form: ChatNewForm,

  /// Contexte crypto (Cert local + peers), chargé une fois par session, partagé via Arc.
  /// None tant que l'utilisateur n'a pas ouvert son premier salon.
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

#[derive(Debug, Clone, Default)]
pub struct ChatNewForm {
  pub name: String,           // nom local du salon en cours de création
  pub relay: String,          // pré-rempli depuis Config.mqtt_default_relay
  pub participants_input: String, // textarea brut, parsé en \n
  pub join_code: String,      // code en cours de saisie pour rejoindre
}
```

### 3.1 Justification des choix de typage

- **`rooms: Vec<Room>` plutôt que `RoomStore`** — accès direct sans pattern-match. `RoomStore` reste un outil de sérialisation interne à `chat/rooms.rs`.
- **`chat_messages: HashMap<String, VecDeque<…>>`** — `VecDeque` permet `pop_front` O(1) pour le bornage FIFO. T2.1 et T2.2 étaient en désaccord (T2.1 → VecDeque, T2.2 → Vec). **Décision retenue : `VecDeque`** car le bornage doit être O(1) sur le chemin chaud (réception de messages).
- **`mqtt_state: MqttState` (enum, pas bool)** — calque le pattern `KeyserverStatus` existant ; permet d'afficher "Reconnecting (attempt N)" et "Failed(reason)" dans la status bar.
- **`mqtt: Option<MqttHandle>`** — `Option` car la connexion est paresseuse (cf. §10). `MqttHandle` ne contient qu'un `mpsc::Sender`, donc `Clone + Send + Sync` sans coût.
- **`chat_crypto: Option<Arc<ChatCryptoCtx>>`** — `Arc` évite de cloner les `Cert` sequoia (volumineux) à chaque envoi de message. `Option` car le chargement est lazy et nécessite un `blocking_task` (export GPG bloquant).
- **`chat_new_form` séparé de `chat_input`** — le formulaire de création/jointure ne doit pas piétiner le draft de message en cours.

### 3.2 Bornage messages : 500/room

T2.1 proposait 200, T2.2 proposait 500. **Décision retenue : 500** (cf. §6 constante `MAX_MESSAGES_PER_ROOM`). Justification : 500 messages × ~250 octets moyens × 50 rooms ≈ 6 Mio de RAM dans le pire cas — acceptable pour un client GUI. 500 messages couvre 2–4 h de conversation active.

### 3.3 Initialisation dans `App::new`

```rust
pub fn new() -> (Self, Task<Message>) {
  let config = Config::load().unwrap_or_default();
  // … existant …
  let rooms = crate::chat::RoomStore::load()
    .map(|s| s.rooms)
    .unwrap_or_default();

  let initial_keys_task = Task::perform(blocking_task(crate::gpg::list_keys), Message::KeysLoaded);

  (Self {
    // … existant …
    rooms,
    active_room: None,
    chat_messages: HashMap::new(),
    presence: crate::chat::PresenceTracker::new(),
    mqtt_state: MqttState::Disconnected,
    mqtt: None,
    chat_input: String::new(),
    chat_new_form: ChatNewForm {
      relay: config.mqtt_default_relay.clone().unwrap_or_default(),
      ..Default::default()
    },
    chat_crypto: None,
  }, initial_keys_task)
}
```

**Important** : aucune connexion MQTT au lancement. Démarrage paresseux à la première ouverture de salon (cf. §10).

### 3.4 Extension de `Config`

```rust
pub struct Config {
  // … champs v0.5.x …
  #[serde(default)]
  pub mqtt_default_relay: Option<String>,
  #[serde(default)]
  pub chat_local_fp: Option<String>, // None = première clef secrète disponible
}
```

`#[serde(default)]` garantit la rétro-compatibilité avec `~/.config/pgpilot/config.yaml` v0.5.x.

---

## 4. Variants `View` et `Message` (liste exhaustive)

### 4.1 `View`

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
  // --- v0.6.0 ---
  ChatList,
  ChatRoom(String),         // room_id UUID
  ChatNewRoom,              // formulaire création
  ChatJoinRoom,             // formulaire jointure (saisie code)
}
```

> **Précision vs T2.1** : T2.1 listait seulement `ChatList` et `ChatRoom(String)`. **Ajout** : `ChatNewRoom` et `ChatJoinRoom` comme vues plein-écran dédiées (suivent le pattern `CreateKey` / `Import`). Justification : créer un salon nécessite plusieurs champs (nom, relay, liste de participants à coller) qui méritent leur propre vue plutôt qu'une modale.

#### 4.1.1 Cohérence avec `previous_view`

Étendre la liste `on_nav_changed` :

```rust
if matches!(view, View::CreateKey | View::Import | View::ChatRoom(_) | View::ChatNewRoom | View::ChatJoinRoom) {
  self.previous_view = Some(self.view.clone());
}
```

Bouton "Retour" depuis :
- `ChatRoom(_)` → `ChatList`
- `ChatNewRoom` / `ChatJoinRoom` → `ChatList`

#### 4.1.2 Logique additionnelle dans `on_nav_changed`

```rust
match &view {
  View::ChatRoom(room_id) => {
    self.active_room = Some(room_id.clone());
    self.chat_input.clear();
    return self.ensure_chat_started(room_id.clone());
  }
  View::ChatList | View::ChatNewRoom | View::ChatJoinRoom => {
    self.active_room = None;
  }
  _ => {
    self.active_room = None;
  }
}
```

### 4.2 `Message` — liste exhaustive

```rust
#[derive(Debug, Clone)]
pub enum Message {
  // … variants v0.5.x conservés …

  // --- Chat : navigation / création / jointure ---
  ChatRoomCreate,                                    // submit du formulaire création
  ChatRoomNameChanged(String),
  ChatRoomRelayChanged(String),
  ChatRoomParticipantsChanged(String),               // textarea brut
  ChatRoomCreated(Result<crate::chat::Room, String>),

  ChatJoinCodeChanged(String),
  ChatRoomJoin,                                      // submit "Rejoindre"
  ChatRoomJoined(Result<crate::chat::Room, String>),

  ChatRoomSelected(String),                          // room_id (clic dans liste)
  ChatRoomLeave(String),
  ChatRoomLeft(Result<String, String>),              // room_id

  // --- Chat : envoi / réception ---
  ChatInputChanged(String),
  ChatSend,                                          // bouton "Envoyer" / Enter
  ChatSent(Result<crate::chat::ChatMessage, String>),
  ChatReceived(String, crate::chat::ChatMessage),    // room_id, message déchiffré

  // --- Chat : partage du join code ---
  ChatJoinCodeCopy(String),                          // room_id → encode + clipboard
  ChatJoinCodeCopied(Result<String, String>),

  // --- MQTT infra ---
  MqttEvent(crate::chat::MqttEvent),                 // évènement venant du Stream
  MqttCryptoLoaded(Result<std::sync::Arc<crate::chat::ChatCryptoCtx>, String>),

  // --- Présence ---
  PresenceUpdated(crate::chat::PresenceUpdate),

  // --- ACK applicatif ---
  ChatAckReceived(String, String, String),           // room_id, msg_id, sender_fp
  ChatAckSent(Result<(), String>),                   // confirmation publication ACK
}
```

> **Précision vs T2.1** : T2.1 incluait `ChatDecryptFailed(String, String)`. **Suppression** : conformément à la décision T2.2 §5.2, les messages non déchiffrables sont ignorés silencieusement (pas de placeholder UI). On logue dans le tracing interne mais on ne propage pas de message UI. Le variant n'est donc pas nécessaire.

> **Précision vs ébauche initiale** : `ChatRoomCreate` ne porte pas le nom — celui-ci vit dans `chat_new_form.name`. Pattern identique à `CreateKeySubmit`.

### 4.3 Routing dans `update()`

```rust
Message::ChatRoomCreate => self.on_chat_room_create(),
Message::ChatRoomCreated(r) => self.on_chat_room_created(r),
Message::ChatRoomJoin => self.on_chat_room_join(),
Message::ChatRoomJoined(r) => self.on_chat_room_joined(r),
Message::ChatRoomSelected(id) => self.on_chat_room_selected(id),
Message::ChatRoomLeave(id) => self.on_chat_room_leave(id),
Message::ChatRoomLeft(r) => self.on_chat_room_left(r),
Message::ChatSend => self.on_chat_send(),
Message::ChatSent(r) => self.on_chat_sent(r),
Message::ChatReceived(id, m) => self.on_chat_received(id, m),
Message::ChatJoinCodeCopy(id) => self.on_chat_join_code_copy(id),
Message::ChatJoinCodeCopied(r) => self.on_chat_join_code_copied(r),
Message::MqttEvent(e) => self.on_mqtt_event(e),
Message::MqttCryptoLoaded(r) => self.on_mqtt_crypto_loaded(r),
Message::PresenceUpdated(u) => self.on_presence_updated(u),
Message::ChatAckReceived(rid, mid, sfp) => self.on_chat_ack_received(rid, mid, sfp),
Message::ChatAckSent(r) => self.on_chat_ack_sent(r),

// Trivial — inline
Message::ChatInputChanged(v) => { self.chat_input = v; Task::none() }
Message::ChatRoomNameChanged(v) => { self.chat_new_form.name = v; Task::none() }
Message::ChatRoomRelayChanged(v) => { self.chat_new_form.relay = v; Task::none() }
Message::ChatRoomParticipantsChanged(v) => { self.chat_new_form.participants_input = v; Task::none() }
Message::ChatJoinCodeChanged(v) => { self.chat_new_form.join_code = v; Task::none() }
```

---

## 5. Handlers dans `src/app/chat.rs` (signatures)

Un seul module `app/chat.rs`. Pas de fragmentation prématurée.

```rust
use iced::Task;
use std::sync::Arc;

use crate::chat::{ChatCryptoCtx, ChatMessage, MqttEvent, MqttHandle, PresenceUpdate, Room};
use super::{blocking_task, App, Message, MqttState, StatusKind, View};

impl App {
  // --- Cycle de vie chat ---

  /// Idempotent. Démarre la connexion MQTT si absente, charge le crypto ctx
  /// si absent, et abonne le client à tous les topics nécessaires (chat de la
  /// room + présence des participants + ack).
  pub(super) fn ensure_chat_started(&mut self, room_id: String) -> Task<Message>;

  /// Helper — abonne aux topics de toutes les rooms connues + présence des
  /// participants. Appelé au reconnect.
  fn subscribe_all_known_topics(&self) -> Task<Message>;

  // --- Création / jointure ---

  pub(super) fn on_chat_room_create(&mut self) -> Task<Message>;
  pub(super) fn on_chat_room_created(&mut self, r: Result<Room, String>) -> Task<Message>;
  pub(super) fn on_chat_room_join(&mut self) -> Task<Message>;
  pub(super) fn on_chat_room_joined(&mut self, r: Result<Room, String>) -> Task<Message>;
  pub(super) fn on_chat_room_selected(&mut self, room_id: String) -> Task<Message>;
  pub(super) fn on_chat_room_leave(&mut self, room_id: String) -> Task<Message>;
  pub(super) fn on_chat_room_left(&mut self, r: Result<String, String>) -> Task<Message>;

  // --- Envoi / réception ---

  pub(super) fn on_chat_send(&mut self) -> Task<Message>;
  pub(super) fn on_chat_sent(&mut self, r: Result<ChatMessage, String>) -> Task<Message>;
  pub(super) fn on_chat_received(&mut self, room_id: String, msg: ChatMessage) -> Task<Message>;

  // --- Join code ---

  pub(super) fn on_chat_join_code_copy(&mut self, room_id: String) -> Task<Message>;
  pub(super) fn on_chat_join_code_copied(&mut self, r: Result<String, String>) -> Task<Message>;

  // --- MQTT infra ---

  pub(super) fn on_mqtt_event(&mut self, event: MqttEvent) -> Task<Message>;
  pub(super) fn on_mqtt_crypto_loaded(&mut self, r: Result<Arc<ChatCryptoCtx>, String>) -> Task<Message>;

  /// Routage interne : déchiffre selon le préfixe topic (chat / presence / ack)
  /// et émet le `Message` applicatif correspondant via Task::perform.
  fn dispatch_mqtt_payload(&self, topic: String, payload: Vec<u8>) -> Task<Message>;

  // --- Présence ---

  pub(super) fn on_presence_updated(&mut self, update: PresenceUpdate) -> Task<Message>;

  // --- ACK ---

  pub(super) fn on_chat_ack_received(&mut self, room_id: String, msg_id: String, sender_fp: String) -> Task<Message>;
  pub(super) fn on_chat_ack_sent(&mut self, r: Result<(), String>) -> Task<Message>;

  // --- Helpers privés ---

  /// Insère un message en RAM, applique le bornage à MAX_MESSAGES_PER_ROOM (FIFO).
  fn push_chat_message(&mut self, room_id: &str, msg: ChatMessage);

  /// Cherche une room par id (équivalent de key_by_fp).
  fn room_by_id(&self, id: &str) -> Option<&Room>;
}
```

### 5.1 `dispatch_mqtt_payload` — routage par préfixe

```text
pgpilot/chat/{hash}      → blocking_task(decrypt) → Message::ChatReceived(room_id, msg)
                                                  + publication d'un ACK (best-effort)
pgpilot/presence/{fp16}  → decode_payload         → Message::PresenceUpdated(update)
pgpilot/ack/{msg_id16}   → parse JSON WireAck     → Message::ChatAckReceived(room_id, msg_id, from)
```

Le routage discrimine sur le préfixe en début de topic. Le hash dans le topic est résolu en `room_id` via une table `topic_to_room: HashMap<String, RoomId>` maintenue par `on_chat_room_created`/`on_chat_room_joined`. Si le topic est inconnu (race au reconnect), on logue et on jette.

---

## 6. Formats de données

### 6.1 `WireMessage` (sur le wire MQTT)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMessage {
  pub id: String,        // UUID v4
  pub sender: String,    // fingerprint 40 hex
  pub ts: i64,           // Unix seconds UTC
  pub payload: String,   // "-----BEGIN PGP MESSAGE-----\n..." (multi-recipients)
  pub signature: String, // "-----BEGIN PGP SIGNATURE-----\n..." (detached)
}
```

**Canonicalisation pour signature** :

```
SIGN_CANONICAL_PREFIX || id || \x00 || sender || \x00 || ts_decimal || \x00 || payload
```

où `SIGN_CANONICAL_PREFIX = b"pgpilot-msg\x00"`. La signature couvre `id + sender + ts + payload` (pas seulement `payload`) pour empêcher la substitution d'émetteur ou d'horodatage.

**Contraintes** :
- Pas de champ `recipients_fps` sur le wire (fuite métadonnée). Les destinataires sont implicites dans les session keys PGP.
- Taille sérialisée JSON ≤ `MAX_WIRE_MESSAGE_BYTES` (64 Kio). Validation côté émetteur ET récepteur. Dépassement → `ChatError::MessageTooLarge`.
- Le broker doit accepter `max_packet_size = 131_072` (128 Kio) pour absorber le framing MQTT.

### 6.2 `WireAck` (sur le wire MQTT)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireAck {
  pub msg_id: String,  // UUID du WireMessage acquitté
  pub from: String,    // fingerprint 40 hex du confirmant
  pub ts: i64,         // Unix seconds UTC
}
```

**ACK non signé** : décision T2.2 §6.1 retenue. Justification finale (corrigée vs T2.2) : même avec sequoia in-process, la signature ajoute de la latence et de la complexité pour un gain sécurité nul (un ACK forgé n'a aucun impact sur la confidentialité ou l'authenticité des messages, juste sur la fiabilité d'affichage de l'état "lu par X"). Le `msg_id` UUID v4 (122 bits d'entropie) rend les ACK forgés aléatoirement statistiquement impossibles.

### 6.3 `JoinCode` (texte partageable hors-bande)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinCode {
  pub room_id: String,           // UUID v4
  pub relay: String,             // "mqtts://host:8883" — TLS exigé par défaut
  pub invited_by: String,        // fingerprint 40 hex de l'invitant
  pub room_name: Option<String>, // hint local, NON signé
  pub sig: String,               // signature PGP détachée sur "room_id || \x00 || relay || \x00 || invited_by"
}

// Encodage : serde_json(JoinCode) → bytes → base64url-no-pad → "pgpilot:join:<base64>"
```

> **Arbitrage T2.1 vs T2.2** : T2.1 ne définissait pas de signature. T2.2 ajoute `sig`. **Décision retenue : T2.2** (signature obligatoire) — sans elle, n'importe qui peut forger un join code dirigeant les victimes vers un broker malveillant. La vérification utilise le keyring local : si la clef de `invited_by` est absente, le code est rejeté avec `ChatError::JoinCodeInviterUnknown`, l'utilisateur doit d'abord importer la clef publique de l'invitant.

`room_name` est un simple hint d'affichage et n'est pas signé (chaque destinataire peut renommer le salon localement).

**Pas d'expiration** (T2.2 §3.2) : un join code reste valide tant que l'invitant n'a pas révoqué sa clef. Pour retirer un participant, on retire son fingerprint de `participants` dans le `rooms.yaml` local — les nouveaux messages ne lui seront plus chiffrés.

### 6.4 `rooms.yaml`

```yaml
rooms:
  - id: "7f3a2b41-1234-5678-abcd-ef0123456789"
    name: "salon-pgp"
    relay: "mqtts://test.mosquitto.org:8883"
    my_fp: "ALICE00000000000000000000000000000000000A"
    created_at: "2026-05-05T10:00:00Z"
    participants:
      - fp: "ALICE00000000000000000000000000000000000A"
        joined_at: "2026-05-05T10:00:00Z"
      - fp: "BOB000000000000000000000000000000000000B"
        joined_at: "2026-05-05T10:05:00Z"
```

**Types Rust** :

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomParticipant {
  pub fp: String,                                  // fingerprint 40 hex
  pub joined_at: chrono::DateTime<chrono::Utc>,    // RFC 3339
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
  pub id: String,                                  // UUID v4
  pub name: String,                                // libellé local
  pub relay: String,                               // URL MQTT (mqtts:// recommandé)
  pub my_fp: String,                               // identité locale dans ce salon
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub participants: Vec<RoomParticipant>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoomStore {
  pub rooms: Vec<Room>,
}

impl Room {
  pub fn chat_topic(&self) -> String;  // pgpilot/chat/{sha256_hex(id)[0..16]}
}

impl RoomStore {
  pub fn path() -> std::path::PathBuf;     // ~/.config/pgpilot/rooms.yaml
  pub fn load() -> ChatResult<Self>;       // tolère absent → Self::default()
  pub fn save(&self) -> ChatResult<()>;
  pub fn get(&self, id: &str) -> Option<&Room>;
  pub fn upsert(&mut self, room: Room);
  pub fn remove(&mut self, id: &str) -> Option<Room>;
}
```

> **Arbitrage T2.1 vs T2.2** :
> - T2.1 : `participants: Vec<String>` (juste les fingerprints).
> - T2.2 : `Vec<RoomParticipant>` avec `joined_at` + champ `my_fp` au niveau Room.
> - **Décision : T2.2** — le champ `my_fp` est nécessaire pour les utilisateurs ayant plusieurs clefs privées (cas réel : clef pro + clef perso) ; `joined_at` est gratuit (écrit une fois) et utile à l'UI.
> - **Non retenu : `last_seen` par participant** (T2.2 §4.1) — donnée volatile, RAM uniquement.

### 6.5 `ChatMessage` (en RAM uniquement)

```rust
#[derive(Debug, Clone)]
pub struct ChatMessage {
  pub id: String,                                  // UUID v4 du WireMessage
  pub sender_fp: String,                           // fingerprint 40 hex
  pub text: String,                                // plaintext déchiffré
  pub ts: chrono::DateTime<chrono::Utc>,           // ts du WireMessage (sender)
  pub received_at: chrono::DateTime<chrono::Utc>,  // ts de réception locale
  pub direction: MessageDirection,                 // Sent | Received
  pub acks: std::collections::HashMap<String, AckStatus>, // fp → AckStatus
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AckStatus { Pending, Received }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageDirection { Sent, Received }
```

Champ `received_at` (ajout T2.2) : permet à l'UI de détecter et marquer les messages antidatés (`ts << received_at`) ou venus du futur.

### 6.6 `ChatPayload` et `VerifiedMessage` (interne crypto)

```rust
/// Sortie de encrypt_for_room — entrée de WireMessage (encapsulé par wire.rs).
#[derive(Debug, Clone)]
pub struct ChatPayload {
  pub ciphertext_armored: String,
  pub signature_armored: String,
}

/// Sortie de decrypt_message.
#[derive(Debug, Clone)]
pub struct VerifiedMessage {
  pub plaintext: String,
  pub signer_fp: Fingerprint,
  pub signed_at: chrono::DateTime<chrono::Utc>,
}
```

> **Pas de contradiction** entre T2.1 (qui définissait `ChatPayload`) et T2.2 (qui définissait `WireMessage`) : `ChatPayload` est l'output de `crypto.rs`, `WireMessage` est ce que `wire.rs` produit en y ajoutant `id`, `sender`, `ts`. Le pipeline complet est :
> ```
> plaintext → crypto::encrypt_for_room → ChatPayload → wire::build_wire_message → WireMessage → JSON → MQTT publish
> ```

---

## 7. Topics MQTT et règles QoS

```
pgpilot/chat/{sha256(room_id)[0..16]}    → messages chat
pgpilot/presence/{fingerprint[0..16]}    → présence (LWT + heartbeat)
pgpilot/ack/{msg_id[0..16]}              → accusés de réception
```

**Décisions T2.2 retenues sans modification** :

| Topic | QoS | Retain | LWT | Justification |
|---|---|---|---|---|
| `chat/...` | 1 | non | non | At-least-once + dédup par UUID. Pas de retain : un nouveau participant ne reçoit pas l'historique (cohérent éphémère). |
| `presence/...` | 0 | **oui** | **oui** | Le statut Online/Offline doit être visible immédiatement aux nouveaux connectés. Le LWT broker publie automatiquement `offline` à la déconnexion brutale. |
| `ack/...` | 0 | non | non | Best-effort. Un ACK perdu laisse le statut `Pending`, sans impact sécurité. |

### 7.1 Pourquoi tronquer à 16 hex ?

Le fingerprint complet (40 hex = 160 bits) dans un topic MQTT est lisible par le broker et tout outil de monitoring. **16 hex = 64 bits** offre 2^64 valeurs distinctes — suffisant pour éviter les collisions accidentelles dans des salons de quelques dizaines de participants, tout en réduisant l'information identifiable exposée. Le fingerprint complet reste dans le champ `sender` du `WireMessage` signé.

Le `room_id` est lui aussi tronqué via `sha256(room_id)[0..16]` (en hex) pour que le nom du salon ne soit pas reconstituable depuis le topic par un observateur extérieur.

### 7.2 Configuration broker recommandée

- TLS obligatoire (`mqtts://`, port 8883). Le client refuse `mqtt://` plain par défaut (`ChatError::TlsError` si forcé).
- `max_packet_size = 131_072` (128 Kio).
- Authentification optionnelle : si activée côté broker, `MqttConfig` doit pouvoir porter `user`/`password`. Hors scope de la première implémentation v0.6.0 : on cible des brokers publics ou privés sans auth.

---

## 8. Intégration iced ↔ MQTT (réponse à Q1)

### 8.1 Confirmation de l'approche

**Oui, l'approche `Subscription::run` autour d'un Stream rumqttc est confirmée.** C'est l'API idiomatique d'iced 0.14 pour brancher un flux d'évènements externes asynchrones, équivalent à un WebSocket dans une app iced standard.

### 8.2 Architecture détaillée

```
┌──────────────────────────────────────────────────────────────────┐
│                      Tâche tokio dédiée (spawn au 1er démarrage) │
│                                                                  │
│   ┌─────────────────────┐         ┌──────────────────────┐       │
│   │ rumqttc::AsyncClient│  poll   │ rumqttc::EventLoop   │       │
│   │ (publish, subscribe)│ ◄─────► │ (réception, reconnect)│      │
│   └──────────▲──────────┘         └──────────┬───────────┘       │
│              │                               │                   │
│              │ MqttCmd                       │ Incoming::Publish │
│              │ (Subscribe / Publish / etc.)  │                   │
│   ┌──────────┴──────────┐         ┌──────────▼───────────┐       │
│   │ cmd_rx (mpsc)       │         │ event_tx (mpsc, 256) │       │
│   └──────────▲──────────┘         └──────────┬───────────┘       │
└──────────────┼────────────────────────────────┼──────────────────┘
               │                                │
               │ MqttHandle.cmd_tx              │ Stream<MqttEvent>
               │                                │
┌──────────────┴────────────────────────────────▼──────────────────┐
│                         App (thread iced)                        │
│                                                                  │
│   App.mqtt: Option<MqttHandle>          App.subscription():      │
│   ↳ .publish(topic, payload, ...)          Subscription::run     │
│     ↳ via cmd_tx.send(MqttCmd::Publish)    autour du Stream      │
│                                            ↳ Message::MqttEvent  │
└──────────────────────────────────────────────────────────────────┘
```

### 8.3 Points clés d'implémentation

1. **Une seule tâche tokio possède le client rumqttc**. Aucun `Arc<Mutex<AsyncClient>>` exposé ailleurs (rejeté formellement — risque de deadlock entre eventloop et publishers).
2. **Communication via `mpsc`** :
   - `cmd_tx` (UI → tâche) : illimité côté UI car les commandes sont rares.
   - `event_tx` (tâche → UI) : borné à 256 ; au-delà, on **drop les évènements anciens** (avec log warn). Évite que l'UI freezée n'accumule indéfiniment.
3. **`MqttHandle` est `Clone + Send + Sync`** trivialement (contient un `mpsc::Sender`). Cloneable autant que voulu dans des `Task::perform`.
4. **Le Stream sortant** est créé une fois au `spawn`, stocké dans un `Arc<Mutex<Option<Stream>>>` interne au handle et `take()` au premier appel de `subscription()`. Iced ne ré-appelle pas la factory tant que l'`id` du `Subscription::run_with_id` est stable.
5. **Squelette `App::subscription`** :

```rust
pub fn subscription(&self) -> iced::Subscription<Message> {
  let file_drop = iced::event::listen_with(|event, _, _| match event {
    iced::Event::Window(iced::window::Event::FileDropped(p)) => Some(Message::FileDropped(p)),
    _ => None,
  });

  let mut subs = vec![file_drop];

  if let Some(handle) = &self.mqtt {
    subs.push(crate::chat::mqtt::subscription(handle.clone()));
  }

  iced::Subscription::batch(subs)
}
```

`crate::chat::mqtt::subscription(handle)` retourne un `Subscription<Message>` qui wrappe le Stream interne via `Subscription::run_with_id` et mappe chaque `MqttEvent` vers `Message::MqttEvent(_)`.

### 8.4 Reconnexion

`rumqttc::EventLoop::poll()` retry implicitement sur erreur. On ajoute :
- Émission d'évènements `Reconnecting { attempt }` à chaque tentative.
- Backoff exponentiel borné : base = `MQTT_RECONNECT_BASE_MS` (1 s), plafond = `MQTT_RECONNECT_MAX_MS` (60 s), facteur ×2.
- Pour limiter le bruit, n'émettre `Reconnecting` qu'à la 1re tentative et toutes les 5 suivantes (ou à chaque fois si `attempt < 3`).

---

## 9. Gestion des clefs (réponse à Q2)

> **Correction importante par rapport à la formulation initiale de la question** : la spec ne charge **pas** la clef privée à chaque opération. Elle est chargée **une fois par session** (au premier accès au chat) et reste en RAM jusqu'à fermeture de l'application. C'est un compromis assumé latence vs sécurité.

### 9.1 Architecture retenue : sequoia in-process, chargement unique par session

- **Cryptographie en process via sequoia** (pas de subprocess `gpg --encrypt` par message). Justification : un appel `gpg --encrypt` coûte 200–500 ms et fait apparaître pinentry pour la signature ; inacceptable en chat temps réel.
- **Une seule entrée dans `gpg --export-secret-keys --armor <fp>`** au démarrage du chat (premier accès) pour exporter la clef privée vers la RAM. C'est le seul moment où pinentry peut apparaître (si la clef a une passphrase).
- **`ChatCryptoCtx` stocké dans `App.chat_crypto: Option<Arc<ChatCryptoCtx>>`** vit pour toute la durée de la session pgpilot.

```rust
pub struct ChatCryptoCtx {
  pub local_cert: sequoia_openpgp::Cert,                  // clef privée locale (Cert sequoia complet)
  pub local_fp: Fingerprint,
  pub peers: HashMap<Fingerprint, sequoia_openpgp::Cert>, // certs publics des participants
}

impl ChatCryptoCtx {
  pub fn load(local_fp: &str, peers: &[Fingerprint]) -> ChatResult<Self>;
  pub fn encrypt_for_room(&self, plaintext: &str, recipients: &[Fingerprint]) -> ChatResult<ChatPayload>;
  pub fn decrypt_message(&self, payload: &ChatPayload) -> ChatResult<VerifiedMessage>;
}
```

`load()` est appelé **dans un `blocking_task`** car `gpg --export-secret-keys` est bloquant. `encrypt_for_room` et `decrypt_message` sont également invoqués via `blocking_task` (chiffrement/signature sequoia ~10–50 ms par message — négligeable mais respecte le pattern Send + 'static d'iced).

### 9.2 Compromis de sécurité

- **Clef privée en RAM toute la session** : oui, c'est le coût d'un chat temps réel sans pinentry par message.
- **Pas de zeroize en v0.6.0** : la `Cert` sequoia n'est pas wipée à `Drop`. Documenté dans l'axe 7 sécurité comme limite assumée. La mitigation reste OS-level (mémoire libérée au quit, pas de swap recommandé sur les machines sensibles).
- **Pas de YubiKey en v0.6.0** : la clef privée d'une YubiKey n'est pas exportable. Tentative d'utiliser une clef YubiKey pour le chat → `ChatError::SignFailed("clef sur smartcard non supportée pour le chat v0.6.0")`. Comportement : refus à l'ouverture de la première room avec un message clair vers l'utilisateur ("créez ou importez une clef logicielle pour utiliser le chat").

### 9.3 Reload de `chat_crypto` lors de l'ajout de participants

Si l'utilisateur rejoint une nouvelle room avec un participant non encore présent dans `peers`, on **recharge intégralement** `ChatCryptoCtx` (`load(local_fp, all_known_peers_fps)`) plutôt que de patcher en place. Plus simple, opération rare. Coût : un `blocking_task` ~100 ms.

---

## 10. Gestion des erreurs et états de connexion (réponse à Q3)

### 10.1 Démarrage paresseux

**Confirmation** : aucune connexion MQTT au lancement de l'application. `App::new` initialise `mqtt: None`, `mqtt_state: MqttState::Disconnected`. La connexion est démarrée via `ensure_chat_started(room_id)`, déclenché par :
- `Message::ChatRoomSelected(room_id)` (clic sur une room dans la liste)
- `Message::ChatRoomJoined(Ok(room))` (succès de jointure)
- `Message::ChatRoomCreated(Ok(room))` (succès de création — l'utilisateur veut probablement entrer dedans immédiatement)

Cela permet à pgpilot de fonctionner pleinement (gestion clefs, chiffrement fichiers, etc.) sans réseau ni broker.

### 10.2 Cycle de vie `MqttState`

```
        ensure_chat_started()
          │
          ▼
    Disconnected ──────► Connecting ──────► Connected
          ▲                  │  ▲                │
          │                  │  │                │
          │                  ▼  │                ▼
          └─── Failed(reason) ◄─┴──── Reconnecting { attempt }
                                       (backoff 1s → 60s)
```

### 10.3 Comportement si broker injoignable au démarrage

**Confirmation et précisions** :

1. **`MqttState::Connecting`** est positionné dès l'appel à `ensure_chat_started`.
2. La tâche tokio démarre `rumqttc::EventLoop::poll()`. La première erreur de connexion (DNS, TCP refused, TLS handshake fail) émet un `MqttEvent::Disconnected(reason)` puis `MqttEvent::Reconnecting { attempt: 1 }`.
3. **`MqttState` passe à `Reconnecting { attempt }`** (pas `Failed`). `Failed` est réservé aux erreurs **non-récupérables** : URL malformée, auth invalide rejetée définitivement, certificat TLS auto-signé refusé sans override.
4. **L'UI affiche un badge** dans la status bar : "MQTT : reconnexion (tentative N)" en couleur warning. Pas d'erreur bloquante affichée — l'utilisateur peut continuer à utiliser le reste de l'app.
5. **Retry automatique** via la boucle rumqttc avec backoff borné (1 s → 60 s, ×2). Pas de cap en nombre de tentatives — on retry indéfiniment tant que la room est ouverte.
6. **Si l'utilisateur quitte la section chat** (`View::ChatList` → autre vue), on **garde la connexion** (le coût d'une socket idle est négligeable) mais on pourrait l'arrêter via `MqttHandle::shutdown()` dans une optimisation future. Décision v0.6.0 : on garde la connexion ouverte tant que `App` vit.
7. **Au reconnect réussi** (`MqttEvent::Connected`), `subscribe_all_known_topics()` est appelé pour ré-abonner à tous les topics de toutes les rooms connues + la présence des participants. C'est essentiel : un broker reset perd les souscriptions.

### 10.4 Affichage UI des erreurs

| Évènement | Affichage |
|---|---|
| `MqttState::Connecting` | Badge gris "Connexion..." dans status bar chat |
| `MqttState::Connected` | Badge vert "Connecté" (transitoire 2 s puis caché) |
| `MqttState::Reconnecting { attempt }` | Badge orange "Reconnexion (N)" persistant |
| `MqttState::Failed(reason)` | Status `StatusKind::Error` + badge rouge persistant |
| `ChatError::DecryptFailed` reçu | **Aucun affichage** (silencieux, log interne uniquement) |
| `ChatError::SignatureInvalid` reçu | **Aucun affichage** (le message est rejeté) |
| `ChatError::EncryptFailed` à l'envoi | Status `StatusKind::Error` immédiat |
| `ChatError::MessageTooLarge` | Status `StatusKind::Error` à la saisie (avant publish) |
| `ChatError::JoinCodeInviterUnknown` | Status `StatusKind::Error` avec message i18n explicite |
| `ChatError::JoinCodeSignatureInvalid` | Status `StatusKind::Error` (rejet de l'invitation) |

### 10.5 `ChatError` consolidé

```rust
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum ChatError {
  // Connectivité MQTT
  #[error("MQTT non connecté")]
  MqttNotConnected,
  #[error("Broker injoignable : {0}")]
  BrokerUnreachable(String),
  #[error("Erreur protocole MQTT : {0}")]
  MqttProtocolError(String),
  #[error("Erreur TLS : {0}")]
  TlsError(String),

  // Cryptographie
  #[error("Échec chiffrement : {0}")]
  EncryptFailed(String),
  #[error("Échec déchiffrement : {0}")]
  DecryptFailed(String),
  #[error("Signature invalide")]
  SignatureInvalid,
  #[error("Échec signature : {0}")]
  SignFailed(String),

  // Identité et rooms
  #[error("Émetteur inconnu : {0}")]
  UnknownSender(String),
  #[error("Salon introuvable : {0}")]
  RoomNotFound(String),
  #[error("Aucune clef de signature utilisable")]
  NoSigningKey,
  #[error("Participant absent du keyring : {0}")]
  ParticipantNotInKeyring(String),

  // Join code
  #[error("Code d'invitation invalide")]
  InvalidJoinCode,
  #[error("Signature du code d'invitation invalide")]
  JoinCodeSignatureInvalid,
  #[error("Clef de l'invitant absente du keyring")]
  JoinCodeInviterUnknown,

  // Validation message
  #[error("Message trop volumineux (max {} octets)", crate::chat::MAX_WIRE_MESSAGE_BYTES)]
  MessageTooLarge,
  #[error("Message dupliqué : {0}")]
  MessageIdDuplicate(String),
  #[error("Message wire malformé : {0}")]
  MalformedWireMessage(String),
  #[error("Fingerprint invalide : {0}")]
  InvalidFingerprint(String),

  // Persistance
  #[error("Lecture rooms.yaml impossible : {0}")]
  RoomsYamlLoadFailed(String),
  #[error("Écriture rooms.yaml impossible : {0}")]
  RoomsYamlSaveFailed(String),

  // Configuration
  #[error("Configuration invalide : {0}")]
  InvalidConfig(String),
}

pub type ChatResult<T> = std::result::Result<T, ChatError>;
```

> **Arbitrage T2.1 vs T2.2** : T2.1 listait 8 variants, T2.2 en listait 21. **Décision : retenir l'ensemble T2.2 enrichi** des variants T2.1 (`InvalidConfig`) qui manquaient. La granularité fine est utile pour les tests et la localisation des messages d'erreur.

> **Précision sur le mapping i18n** : tous les `ChatError` doivent avoir une traduction dans le trait `Strings` (méthode `chat_error_<variant>(detail: &str) -> String`). L'axe 5 (UI) précisera les chaînes exactes en EN/FR.

---

## 11. Stratégie de test (réponse à Q4)

### 11.1 Question : comment tester le transport sans vrai broker ?

**Réponse à 3 niveaux**, du plus rapide au plus complet.

### 11.2 Niveau 1 : tests unitaires avec `MockTransport` (ChatTransport mock)

Le trait `ChatTransport` (défini en T2.1 §1.3) est l'abstraction qui rend ceci possible :

```rust
#[async_trait::async_trait]
pub trait ChatTransport: Send + Sync {
  async fn subscribe(&self, topic: &str, qos: u8) -> ChatResult<()>;
  async fn unsubscribe(&self, topic: &str) -> ChatResult<()>;
  async fn publish(&self, topic: &str, payload: Vec<u8>, qos: u8, retain: bool) -> ChatResult<()>;
}
```

`MqttHandle` implémente `ChatTransport`. Pour les tests :

```rust
// tests/chat_mock_transport.rs
struct MockTransport {
  published: Mutex<Vec<(String, Vec<u8>, u8, bool)>>,
  subscriptions: Mutex<HashSet<String>>,
  inbound_tx: tokio::sync::mpsc::UnboundedSender<MqttEvent>,
}

#[async_trait::async_trait]
impl ChatTransport for MockTransport {
  async fn publish(&self, topic: &str, payload: Vec<u8>, qos: u8, retain: bool) -> ChatResult<()> {
    self.published.lock().unwrap().push((topic.into(), payload, qos, retain));
    Ok(())
  }
  // ...
}
```

**Couverture** :
- Sérialisation `WireMessage` ↔ JSON (round-trip).
- Canonicalisation pour signature (vérifier les séparateurs `\x00`).
- Encode/decode `JoinCode` (base64url, vérification signature avec un keyring temporaire).
- Bornage `MessageTooLarge` (rejeter un payload > 64 Kio).
- `PresenceTracker.apply()` change correctement les états.
- Routage `dispatch_mqtt_payload` selon le préfixe topic.

Tous les handlers `app/chat.rs` qui n'ont pas besoin d'un vrai eventloop peuvent être testés contre un `App` injecté avec `mqtt: Some(MqttHandle::from_transport(Arc::new(MockTransport::new())))`.

> **Décision retenue : niveau 1 obligatoire, axé sur la logique métier**. Permet des tests `cargo test --lib` rapides (< 1 s).

### 11.3 Niveau 2 : tests d'intégration avec broker MQTT embarqué

**Crate retenue : `rumqttd`** (le broker compagnon de `rumqttc`, du même auteur — bytebeam.io).

- Disponible sur crates.io (`rumqttd = "0.19"` ou plus récent).
- Embarqué dans le binaire de test, démarré sur `127.0.0.1:<port aléatoire>` dans un `tempdir`.
- Configurable programmatiquement (pas de fichier conf à charger).
- Supporte TLS (avec un certificat auto-signé généré par `rcgen` en `dev-dependencies`).

Pattern :

```rust
// tests/common/mqtt_broker.rs
pub struct EmbeddedBroker {
  pub addr: SocketAddr,
  _shutdown: tokio::sync::oneshot::Sender<()>,
  _tempdir: TempDir,
}

impl EmbeddedBroker {
  pub async fn start() -> Self { /* spawn rumqttd::Broker sur port libre */ }
  pub fn url(&self) -> String { format!("mqtt://{}", self.addr) }
}
```

**Couverture** :
- Round-trip complet : Alice publish → broker → Bob receive → décrypte → ACK.
- Reconnexion : tuer le broker, relancer, vérifier que les souscriptions sont restaurées.
- LWT : déconnexion brutale (drop du client) → autres clients reçoivent le `offline` rétention.
- Retain `presence` : nouveau client connecté reçoit le statut courant.
- Plusieurs participants simultanés (spawn 3+ tasks tokio dans le test).

> **Décision retenue : niveau 2 obligatoire, marqué `#[ignore]`** comme les tests `gpg` lents. Run via `cargo test -- --ignored`. Démarrage broker ~100 ms, tests typiques 500 ms à 2 s.

### 11.4 Niveau 3 : test manuel contre broker public

Test exploratoire (pas dans la suite CI) contre un broker public type `test.mosquitto.org` ou `broker.hivemq.com`. Sert à valider la latence réelle, le comportement TLS et les éventuelles particularités de brokers exotiques. **Non automatisé**.

### 11.5 Tests cryptographie (sans MQTT du tout)

Module `tests/chat_crypto.rs` :
- Génère 2 keypairs PGP dans un `setup_test_gnupghome()` (pattern existant).
- Charge `ChatCryptoCtx::load(alice_fp, [bob_fp])` côté Alice et inverse côté Bob.
- `alice.encrypt_for_room("hello", &[bob_fp])` → `bob.decrypt_message(payload)` → vérifie plaintext + signer_fp.
- Test de tamper : modifier 1 octet de `payload.signature_armored` → `decrypt_message` doit retourner `SignatureInvalid`.
- Test substitution : intercepter le `WireMessage`, changer `sender` → vérifier que la signature canonicalisée échoue.

> **Décision : ces tests vivent dans `tests/chat_crypto.rs`** et utilisent les helpers existants `tests/common/mod.rs` (extension : ajouter `setup_two_test_keypairs()`).

### 11.6 Tests handlers `app/chat.rs`

Module `tests/app_chat_handlers.rs` (suit le pattern `tests/app_handlers.rs` existant).
- Crée un `App` minimal (sans iced rendering).
- Injecte `MockTransport` via `MqttHandle::from_transport`.
- Fait tourner les handlers (`on_chat_send`, `on_chat_received`, etc.) et inspecte l'état résultant.
- Vérifie : bornage à 500 messages, statut `MqttState`, contenu de `chat_messages`, ACK envoyé après réception.

### 11.7 Récapitulatif

| Niveau | Outil | Vitesse | `#[ignore]` ? | Couverture |
|---|---|---|---|---|
| 1 — unit | `MockTransport` (mock du trait `ChatTransport`) | < 1 s | non | Logique métier, sérialisation, validation |
| 2 — intégration | `rumqttd` embarqué (`tests/common/mqtt_broker.rs`) | 100 ms – 2 s | **oui** | Round-trip réseau, reconnect, LWT, retain |
| 3 — manuel | Broker public | N/A | N/A (hors CI) | Validation production |
| crypto | sequoia + gpg test homedir | < 100 ms | non | encrypt/decrypt/sign/verify, tamper detection |
| handlers | `App` + `MockTransport` | < 100 ms | non | Comportement des `on_*` sans iced |

---

## 12. Hors-scope v0.6.0

Documenté pour mémoire et pour orienter v0.7+ :

- **Persistance optionnelle des messages** (chiffrée localement en SQLite + sequoia). Reste explicitement hors v0.6.0 : conformité à l'exigence "éphémère par conception".
- **Drafts par room** (`HashMap<RoomId, String>` au lieu d'un `chat_input` global).
- **Multi-device** (un même utilisateur sur 2 instances pgpilot avec synchronisation des messages).
- **Pièces jointes** (fichiers chiffrés transmis via MQTT ou hors-bande type S3/object store).
- **YubiKey support** (signature in-card sans export via `gpg --sign` + ChatTransport spécial qui passe par subprocess plutôt que sequoia in-process — architecture différente).
- **Forward secrecy** (per-message ephemeral keys, type Signal X3DH/Double Ratchet).
- **Modération / kick / ban** (un participant retiré reste capable de lire les anciens messages capturés ; réelle révocation impossible sans re-keying complet).
- **Refacto master-detail** entre `key_list` et `chat_list` (laisser diverger le temps de v0.6.0, factoriser plus tard si patterns convergent).
- **Authentification broker** (user/password ou JWT). À ajouter dans `MqttConfig` si besoin v0.7+.
- **Zeroize de `ChatCryptoCtx`** au quit. Mitigation OS-level pour l'instant.
- **Auth mutuelle TLS** (mTLS client cert) côté broker.

---

## 13. Synthèse des arbitrages T2.1 vs T2.2

| # | Sujet | T2.1 | T2.2 | Décision finale | Justification |
|---|---|---|---|---|---|
| 1 | Layout modules | 5 fichiers | (n/a) | **6 fichiers** (ajout `wire.rs`) | `WireMessage`/`WireAck` méritent leur module dédié |
| 2 | `Room.participants` | `Vec<String>` (fp) | `Vec<RoomParticipant>` (fp + joined_at) | **T2.2** | `joined_at` utile UI, gratuit |
| 3 | `Room.my_fp` | absent | présent | **T2.2** | Multi-clef privée |
| 4 | `JoinCode.sig` | absent | présent | **T2.2** | Sécurité : empêche forgery |
| 5 | Bornage messages/room | 200 (VecDeque) | 500 (Vec) | **500 dans VecDeque** | Confort utilisateur + bornage O(1) |
| 6 | `ChatPayload` vs `WireMessage` | `ChatPayload` | `WireMessage` | **Les deux** (rôles distincts) | `ChatPayload` interne crypto, `WireMessage` sur le wire |
| 7 | `MQTT_KEEPALIVE_SECS` | 30 | 60 | **60** | Cohérent avec `PRESENCE_LWT_TIMEOUT_SECS = 90` (3× heartbeat) |
| 8 | `MQTT_RECONNECT_MAX_MS` | 30 000 | 60 000 | **60 000** | Backoff plus généreux pour brokers récalcitrants |
| 9 | `ChatError` granularité | 8 variants | 21 variants | **T2.2 enrichi** (+ `InvalidConfig` de T1) | Granularité fine = meilleurs tests + i18n |
| 10 | `ChatDecryptFailed` Message | présent | (n/a, T2.2 §5.2 dit "ignore") | **Supprimé** | Cohérence avec décision "ignore silencieux" |
| 11 | `ChatRoomCreate(String)` | sans paramètre | (n/a) | **Sans paramètre** | Suit pattern `CreateKeySubmit` |
| 12 | `View` chat | `ChatList`, `ChatRoom` | (n/a) | **+ `ChatNewRoom`, `ChatJoinRoom`** | Vues plein-écran cohérentes avec `CreateKey`/`Import` |

---

## 14. Constantes globales (`src/chat/mod.rs`)

```rust
pub const CHAT_TOPIC_PREFIX:     &str   = "pgpilot/chat";
pub const PRESENCE_TOPIC_PREFIX: &str   = "pgpilot/presence";
pub const ACK_TOPIC_PREFIX:      &str   = "pgpilot/ack";

pub const MAX_WIRE_MESSAGE_BYTES:    usize = 65_536;        // 64 Kio JSON sérialisé
pub const MAX_MESSAGES_PER_ROOM:     usize = 500;           // bornage RAM, FIFO
pub const PRESENCE_HEARTBEAT_SECS:   u64   = 30;
pub const PRESENCE_LWT_TIMEOUT_SECS: u16   = 90;            // 3 × heartbeat
pub const MQTT_KEEPALIVE_SECS:       u16   = 60;
pub const MQTT_RECONNECT_BASE_MS:    u64   = 1_000;         // backoff exp base 1 s
pub const MQTT_RECONNECT_MAX_MS:     u64   = 60_000;        // plafond 60 s
pub const MQTT_EVENT_CHANNEL_CAP:    usize = 256;           // mpsc tâche → UI
pub const MQTT_RECONNECT_LOG_EVERY:  u32   = 5;             // n'émettre Reconnecting que toutes les N tentatives au-delà de 3

pub const SIGN_CANONICAL_PREFIX: &[u8] = b"pgpilot-msg\x00";
pub const JOIN_CODE_PREFIX:      &str  = "pgpilot:join:";
```

---

## 15. Critères d'acceptation de la spec finale

- [x] Layout modules `src/chat/` défini (6 fichiers, dépendances internes acycliques)
- [x] Champs ajoutés à `App` listés avec types exacts
- [x] Variants `View` exhaustifs (ChatList, ChatRoom, ChatNewRoom, ChatJoinRoom)
- [x] Variants `Message` exhaustifs (création/jointure/envoi/réception/MQTT/présence/ACK)
- [x] Signatures de tous les handlers `app/chat.rs`
- [x] Formats `WireMessage`, `WireAck`, `JoinCode`, `rooms.yaml`, `ChatMessage`, `ChatPayload` définis
- [x] Topics MQTT et règles QoS spécifiées
- [x] Q1 : intégration iced ↔ MQTT — `Subscription::run` autour d'un Stream rumqttc, mpsc bornée 256
- [x] Q2 : vie des clefs — sequoia in-process, chargement unique par session, RAM jusqu'au quit
- [x] Q3 : broker injoignable — `MqttState::Reconnecting` avec backoff borné, badge UI, retry indéfini
- [x] Q4 : tests sans broker — 3 niveaux (mock trait, `rumqttd` embarqué, manuel)
- [x] Hors-scope documenté
- [x] Contradictions T2.1/T2.2 arbitrées dans §13
