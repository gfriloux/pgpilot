# Axe 2 — Architecture chat

## Objectif

Produire une **spec technique complète** du système de chat avant tout code. Deux agents
travaillent en parallèle : un architecte analyse les modules et interfaces, un API designer
définit les contrats internes. Leurs outputs fusionnent en un document de spec qui sert de
référence pour les axes 3–8.

Aucun code n'est écrit dans cet axe. Uniquement des documents de spec.

---

## T2.1 — Architecture modules

**Complexité** : M
**Agent** : `voltagent-qa-sec:architect-reviewer`
**Dépendances** : aucune (lire le codebase existant)

### Contexte à fournir à l'agent

- Lire `src/app/mod.rs` (structure App, Message enum, update router)
- Lire `src/gpg/mod.rs` + `src/gpg/keyring.rs` (pattern blocking_task, gpg_command)
- Lire `src/ui/mod.rs` + `src/ui/key_list.rs` (pattern master-detail existant)
- Lire `src/config/mod.rs` (pattern Config load/save YAML)
- Lire `CLAUDE.md` sections "Architecture", "State flow", "Module layout"

### Ce qui est à produire

**1. Layout des nouveaux modules**

```
src/
├── chat/
│   ├── mod.rs        — re-exports, ChatState, ChatMessage, Room structs
│   ├── rooms.rs      — load/save rooms.yaml, Room CRUD
│   ├── mqtt.rs       — connexion MQTT, subscribe/publish, reconnect loop
│   ├── crypto.rs     — encrypt_for_room(), decrypt_message(), verify_sender()
│   └── presence.rs   — PresenceState, publish_online(), handle_presence_event()
```

Pour chaque fichier : liste des types publics, fonctions publiques, dépendances entre fichiers.

**2. Intégration dans `App`**

Quels champs ajouter à la struct `App` :
- `chat_state: Option<ChatState>` ou `rooms: Vec<Room>` ?
- `active_room: Option<String>` (room_id) ?
- `chat_messages: HashMap<String, Vec<ChatMessage>>` (room_id → messages RAM) ?
- `presence: HashMap<String, PresenceStatus>` (fingerprint → Online/Offline) ?
- `mqtt_connected: bool` ?

**3. Nouveaux `View` variants**

```rust
View::ChatList          // liste des rooms
View::ChatRoom(String)  // room_id actif
```

Vérifier cohérence avec `previous_view` et `on_nav_changed`.

**4. Nouveaux `Message` variants**

Liste exhaustive des messages nécessaires :
```rust
// Rooms
Message::ChatRoomCreate(String)           // nom local
Message::ChatRoomJoin(String)             // join code base64
Message::ChatRoomSelected(String)         // room_id
Message::ChatRoomsLoaded(Vec<Room>)

// Envoi / réception
Message::ChatSend(String, String)         // room_id, text
Message::ChatReceived(String, ChatMessage) // room_id, message déchiffré
Message::ChatDecryptFailed(String, String) // room_id, msg_id

// MQTT infra
Message::MqttConnected
Message::MqttDisconnected
Message::MqttReconnecting

// Présence
Message::PresenceUpdated(String, PresenceStatus) // fingerprint, status

// ACK
Message::ChatAckReceived(String, String, String) // room_id, msg_id, sender_fp
```

**5. Découpage handlers dans `app/`**

```
src/app/chat.rs  — on_chat_room_create, on_chat_send, on_chat_received,
                   on_mqtt_connected, on_presence_updated, on_chat_ack_received
```

**6. Réutilisation du code existant**

- `blocking_task()` pour encrypt/decrypt (même pattern que gpg ops)
- `Config` : étendre pour stocker `mqtt_relay: Option<String>` ?
- `gnupg_dir()` : réutilisé dans `chat/crypto.rs`
- Sequoia déjà disponible pour chiffrement/signature

**7. Risques architecturaux identifiés**

- MQTT tourne en tâche async de fond — comment l'intégrer avec iced subscriptions ?
  (Option A : `App::subscription` retourne un stream MQTT ; Option B : channel tokio → Task)
- Reconnexion automatique MQTT sans bloquer l'UI
- `HashMap<room_id, Vec<ChatMessage>>` en RAM → pas de sérialisation → taille bornée ?

**Commit** : aucun — livrable = `axe2-spec-modules.md` dans `.claude/plans/v0.6.0/`

---

## T2.2 — Design API interne

**Complexité** : M
**Agent** : `voltagent-core-dev:api-designer`
**Dépendances** : aucune (parallèle avec T2.1)

### Ce qui est à produire

**1. Format du message sur le wire (JSON)**

```rust
#[derive(Serialize, Deserialize)]
pub struct WireMessage {
    pub id: String,           // UUID v4
    pub sender: String,       // fingerprint 40 hex
    pub ts: i64,              // Unix timestamp secondes
    pub payload: String,      // "-----BEGIN PGP MESSAGE-----\n..."
    pub signature: String,    // "-----BEGIN PGP SIGNATURE-----\n..."
}
```

Questions à trancher :
- La signature couvre-t-elle le payload seul ou `{id+sender+ts+payload}` ? (recommandation + justification)
- Doit-on inclure `recipients_fps` dans le wire message ? (implications metadata)
- Taille max d'un message (pour éviter abus du broker) ?

**2. Topics MQTT**

```
pgpilot/chat/{SHA256(room_id)[0..16]}          → messages du salon
pgpilot/presence/{fingerprint[0..16]}          → présence (retained + LWT)
pgpilot/ack/{msg_id[0..16]}                    → accusés de réception
```

Questions à trancher :
- Utiliser le fingerprint complet ou tronqué dans les topics de présence ? (privacy vs lisibilité)
- QoS 0, 1 ou 2 pour les messages ? Pour les ACK ? Pour la présence ?
- Retained flag sur les messages chat ? (oui = dernier message visible aux nouveaux arrivants)

**3. Format du join code**

```rust
#[derive(Serialize, Deserialize)]
pub struct JoinCode {
    pub room_id: String,      // UUID v4
    pub relay: String,        // "mqtt://test.mosquitto.org:8883"
    pub invited_by: String,   // fingerprint 40 hex
    pub room_name: Option<String>, // hint local uniquement
}
// Encodage : serde_json → base64url → chaîne copiable
// Exemple : pgpilot:join:eyJyb29tX2lkIjoiN2YzYTJiNDEt...
```

**4. Format `rooms.yaml`**

```yaml
rooms:
  - id: "7f3a2b41-1234-5678-abcd-ef0123456789"
    name: "salon-pgp"
    relay: "mqtt://test.mosquitto.org:8883"
    participants:
      - "ALICE000000000000000000000000000000000000"
      - "BOB00000000000000000000000000000000000000"
    created_at: "2026-05-04T10:00:00Z"
```

Questions : stocker `last_seen` par participant ? Stocker un `joined_at` ?

**5. Struct `ChatMessage` en RAM**

```rust
pub struct ChatMessage {
    pub id: String,
    pub sender_fp: String,
    pub text: String,          // déchiffré, en clair
    pub ts: chrono::DateTime<Utc>,
    pub acks: HashMap<String, AckStatus>, // fp → Received/Pending
    pub direction: MessageDirection,       // Sent | Received
}
```

**6. Gestion des erreurs**

Définir un type `ChatError` exhaustif :
- `MqttNotConnected`
- `EncryptFailed(String)`
- `DecryptFailed(String)` — clé manquante, message corrompu
- `SignatureInvalid`
- `UnknownSender` — fingerprint absent du keyring
- `RoomNotFound(String)`
- `InvalidJoinCode`

**Commit** : aucun — livrable = `axe2-spec-api.md` dans `.claude/plans/v0.6.0/`

---

## T2.3 — Fusion et validation spec

**Complexité** : S
**Agent** : `voltagent-qa-sec:architect-reviewer`
**Dépendances** : T2.1, T2.2

### Ce qui est à faire

1. Lire `axe2-spec-modules.md` et `axe2-spec-api.md`
2. Identifier contradictions ou angles morts
3. Écrire `axe2-spec-finale.md` : document unique de référence pour axes 3–8
4. La spec finale doit répondre explicitement à :
   - Comment MQTT s'intègre avec la boucle iced (subscription vs channel) ?
   - Où vivent les clés de déchiffrement (pas en mémoire longtemps) ?
   - Que se passe-t-il si le broker est injoignable au démarrage ?
   - Comment tester le transport sans vrai broker (mock ou broker embarqué) ?

**Commit** : aucun — livrable = `axe2-spec-finale.md`

---

## Livrables de l'axe 2

```
.claude/plans/v0.6.0/
├── axe2-spec-modules.md   (T2.1)
├── axe2-spec-api.md       (T2.2)
└── axe2-spec-finale.md    (T2.3) ← référence pour axes 3–8
```

## Critères d'acceptation

- [ ] `axe2-spec-finale.md` existe et répond aux 4 questions de T2.3
- [ ] Layout des modules `src/chat/` défini avec types + fonctions publiques
- [ ] Liste exhaustive des `Message` variants définie
- [ ] Format WireMessage, JoinCode, rooms.yaml, ChatMessage définis
- [ ] Risques architecturaux documentés avec options choisies
