# Axe 4 — Core : rooms & messages

## Objectif

Implémenter la logique métier du chat : rooms (CRUD + persistance), chiffrement/déchiffrement
des messages, intégration dans `App`. Pas d'UI ici.

**Référence** : `axe2-spec-finale.md`

---

## T4.1 — Rooms : struct + persistance

**Complexité** : M
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T3.1

### Fichier : `src/chat/rooms.rs`

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,                    // UUID v4
    pub name: String,                  // local uniquement
    pub relay: String,                 // "mqtt://test.mosquitto.org:8883"
    pub participants: Vec<String>,     // fingerprints 40 hex
    pub created_at: DateTime<Utc>,
}

impl Room {
    pub fn new(name: String, relay: String, participants: Vec<String>) -> Self;
    pub fn topic(&self) -> String;     // "pgpilot/chat/{SHA256(id)[0..16]}"
    pub fn to_join_code(&self, inviter_fp: &str) -> String;  // base64url(JSON)
}

pub fn from_join_code(code: &str) -> Result<Room, String>;
pub fn rooms_path() -> Result<std::path::PathBuf, String>;  // ~/.config/pgpilot/rooms.yaml
pub fn load_rooms() -> Result<Vec<Room>, String>;
pub fn save_rooms(rooms: &[Room]) -> Result<(), String>;
```

### Format `rooms.yaml`

```yaml
rooms:
  - id: "7f3a2b41-..."
    name: "salon-pgp"
    relay: "mqtt://test.mosquitto.org:8883"
    participants:
      - "ALICE..."
      - "BOB..."
    created_at: "2026-05-04T10:00:00Z"
    my_identity: "ALICE..."   # fingerprint de la clef privée choisie pour cette room
```

Le champ `my_identity` est obligatoire. Il est sélectionné à l'entrée dans la room (voir T4.4).

**Commit** : `feat(chat): Room struct, rooms.yaml persistence, join code`

---

## T4.4 — Identité et gestion du cycle de vie d'une room

**Complexité** : M
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T4.1

### Sélection d'identité à l'entrée dans une room

Quand l'utilisateur entre dans une room pour la première fois (création ou join), PGPilot doit
lui demander **quelle clef privée il souhaite incarner** dans cette room.

Cette question se pose uniquement si l'utilisateur possède **plusieurs clefs privées** dans son
keyring. Si une seule clef privée existe, elle est sélectionnée automatiquement sans modal.

Le choix est persisté dans `rooms.yaml` sous `my_identity` (fingerprint). Il est rappelable
mais modifiable : l'utilisateur peut changer d'identité pour une room via les settings de la room.

**Nouveaux `Message` variants** :
```rust
Message::ChatIdentityRequired(String),               // room_id — déclenche le modal
Message::ChatIdentitySelected(String, String),       // room_id, identity_fp
```

**Handler `on_chat_identity_selected`** :
```rust
fn on_chat_identity_selected(&mut self, room_id: String, identity_fp: String) -> Task<Message> {
    if let Some(room) = self.rooms.iter_mut().find(|r| r.id == room_id) {
        room.my_identity = identity_fp;
    }
    save_rooms(&self.rooms).ok();
    // Puis naviguer vers View::ChatRoom(room_id)
    self.on_chat_room_selected(room_id)
}
```

**Dans `on_chat_send`** : utiliser `room.my_identity` comme `signer_fp`.

### Quitter une room

**Nouveaux `Message` variants** :
```rust
Message::ChatLeaveRoom(String),                      // room_id — demande confirmation
Message::ChatLeaveRoomConfirmed(String),             // room_id — effectue le leave
```

**Comportement** :
- Afficher un modal de confirmation : "Leave this room? You will no longer receive messages."
- À la confirmation :
  1. Unsubscribe du topic MQTT de la room
  2. Publier présence "offline" sur le topic de présence (signalé aux autres)
  3. Supprimer la room de `self.rooms`
  4. Sauvegarder `rooms.yaml`
  5. Naviguer vers `View::ChatList`

**PendingOp** à ajouter :
```rust
PendingOp::LeaveRoom(String)  // room_id
```

**Handler** :
```rust
fn on_chat_leave_room(&mut self, room_id: String) -> Task<Message> {
    self.reset_pending_ops();
    self.pending = Some(PendingOp::LeaveRoom(room_id));
    Task::none()
}

fn on_chat_leave_room_confirmed(&mut self, room_id: String) -> Task<Message> {
    // unsubscribe + publish offline + remove room + save + navigate
}
```

**Commit** : `feat(chat): identity selection on room enter, leave room with confirmation`

---

## T4.2 — Chiffrement des messages

**Complexité** : L
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T4.1

### Fichier : `src/chat/crypto.rs`

```rust
use sequoia_openpgp as openpgp;

/// Chiffre `text` pour tous les `recipients_fps` et signe avec `signer_fp`.
/// Retourne un WireMessage prêt à publier.
pub fn encrypt_for_room(
    homedir: &str,
    text: &str,
    room_id: &str,
    signer_fp: &str,
    recipients_fps: &[String],
) -> Result<WireMessage, String>;

/// Déchiffre et vérifie la signature d'un WireMessage.
/// Retourne (texte clair, fingerprint signataire vérifié).
/// Échoue si la signature est invalide ou si sender ne correspond pas.
pub fn decrypt_message(
    homedir: &str,
    msg: &WireMessage,
) -> Result<(String, String), ChatError>;

/// Vérifie que le sender déclaré dans le JSON correspond à la clé qui a signé.
pub fn verify_sender(
    homedir: &str,
    msg: &WireMessage,
) -> Result<(), ChatError>;
```

### Détails d'implémentation

- Réutiliser `gpg_command(homedir)` pour les appels GPG (jamais `Command::new("gpg")` direct)
- Le payload OpenPGP est produit par `gpg --encrypt --sign --armor` avec `--recipient fp1 --recipient fp2 ...`
- La signature dans `WireMessage.signature` couvre `{id}|{sender}|{ts}|{payload}` (string concaténée) — empêche la substitution de metadata
- Si le fingerprint du signataire vérifié ≠ `msg.sender` → `ChatError::SignatureInvalid`
- Si la clé du sender est absente du keyring → `ChatError::UnknownSender`

**Commit** : `feat(chat): PGP message encryption and signature verification`

---

## T4.3 — Intégration dans `App`

**Complexité** : L
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T3.2, T4.1, T4.2

### Champs à ajouter à `App` (`src/app/mod.rs`)

```rust
pub struct App {
    // ... champs existants ...

    // Chat
    pub rooms: Vec<Room>,
    pub active_room: Option<String>,              // room_id
    pub chat_messages: HashMap<String, Vec<ChatMessage>>,  // room_id → messages RAM
    pub presence: HashMap<String, PresenceStatus>,          // fp → Online/Offline
    pub mqtt_connected: bool,
    pub chat_client: Option<Arc<MqttClient>>,
}
```

### Nouveaux `Message` variants (`src/app/mod.rs`)

Ajouter dans l'enum `Message` :

```rust
// Rooms
ChatRoomCreate(String),                   // nom local
ChatRoomJoin(String),                     // join code
ChatRoomSelected(String),                 // room_id
ChatRoomsLoaded(Result<Vec<Room>, String>),

// Envoi / réception
ChatSend(String, String),                 // room_id, texte saisi
ChatSent(String, ChatMessage),            // room_id, message envoyé (pour l'afficher)
ChatReceived(String, Result<ChatMessage, ChatError>), // room_id, résultat déchiffrement

// MQTT infra
MqttEvent(MqttEvent),                     // relayé depuis le channel MQTT

// ACK
ChatAckReceived(String, String, String),  // room_id, msg_id, sender_fp
```

### Handlers (`src/app/chat.rs`)

```rust
impl App {
    pub(super) fn on_chat_room_create(&mut self, name: String) -> Task<Message>;
    pub(super) fn on_chat_room_join(&mut self, code: String) -> Task<Message>;
    pub(super) fn on_chat_room_selected(&mut self, room_id: String) -> Task<Message>;
    pub(super) fn on_chat_send(&mut self, room_id: String, text: String) -> Task<Message>;
    pub(super) fn on_chat_received(&mut self, room_id: String, result: Result<ChatMessage, ChatError>) -> Task<Message>;
    pub(super) fn on_mqtt_event(&mut self, event: MqttEvent) -> Task<Message>;
    pub(super) fn on_chat_ack_received(&mut self, room_id: String, msg_id: String, sender_fp: String) -> Task<Message>;
}
```

### Pattern `on_chat_send`

```rust
fn on_chat_send(&mut self, room_id: String, text: String) -> Task<Message> {
    let Some(room) = self.rooms.iter().find(|r| r.id == room_id).cloned()
        else { return Task::none(); };
    let homedir = match gnupg_dir() { Ok(h) => h, Err(_) => return Task::none() };
    let signer_fp = /* clé par défaut de l'utilisateur */;
    let client = self.chat_client.clone();

    blocking_task(move || {
        let wire_msg = encrypt_for_room(&homedir, &text, &room.id, &signer_fp, &room.participants)?;
        let json = serde_json::to_vec(&wire_msg).map_err(|e| e.to_string())?;
        // publish via client...
        Ok(wire_msg)
    })
    .map(|result| Message::ChatSent(room_id, result.unwrap())) // simplification
}
```

**Commit** : `feat(chat): App integration, Message variants, chat handlers`

---

## Fichiers créés / modifiés

```
src/chat/rooms.rs        (nouveau — + my_identity dans Room)
src/chat/crypto.rs       (nouveau)
src/app/mod.rs           (+ champs App, + Message variants dont ChatIdentityRequired/Selected, ChatLeaveRoom*)
src/app/chat.rs          (nouveau — handlers dont on_chat_identity_selected, on_chat_leave_room*)
```

## Critères d'acceptation

- [ ] `cargo build` ✓
- [ ] `load_rooms()` / `save_rooms()` roundtrip correct (incluant `my_identity`)
- [ ] `from_join_code(room.to_join_code(fp))` reconstruit la room
- [ ] `encrypt_for_room` + `decrypt_message` roundtrip (test avec vraie clé GPG)
- [ ] `verify_sender` rejette un WireMessage dont le sender ne correspond pas au signataire
- [ ] Si une seule clef privée → `my_identity` assignée automatiquement
- [ ] Si plusieurs clefs privées → modal de sélection affiché avant entrée dans la room
- [ ] Leave room : unsubscribe MQTT + suppression rooms.yaml + navigation ChatList
