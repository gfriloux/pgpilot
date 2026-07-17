# Axe 2 — Spec API interne chat (T2.2)

Document de référence pour les contrats de données du système de chat PGPilot.
Aucun code dans ce document — uniquement des décisions de conception avec justifications.

---

## 1. Format du message sur le wire (JSON via MQTT)

### Structure retenue

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMessage {
  pub id: String,        // UUID v4 — identifiant global unique du message
  pub sender: String,    // fingerprint 40 hex — émetteur déclaré
  pub ts: i64,           // Unix timestamp en secondes UTC
  pub payload: String,   // "-----BEGIN PGP MESSAGE-----\n..." (chiffré pour les destinataires)
  pub signature: String, // "-----BEGIN PGP SIGNATURE-----\n..." (signature détachée)
}
```

### Décision 1.1 — Ce que la signature couvre

**La signature couvre la concaténation canonique `id || sender || ts || payload`, pas uniquement le payload.**

Justification : signer seulement le payload permet à un attaquant de réutiliser une signature valide sur un message différent en changeant `id`, `sender` ou `ts` (replay attack / message substitution). En signant l'ensemble des champs publics, on garantit l'intégrité du message complet : un attaquant ne peut pas modifier l'émetteur déclaré, le timestamp ni l'identifiant sans invalider la signature.

Canonisation : les 4 champs sont concaténés avec un séparateur non ambigu avant signature :
```
"pgpilot-msg\x00" + id + "\x00" + sender + "\x00" + ts_as_decimal_string + "\x00" + payload
```
Le préfixe fixe `"pgpilot-msg\x00"` évite la confusion avec d'autres contextes de signature GPG dans la même application (ex. : `sign_file`).

### Décision 1.2 — Inclure `recipients_fps` dans le wire message ?

**Non — `recipients_fps` n'est pas inclus dans le wire message.**

Justification : publier les fingerprints des destinataires dans un topic MQTT (potentiellement public ou logué par le broker) constitue une fuite de métadonnées sérieuse : elle révèle qui communique avec qui à quiconque peut lire le topic, y compris l'opérateur du broker. L'information `pour qui ce message est chiffré` est déjà encodée dans les en-têtes PGP du payload chiffré (session keys encryptées par clef publique de chaque destinataire) — GPG peut déchiffrer sans liste explicite. Le léger inconfort opérationnel (pas de routing côté broker) est acceptable pour un système de chat éphémère.

### Décision 1.3 — Taille maximale d'un message

**Limite stricte : 65 536 octets (64 Kio) pour le `WireMessage` sérialisé en JSON.**

Justification : un message PGP textuel chiffré pour 5 destinataires avec un texte de 10 000 caractères génère environ 15 Kio de payload armored. 64 Kio couvre confortablement les usages légitimes (texte long, petite pièce jointe encodée en base64 dans le texte) sans permettre l'utilisation du broker comme transport de fichiers non contrôlé. La validation se fait côté émetteur avant publication et côté récepteur avant déchiffrement — les messages dépassant la limite sont rejetés avec `ChatError::MessageTooLarge`.

Le broker MQTT doit être configuré avec `max_packet_size = 131072` (128 Kio) pour absorber le framing MQTT autour du JSON.

---

## 2. Topics MQTT

### Structure retenue

```
pgpilot/chat/{SHA256(room_id)[0..16]}        → messages du salon (QoS 1, no retain)
pgpilot/presence/{fingerprint[0..16]}        → présence (QoS 0, retained, LWT)
pgpilot/ack/{msg_id[0..16]}                  → accusés de réception (QoS 0, no retain)
```

### Décision 2.1 — Fingerprint complet ou tronqué dans les topics présence

**Fingerprint tronqué aux 16 premiers caractères hexadécimaux (64 bits) dans les topics.**

Justification : le fingerprint complet (40 hex = 160 bits) dans un topic MQTT est visible en clair dans les logs du broker et dans les outils de monitoring réseau. Les 16 premiers caractères offrent 2^64 valeurs — suffisant pour éviter les collisions accidentelles dans un salon de quelques dizaines de participants, tout en réduisant significativement la quantité d'information identifiable exposée. Le fingerprint complet reste présent dans les champs `sender` du `WireMessage` signé — il n'est jamais perdu pour la vérification cryptographique.

Note : le `room_id` est lui aussi tronqué (SHA256 des 16 premiers octets) pour que le nom du salon ne soit pas reconstituable depuis le topic par un observateur extérieur.

### Décision 2.2 — QoS par type de topic

| Topic | QoS | Justification |
|-------|-----|---------------|
| `pgpilot/chat/...` | **QoS 1** | At-least-once delivery garantit que les messages ne sont pas silencieusement perdus sur une connexion instable. La déduplication côté récepteur (par `id` UUID) gère les doublons éventuels. QoS 2 (exactly-once) ajoute un round-trip supplémentaire pour un gain nul : le chiffrement PGP rend les doublons inoffensifs. |
| `pgpilot/presence/...` | **QoS 0** | Fire-and-forget suffit : la présence est mise à jour régulièrement (heartbeat toutes les 30 s) et le LWT (Last Will and Testament) gère la déconnexion. Perdre un ping de présence est négligeable. |
| `pgpilot/ack/...` | **QoS 0** | Un ACK perdu n'est pas grave : la seule conséquence est qu'un message reste marqué `AckStatus::Pending` un peu plus longtemps. Relancer l'ACK à chaque reconnexion couvrirait ce cas si nécessaire. |

### Décision 2.3 — Retained flag sur les messages chat

**Non — le retained flag est désactivé sur `pgpilot/chat/...`.**

Justification : retenir le dernier message dans le broker expose ce message chiffré (même si illisible) sur le serveur de façon persistante, et un nouveau participant voit arriver un unique message sorti de contexte qui ne lui apporte rien (il ne peut probablement pas le déchiffrer si la session key n'est pas pour lui). Le chat PGPilot est éphémère par conception : les messages vivent en RAM, pas sur le broker. Un participant qui arrive après l'émission d'un message ne le voit pas — c'est intentionnel et cohérent avec le modèle de sécurité.

Le topic de présence utilise bien `retain = true` pour que le statut Online/Offline soit immédiatement visible aux participants qui se connectent après.

---

## 3. Format du join code

### Structure retenue

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinCode {
  pub room_id: String,           // UUID v4
  pub relay: String,             // "mqtts://test.mosquitto.org:8883" — TLS obligatoire
  pub invited_by: String,        // fingerprint 40 hex
  pub room_name: Option<String>, // hint local uniquement, non authentifié
  pub sig: String,               // "-----BEGIN PGP SIGNATURE-----\n..." sur room_id+relay+invited_by
}
// Encodage : serde_json → bytes → base64url (sans padding) → préfixe "pgpilot:join:"
// Exemple : pgpilot:join:eyJyb29tX2lkIjoiN2YzYTJiNDEtMTIzNC01Njc4...
```

### Décision 3.1 — Inclure une signature dans le join code

**Oui — le join code contient une signature GPG de `invited_by` sur `room_id || relay || invited_by`.**

Justification : sans signature, n'importe qui peut forger un join code prétendant être émis par Alice pour diriger des victimes vers un broker malveillant. La signature permet au destinataire de vérifier, avant de se connecter au broker, que l'invitation provient bien de l'utilisateur dont il possède la clef publique dans son keyring. La vérification utilise la même infrastructure GPG que le reste de l'application (`verify_signature` / `gpg --verify`). Si la clef de `invited_by` est absente du keyring local, le join code est rejeté avec `ChatError::UnknownSender` — l'utilisateur doit d'abord importer la clef publique de l'invitant.

`room_name` n'est pas signé car il est un simple hint d'affichage local et n'a pas d'impact sécurité.

### Décision 3.2 — Expiration du join code

**Le join code est permanent (pas d'expiration intégrée).**

Justification : un système d'expiration nécessiterait soit un timestamp dans le code (facile à manipuler sans mécanisme de révocation), soit une infrastructure de révocation (hors de portée pour un chat éphémère). La sécurité repose sur la possession de la clef privée de `invited_by` : si l'on fait confiance à Alice pour signer l'invitation, un join code signé d'Alice reste valide aussi longtemps qu'Alice ne révoque pas sa clef. Pour retirer l'accès à un participant, on le retire de la liste des destinataires dans le keyring local — les nouveaux messages ne seront plus chiffrés pour lui. Les anciens messages (éphémères, en RAM) sont déjà partis.

---

## 4. Format `rooms.yaml`

### Structure retenue

```yaml
rooms:
  - id: "7f3a2b41-1234-5678-abcd-ef0123456789"
    name: "salon-pgp"
    relay: "mqtts://test.mosquitto.org:8883"
    participants:
      - fp: "ALICE00000000000000000000000000000000000A"
        joined_at: "2026-05-05T10:00:00Z"
      - fp: "BOB000000000000000000000000000000000000B"
        joined_at: "2026-05-05T10:05:00Z"
    created_at: "2026-05-05T10:00:00Z"
    my_fp: "ALICE00000000000000000000000000000000000A"
```

### Types Rust correspondants

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomParticipant {
  pub fp: String,         // fingerprint 40 hex
  pub joined_at: String,  // RFC 3339
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
  pub id: String,                          // UUID v4
  pub name: String,                        // nom local
  pub relay: String,                       // URL MQTT (mqtts://)
  pub participants: Vec<RoomParticipant>,  // participants connus
  pub created_at: String,                  // RFC 3339
  pub my_fp: String,                       // fingerprint de notre identité dans ce salon
}
```

### Décision 4.1 — Stocker `last_seen` par participant

**Non — `last_seen` n'est pas persisté dans `rooms.yaml`.**

Justification : `last_seen` est une information de présence volatile qui change à chaque session. La persister dans `rooms.yaml` implique des écritures fréquentes sur disque (à chaque ping de présence reçu) et crée un fichier qui grossit en information de surveillance locale. La présence est gérée en RAM dans `HashMap<String, PresenceStatus>` et perdue à la fermeture — cohérent avec le modèle éphémère. Si l'utilisateur veut savoir qui était en ligne, il observe l'UI en temps réel.

### Décision 4.2 — Stocker `joined_at` par participant

**Oui — `joined_at` est persisté par participant dans `rooms.yaml`.**

Justification : `joined_at` est une information stable écrite une seule fois (quand le participant rejoint le salon) et ne change plus. Elle permet d'afficher un ordre d'arrivée dans l'UI et de comprendre le contexte d'un salon au démarrage sans trafic réseau. Le coût est négligeable (une ligne par participant).

### Décision 4.3 — Le nom est-il local ou partagé dans le join code

**Le nom est local uniquement** — il est transmis dans `JoinCode.room_name` comme hint non authentifié que chaque client peut librement renommer.

Justification : un nom de salon partagé et immuable nécessiterait un mécanisme de consensus ou de maître — inutile ici. Chaque utilisateur nomme ses salons comme il l'entend dans son `rooms.yaml`. Le `room_id` (UUID) est le seul identifiant canonique et partagé. Le champ `my_fp` dans `rooms.yaml` permet à l'application de savoir quelle identité locale utiliser dans ce salon si l'utilisateur possède plusieurs clefs privées.

---

## 5. Struct `ChatMessage` en RAM

### Structure retenue

```rust
#[derive(Debug, Clone)]
pub struct ChatMessage {
  pub id: String,                           // UUID v4 du WireMessage original
  pub sender_fp: String,                    // fingerprint 40 hex
  pub text: String,                         // texte en clair après déchiffrement
  pub ts: chrono::DateTime<chrono::Utc>,    // timestamp du WireMessage
  pub received_at: chrono::DateTime<chrono::Utc>, // timestamp de réception locale
  pub acks: HashMap<String, AckStatus>,     // fp → AckStatus
  pub direction: MessageDirection,           // Sent | Received
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AckStatus {
  Pending,
  Received,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageDirection {
  Sent,
  Received,
}
```

Le champ `received_at` est ajouté par rapport à la structure initiale pour distinguer le timestamp de l'émetteur (potentiellement falsifié) du moment réel de réception locale — utile pour détecter des messages antidatés.

### Décision 5.1 — Borner le Vec<ChatMessage> par room

**Oui — limite de 500 messages par salon en RAM, politique FIFO.**

Justification : un salon sans limite peut accumuler des messages indéfiniment pendant une session longue, augmentant la consommation mémoire et le temps de rendu de la liste. 500 messages correspond à environ 2 à 4 heures de conversation active — au-delà, les messages les plus anciens sont supprimés en tête de `Vec`. Le dépassement de la limite est géré dans `on_chat_received` : si `messages.len() >= 500`, on retire `messages[0]` avant d'ajouter le nouveau. Ce comportement est transparent pour l'utilisateur (les messages anciens disparaissent du haut de la fenêtre, comme dans tout chat éphémère).

### Décision 5.2 — Conserver les messages non déchiffrables

**Non — les messages non déchiffrables sont silencieusement ignorés.**

Justification : si un message ne peut pas être déchiffré (clef absente du keyring, payload corrompu, mauvais destinataire), l'afficher comme "message illisible" crée de la confusion et révèle une information de métadonnée (quelqu'un a envoyé quelque chose). Comme le système n'offre pas de mécanisme de récupération a posteriori (pas de stockage persistant), afficher un placeholder ne sert à rien. L'échec est loggué en interne avec `ChatError::DecryptFailed` pour le débogage mais n'est pas présenté à l'UI. Exception : si la signature est invalide sur un message déchiffrable, `ChatError::SignatureInvalid` est émis et le message est traité comme s'il n'existait pas — on ne présente jamais de texte dont l'authenticité ne peut être vérifiée.

---

## 6. Format ACK sur le wire

### Structure retenue

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireAck {
  pub msg_id: String,  // UUID du WireMessage original
  pub from: String,    // fingerprint 40 hex du confirmant
  pub ts: i64,         // timestamp de réception (secondes UTC)
}
```

### Décision 6.1 — L'ACK doit-il être signé

**Non — les ACK ne sont pas signés.**

Justification : le coût de signature GPG (appel subprocess, attente pinentry potentielle) est prohibitif pour une opération qui doit être déclenchée automatiquement à chaque message reçu. Un ACK forgé par un tiers ne cause aucun dommage sécurité : l'attaquant peut au pire faire croire à l'émetteur qu'un destinataire a reçu le message alors qu'il ne l'a pas — c'est un problème de confort, pas de confidentialité. Le `msg_id` est un UUID non devinable ce qui limite l'utilité d'un ACK forgé aléatoirement. Si l'authenticité des ACK devient un besoin (ex. : comptabilisation de livraison), elle sera ajoutée dans une version ultérieure avec un mécanisme de signature batch.

---

## 7. Type d'erreur `ChatError`

### Structure retenue

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ChatError {
  // Connectivité MQTT
  MqttNotConnected,
  BrokerUnreachable(String),    // URL du broker
  MqttProtocolError(String),    // erreur de niveau protocole MQTT
  TlsError(String),             // échec TLS (certificat, handshake)

  // Cryptographie
  EncryptFailed(String),        // échec de chiffrement GPG
  DecryptFailed(String),        // clef absente, payload corrompu
  SignatureInvalid,             // vérification de signature échouée
  SignFailed(String),           // échec de signature (ex: clef protégée par passphrase)

  // Identité et rooms
  UnknownSender,                // fingerprint absent du keyring local
  RoomNotFound(String),         // room_id inconnu dans rooms.yaml
  NoSigningKey,                 // aucune clef privée utilisable pour ce salon
  ParticipantNotInKeyring(String), // fingerprint d'un participant sans clef publique locale

  // Join code
  InvalidJoinCode,              // base64 invalide, JSON malformé, champs manquants
  JoinCodeSignatureInvalid,     // signature du join code ne correspond pas à invited_by
  JoinCodeInviterUnknown,       // clef de invited_by absente du keyring

  // Validation de message
  MessageTooLarge,              // WireMessage JSON dépasse 65 536 octets
  MessageIdDuplicate(String),   // UUID déjà vu dans cette session (replay)
  MalformedWireMessage(String), // JSON invalide ou champ manquant
  InvalidFingerprint(String),   // fingerprint sender ne passe pas validate_fp()

  // Persistance
  RoomsYamlLoadFailed(String),  // fichier rooms.yaml illisible ou malformé
  RoomsYamlSaveFailed(String),  // impossible d'écrire rooms.yaml
}
```

### Correspondance avec StatusKind

Tous les variants `ChatError` se mappent sur `StatusKind::Error` dans l'UI. Le texte affiché est produit par `impl std::fmt::Display for ChatError` qui fournit un message localisable via le trait `Strings`. Les erreurs de connectivité (`MqttNotConnected`, `BrokerUnreachable`) déclenchent en plus un indicateur de statut MQTT dans la sidebar.

---

## 8. Constantes et paramètres de configuration

Valeurs fixes définies dans `src/chat/mod.rs` :

```rust
pub const CHAT_TOPIC_PREFIX: &str = "pgpilot/chat";
pub const PRESENCE_TOPIC_PREFIX: &str = "pgpilot/presence";
pub const ACK_TOPIC_PREFIX: &str = "pgpilot/ack";

pub const MAX_WIRE_MESSAGE_BYTES: usize = 65_536;  // 64 Kio
pub const MAX_MESSAGES_PER_ROOM: usize = 500;
pub const PRESENCE_HEARTBEAT_SECS: u64 = 30;
pub const PRESENCE_LWT_TIMEOUT_SECS: u16 = 90;    // 3 × heartbeat avant LWT
pub const MQTT_KEEPALIVE_SECS: u16 = 60;
pub const MQTT_RECONNECT_BASE_MS: u64 = 1_000;    // backoff exponentiel, base 1 s
pub const MQTT_RECONNECT_MAX_MS: u64 = 60_000;    // plafond 60 s

pub const SIGN_CANONICAL_PREFIX: &[u8] = b"pgpilot-msg\x00";
pub const JOIN_CODE_PREFIX: &str = "pgpilot:join:";
```

---

## 9. Récapitulatif des décisions

| # | Question | Décision |
|---|----------|----------|
| 1.1 | Portée de la signature | `id + sender + ts + payload` avec préfixe canonique |
| 1.2 | `recipients_fps` dans le wire | Non — fuite de métadonnées inacceptable |
| 1.3 | Taille max message | 64 Kio (JSON sérialisé) |
| 2.1 | Fingerprint dans topic | Tronqué à 16 hex chars |
| 2.2 | QoS messages/ack/présence | QoS 1 / QoS 0 / QoS 0 |
| 2.3 | Retained sur messages chat | Non — chat éphémère par conception |
| 3.1 | Signature dans join code | Oui — authentification de l'invitation obligatoire |
| 3.2 | Expiration join code | Permanent — révocation via révocation de clef GPG |
| 4.1 | `last_seen` dans rooms.yaml | Non — donnée volatile, RAM uniquement |
| 4.2 | `joined_at` dans rooms.yaml | Oui — stable, écrit une fois |
| 4.3 | Nom local ou partagé | Local uniquement, `room_id` UUID est l'identifiant canonique |
| 5.1 | Borne Vec<ChatMessage> | 500 messages par salon, FIFO |
| 5.2 | Conserver messages illisibles | Non — ignorés silencieusement |
| 6.1 | ACK signé | Non — coût prohibitif, impact sécurité nul |
