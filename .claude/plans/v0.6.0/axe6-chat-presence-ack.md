# Axe 6 — Présence & ACK

## Objectif

Implémenter les deux protocoles applicatifs au-dessus du transport MQTT :
- **Présence** : chaque client annonce son état online/offline, signé cryptographiquement
- **ACK** : confirmation de réception et déchiffrement réussi, signé

**Référence** : `axe2-spec-finale.md`

---

## T6.1 — Module présence

**Complexité** : M
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T3.2, T4.3

### Fichier : `src/chat/presence.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PresenceStatus {
    Online,
    Offline,
}

#[derive(Serialize, Deserialize)]
struct PresencePayload {
    status: String,      // "online" | "offline"
    fp: String,          // fingerprint complet 40 hex
    ts: i64,
    signature: String,   // sig OpenPGP sur "{status}|{fp}|{ts}"
}

/// Publie "online" sur le topic de présence. Retained = true.
/// La payload est signée avec la clé privée de l'utilisateur.
pub async fn publish_online(
    client: &MqttClient,
    homedir: &str,
    user_fp: &str,
) -> Result<(), String>;

/// Appelé à la déconnexion propre. Le LWT gère la déconnexion brutale.
pub async fn publish_offline(
    client: &MqttClient,
    homedir: &str,
    user_fp: &str,
) -> Result<(), String>;

/// Subscribe aux topics de présence de tous les participants d'une room.
pub async fn subscribe_room_presence(
    client: &MqttClient,
    room: &Room,
) -> Result<(), String>;

/// Parse et vérifie un message de présence entrant.
/// Retourne (fingerprint, PresenceStatus) si signature valide.
pub fn parse_presence_event(
    homedir: &str,
    topic: &str,
    payload: &[u8],
) -> Result<(String, PresenceStatus), String>;
```

### Topics de présence

```
pgpilot/presence/{fp[0..16]}
```

Fingerprint tronqué à 16 chars pour limiter l'exposition de metadata tout en restant identifiable.

### Sécurité

La payload est signée avec la clé privée GPG de l'utilisateur. N'importe qui peut subscribre
au topic, mais falsifier une présence exigerait la clé privée de la victime.

Si la signature de présence est invalide → ignorer silencieusement + log debug.

**Commit** : `feat(chat): signed presence publish/subscribe`

---

## T6.2 — Module ACK

**Complexité** : M
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T4.3

### Fichier : `src/chat/ack.rs`  (ou dans `src/chat/mod.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AckStatus {
    Pending,
    Received,
    DecryptFailed,
}

#[derive(Serialize, Deserialize)]
struct AckPayload {
    msg_id: String,
    from_fp: String,
    status: String,      // "received" | "decrypt_failed"
    ts: i64,
    signature: String,   // sig sur "{msg_id}|{from_fp}|{status}|{ts}"
}

/// Publie un ACK après déchiffrement réussi.
pub async fn publish_ack(
    client: &MqttClient,
    homedir: &str,
    sender_fp: &str,
    msg_id: &str,
    status: AckStatus,
) -> Result<(), String>;

/// Subscribe au topic ACK d'un message envoyé.
pub async fn subscribe_ack(
    client: &MqttClient,
    msg_id: &str,
) -> Result<(), String>;

/// Parse et vérifie un ACK entrant.
pub fn parse_ack(
    homedir: &str,
    payload: &[u8],
) -> Result<AckPayload, String>;
```

### Topics ACK

```
pgpilot/ack/{msg_id[0..16]}
```

### Cycle de vie

1. Alice envoie → `publish_reliable` sur le topic chat + `subscribe_ack(msg_id)`
2. Alice initialise `acks: HashMap` avec `{bob_fp: Pending, carol_fp: Pending}`
3. Bob reçoit + déchiffre → `publish_ack(msg_id, status: Received)`
4. Alice reçoit ACK → `parse_ack` → si signature valide → `acks[bob_fp] = Received`
5. Si Carol hors ligne : `acks[carol_fp]` reste `Pending` indéfiniment

**Commit** : `feat(chat): signed ACK publish/subscribe`

---

## T6.3 — Intégration présence + ACK dans `App`

**Complexité** : M
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T6.1, T6.2, T5.4

### Modifications `src/app/chat.rs`

**`on_mqtt_event`** : router les events MQTT entrants :
```rust
fn on_mqtt_event(&mut self, event: MqttEvent) -> Task<Message> {
    match event {
        MqttEvent::Connected => {
            // 1. Publier présence "online"
            // 2. Subscribe à tous les topics des rooms actives
            // 3. Subscribe aux topics de présence des participants
        }
        MqttEvent::MessageReceived { topic, payload } => {
            if topic.starts_with("pgpilot/chat/") {
                // déchiffrer → Message::ChatReceived
            } else if topic.starts_with("pgpilot/presence/") {
                // parser présence → Message::PresenceUpdated
            } else if topic.starts_with("pgpilot/ack/") {
                // parser ACK → Message::ChatAckReceived
            }
        }
        MqttEvent::Disconnected => {
            self.mqtt_connected = false;
            // UI bannière déconnexion
        }
    }
}
```

**`on_presence_updated`** :
```rust
fn on_presence_updated(&mut self, fp: String, status: PresenceStatus) -> Task<Message> {
    self.presence.insert(fp, status);
    Task::none()
}
```

**`on_chat_ack_received`** :
```rust
fn on_chat_ack_received(&mut self, room_id: String, msg_id: String, sender_fp: String) -> Task<Message> {
    if let Some(messages) = self.chat_messages.get_mut(&room_id) {
        if let Some(msg) = messages.iter_mut().find(|m| m.id == msg_id) {
            msg.acks.insert(sender_fp, AckStatus::Received);
        }
    }
    Task::none()
}
```

**Commit** : `feat(chat): integrate presence and ACK into App update loop`

---

## Fichiers créés / modifiés

```
src/chat/presence.rs     (nouveau)
src/chat/ack.rs          (nouveau ou dans mod.rs)
src/app/chat.rs          (+ on_mqtt_event router, on_presence_updated, on_chat_ack_received)
src/app/mod.rs           (+ Message::PresenceUpdated, Message::ChatAckReceived)
```

## Critères d'acceptation

- [ ] Deux instances PGPilot → chacune voit l'autre ● en ligne
- [ ] Fermeture d'une instance → l'autre la voit ○ hors ligne (LWT déclenché)
- [ ] Présence avec signature invalide → ignorée silencieusement
- [ ] Message envoyé → indicateur ✓ Bob s'affiche après réception
- [ ] ACK avec signature invalide → ignoré
- [ ] `cargo build` ✓
