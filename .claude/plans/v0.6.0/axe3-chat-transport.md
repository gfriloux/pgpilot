# Axe 3 — Transport MQTT

## Objectif

Implémenter la couche réseau MQTT : connexion, subscribe/publish, reconnexion automatique,
intégration dans la boucle iced. Aucune logique métier ici — uniquement le tuyau.

**Référence** : `axe2-spec-finale.md` (lire avant de commencer)

---

## T3.1 — Dépendance Cargo + module skeleton

**Complexité** : S
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T2.3 (spec finale)

### Ce qui est à faire

**`Cargo.toml`** — ajouter :
```toml
rumqttc = { version = "0.24", features = ["use-rustls"] }
uuid = { version = "1", features = ["v4"] }
base64 = "0.22"
```

**`src/chat/mod.rs`** — skeleton vide :
```rust
pub mod mqtt;
pub mod rooms;
pub mod crypto;
pub mod presence;

pub use mqtt::MqttClient;
pub use rooms::{Room, load_rooms, save_rooms};
pub use crypto::{encrypt_for_room, decrypt_message};
pub use presence::PresenceStatus;
```

**Commit** : `feat(chat): add rumqttc dependency and module skeleton`

---

## T3.2 — Client MQTT

**Complexité** : L
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T3.1

### Fichier : `src/chat/mqtt.rs`

```rust
use rumqttc::{AsyncClient, MqttOptions, EventLoop, QoS, TlsConfiguration, Transport};
use tokio::sync::mpsc;

pub struct MqttClient {
    client: AsyncClient,
    relay: String,
}

/// Événements entrants depuis le broker vers l'app
pub enum MqttEvent {
    Connected,
    Disconnected,
    MessageReceived { topic: String, payload: Vec<u8> },
}

impl MqttClient {
    /// Crée la connexion TLS vers le broker. Retourne (client, receiver channel).
    pub async fn connect(
        relay: &str,
        client_id: &str,  // fingerprint tronqué à 23 chars (limite MQTT)
    ) -> Result<(Self, mpsc::UnboundedReceiver<MqttEvent>), String>;

    /// Subscribe à un topic (QoS selon spec axe2)
    pub async fn subscribe(&self, topic: &str) -> Result<(), String>;

    /// Publish un payload sur un topic
    pub async fn publish(&self, topic: &str, payload: &[u8], retained: bool) -> Result<(), String>;

    /// Publish avec QoS 1 (messages chat — au-moins-une-fois)
    pub async fn publish_reliable(&self, topic: &str, payload: &[u8]) -> Result<(), String>;
}
```

### Intégration iced

La boucle MQTT tourne en `tokio::spawn` séparé et envoie des `MqttEvent` via un channel
`mpsc::UnboundedSender<MqttEvent>`. L'app iced lit ces events via `App::subscription` :

```rust
// src/app/mod.rs
fn subscription(&self) -> Subscription<Message> {
    // Wraps le receiver MQTT comme un iced Subscription
    // Chaque MqttEvent devient un Message::MqttEvent(...)
}
```

Détail d'implémentation : utiliser `iced::subscription::channel` ou `iced::subscription::unfold`
selon la version iced 0.14 — vérifier l'API exacte.

### Reconnexion

`rumqttc` gère la reconnexion automatiquement via `EventLoop`. En cas de `ConnectionError`,
publier `Message::MqttDisconnected` vers l'UI, continuer le loop, publier `Message::MqttConnected`
quand réétabli. Pas de retry manuel — confier à rumqttc.

### Last Will Testament

À la connexion, configurer le LWT :
```rust
options.set_last_will(LastWill::new(
    format!("pgpilot/presence/{}", fp_short),
    b"offline".to_vec(),
    QoS::AtLeastOnce,
    true, // retained
));
```

**Commit** : `feat(chat): MQTT client with TLS, subscribe/publish, reconnect, LWT`

---

## T3.3 — Tests transport

**Complexité** : M
**Agent** : `voltagent-qa-sec:test-automator`
**Dépendances** : T3.2

### Stratégie

Utiliser un broker MQTT embarqué pour les tests (`mqtt-broker` crate ou spawn d'un processus
`mosquitto` local si disponible dans le dev shell).

Alternative : mock du `AsyncClient` avec un trait.

```rust
// tests/chat_mqtt.rs

#[tokio::test]
#[ignore] // nécessite broker MQTT disponible
async fn mqtt_connect_and_publish() {
    // Connexion au broker local de test
    // Publish sur un topic
    // Subscribe et vérifier réception
}

#[tokio::test]
async fn mqtt_client_id_truncated_to_23_chars() {
    // Vérifier que le client_id respecte la limite MQTT
    let fp = "A".repeat(40);
    let client_id = MqttClient::make_client_id(&fp);
    assert!(client_id.len() <= 23);
}

#[tokio::test]
async fn wire_message_serialization_roundtrip() {
    let msg = WireMessage {
        id: "test-uuid".to_string(),
        sender: "A".repeat(40),
        ts: 1746360000,
        payload: "-----BEGIN PGP MESSAGE-----\ntest\n-----END PGP MESSAGE-----".to_string(),
        signature: "-----BEGIN PGP SIGNATURE-----\ntest\n-----END PGP SIGNATURE-----".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: WireMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.id, msg.id);
    assert_eq!(decoded.sender, msg.sender);
}
```

**Commit** : `test(chat): MQTT transport unit tests`

---

## Fichiers créés / modifiés

```
Cargo.toml               (+ rumqttc, uuid, base64)
src/chat/mod.rs          (skeleton)
src/chat/mqtt.rs         (nouveau)
tests/chat_mqtt.rs       (nouveau)
```

## Critères d'acceptation

- [ ] `cargo build` ✓ avec `rumqttc`
- [ ] `MqttClient::connect` établit une connexion TLS vers `test.mosquitto.org:8883`
- [ ] Publish + subscribe fonctionnels manuellement (test cargo run)
- [ ] LWT configuré à la connexion
- [ ] Reconnexion automatique sans crash de l'UI
- [ ] `cargo test --test chat_mqtt` ✓
