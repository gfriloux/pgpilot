# Axe 8 — Tests chat

## Objectif

Couvrir tous les modules chat par des tests unitaires et d'intégration. Stratégie identique
à l'axe 3 de v0.5.0 : vrais processus GPG, broker MQTT local pour les tests réseau.

---

## T8.1 — Tests `src/chat/rooms.rs`

**Complexité** : S
**Agent** : `voltagent-qa-sec:test-automator`
**Dépendances** : T4.1

```rust
// tests/chat_rooms.rs

#[test]
fn room_topic_is_opaque() {
    let room = Room::new("test".to_string(), "mqtt://localhost".to_string(), vec![]);
    let topic = room.topic();
    // Le topic ne contient pas le nom ni l'ID lisible
    assert!(!topic.contains("test"));
    assert!(topic.starts_with("pgpilot/chat/"));
    assert_eq!(topic.len(), "pgpilot/chat/".len() + 16);
}

#[test]
fn join_code_roundtrip() {
    let room = Room::new(
        "salon".to_string(),
        "mqtt://test.mosquitto.org:8883".to_string(),
        vec!["A".repeat(40), "B".repeat(40)],
    );
    let code = room.to_join_code(&"C".repeat(40));
    let recovered = from_join_code(&code).unwrap();
    assert_eq!(recovered.id, room.id);
    assert_eq!(recovered.relay, room.relay);
    assert_eq!(recovered.participants, room.participants);
}

#[test]
fn join_code_invalid_base64_returns_error() {
    assert!(from_join_code("not-valid-base64!!!").is_err());
}

#[test]
fn join_code_missing_fields_returns_error() {
    // JSON valide mais champs manquants
    let bad = base64::encode(r#"{"room_id": "abc"}"#);
    assert!(from_join_code(&format!("pgpilot:join:{bad}")).is_err());
}

#[test]
fn rooms_yaml_roundtrip() {
    use tempfile::TempDir;
    let dir = TempDir::new().unwrap();
    // Patch rooms_path() pour utiliser le tempdir
    let rooms = vec![
        Room::new("test".to_string(), "mqtt://localhost".to_string(),
                  vec!["A".repeat(40)]),
    ];
    save_rooms_to(&rooms, &dir.path().join("rooms.yaml")).unwrap();
    let loaded = load_rooms_from(&dir.path().join("rooms.yaml")).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].name, "test");
}

#[test]
fn room_participants_must_be_valid_fingerprints() {
    let result = Room::new(
        "test".to_string(),
        "mqtt://localhost".to_string(),
        vec!["not-a-fingerprint".to_string()],
    );
    // Room::new doit valider les fps ou on valide avant d'appeler
    // Selon choix arch : soit Room::new retourne Result, soit on valide ailleurs
}
```

**Commit** : `test(chat): room CRUD, join code roundtrip, rooms.yaml persistence`

---

## T8.2 — Tests `src/chat/crypto.rs`

**Complexité** : L
**Agent** : `voltagent-qa-sec:test-automator`
**Dépendances** : T4.2, T3.1 (fixtures GPG existantes)

```rust
// tests/chat_crypto.rs
mod common;
use common::{setup_test_gnupghome, import_armored};
use pgpilot::chat::crypto::*;
use pgpilot::fixtures;

#[test]
#[ignore] // nécessite GPG
fn encrypt_decrypt_roundtrip() {
    let (_dir, homedir) = setup_test_gnupghome();
    let sender_fp = import_armored(&homedir, fixtures::TEST_SECRET_KEY);
    let recipient_fp = import_armored(&homedir, fixtures::TEST_PUBLIC_KEY);

    let wire_msg = encrypt_for_room(
        &homedir,
        "Hello, comrade!",
        "test-room-id",
        &sender_fp,
        &[recipient_fp.clone()],
    ).unwrap();

    assert!(!wire_msg.payload.is_empty());
    assert!(!wire_msg.signature.is_empty());

    let (plaintext, verified_fp) = decrypt_message(&homedir, &wire_msg).unwrap();
    assert_eq!(plaintext, "Hello, comrade!");
    // Le fingerprint vérifié correspond au sender
    assert!(sender_fp.ends_with(&verified_fp) || verified_fp == sender_fp);
}

#[test]
#[ignore]
fn decrypt_tampered_payload_fails() {
    let (_dir, homedir) = setup_test_gnupghome();
    let sender_fp = import_armored(&homedir, fixtures::TEST_SECRET_KEY);
    let recipient_fp = import_armored(&homedir, fixtures::TEST_PUBLIC_KEY);

    let mut wire_msg = encrypt_for_room(
        &homedir, "Secret", "room", &sender_fp, &[recipient_fp],
    ).unwrap();

    // Tamper payload
    wire_msg.payload.push_str("TAMPERED");

    let result = decrypt_message(&homedir, &wire_msg);
    assert!(result.is_err());
}

#[test]
#[ignore]
fn verify_sender_rejects_wrong_fp() {
    let (_dir, homedir) = setup_test_gnupghome();
    let sender_fp = import_armored(&homedir, fixtures::TEST_SECRET_KEY);
    let recipient_fp = import_armored(&homedir, fixtures::TEST_PUBLIC_KEY);

    let mut wire_msg = encrypt_for_room(
        &homedir, "Hello", "room", &sender_fp, &[recipient_fp],
    ).unwrap();

    // Usurper le sender
    wire_msg.sender = "B".repeat(40);

    let result = verify_sender(&homedir, &wire_msg);
    assert!(matches!(result, Err(ChatError::SignatureInvalid)));
}

#[test]
#[ignore]
fn decrypt_fails_without_private_key() {
    let (_dir, homedir) = setup_test_gnupghome();
    let sender_fp = import_armored(&homedir, fixtures::TEST_SECRET_KEY);
    // N'importe pas la clé privée du destinataire
    let recipient_fp = import_armored(&homedir, fixtures::TEST_THIRD_PARTY_PUBLIC);

    let wire_msg = encrypt_for_room(
        &homedir, "Secret", "room", &sender_fp, &[recipient_fp],
    ).unwrap();

    // Nouveau homedir sans clé privée
    let (_dir2, homedir2) = setup_test_gnupghome();
    let result = decrypt_message(&homedir2, &wire_msg);
    assert!(matches!(result, Err(ChatError::DecryptFailed(_))));
}

#[test]
fn wire_message_too_large_rejected() {
    let large_payload = "X".repeat(600 * 1024); // 600 KiB > 512 KiB max
    let wire_msg = WireMessage {
        id: "test".to_string(),
        sender: "A".repeat(40),
        ts: 0,
        payload: large_payload,
        signature: String::new(),
    };
    // parse_wire_message doit rejeter
    let result = pgpilot::chat::crypto::parse_wire_message(
        &serde_json::to_vec(&wire_msg).unwrap()
    );
    assert!(matches!(result, Err(ChatError::MessageTooLarge)));
}
```

**Commit** : `test(chat): crypto encrypt/decrypt roundtrip, sender verification, size limit`

---

## T8.3 — Tests `src/chat/presence.rs`

**Complexité** : M
**Agent** : `voltagent-qa-sec:test-automator`
**Dépendances** : T6.1

```rust
// tests/chat_presence.rs

#[test]
#[ignore]
fn presence_payload_verify_valid_signature() {
    let (_dir, homedir) = setup_test_gnupghome();
    let fp = import_armored(&homedir, fixtures::TEST_SECRET_KEY);

    let payload = build_presence_payload(&homedir, &fp, PresenceStatus::Online).unwrap();
    let (verified_fp, status) = parse_presence_event(&homedir, "pgpilot/presence/ABCD", &payload).unwrap();
    assert_eq!(status, PresenceStatus::Online);
    assert_eq!(verified_fp, fp);
}

#[test]
fn presence_invalid_signature_is_rejected() {
    // Payload bien formé mais signature bricolée
    let payload = serde_json::to_vec(&serde_json::json!({
        "status": "online",
        "fp": "A".repeat(40),
        "ts": 1234567890_i64,
        "signature": "-----BEGIN PGP SIGNATURE-----\ninvalid\n-----END PGP SIGNATURE-----"
    })).unwrap();
    let (_dir, homedir) = setup_test_gnupghome();
    let result = parse_presence_event(&homedir, "pgpilot/presence/AAAA", &payload);
    assert!(result.is_err());
}

#[test]
fn presence_unknown_status_string_is_rejected() {
    // status: "away" n'existe pas
}

#[test]
fn presence_fp_mismatch_topic_vs_payload_is_rejected() {
    // topic = pgpilot/presence/AAAA mais payload.fp = B*40
}
```

**Commit** : `test(chat): presence signature verification, invalid payload rejection`

---

## T8.4 — Tests `src/chat/ack.rs`

**Complexité** : S
**Agent** : `voltagent-qa-sec:test-automator`
**Dépendances** : T6.2

```rust
// tests/chat_ack.rs

#[test]
#[ignore]
fn ack_payload_roundtrip() {
    let (_dir, homedir) = setup_test_gnupghome();
    let fp = import_armored(&homedir, fixtures::TEST_SECRET_KEY);
    let payload = build_ack_payload(&homedir, &fp, "msg-id-123", AckStatus::Received).unwrap();
    let parsed = parse_ack(&homedir, &payload).unwrap();
    assert_eq!(parsed.msg_id, "msg-id-123");
    assert_eq!(parsed.from_fp, fp);
    assert_eq!(parsed.status, "received");
}

#[test]
fn ack_invalid_signature_rejected() { /* ... */ }

#[test]
fn ack_status_mapping() {
    assert_eq!(AckStatus::Received.as_str(), "received");
    assert_eq!(AckStatus::DecryptFailed.as_str(), "decrypt_failed");
}
```

**Commit** : `test(chat): ACK build, parse, signature verification`

---

## T8.5 — Tests handlers `src/app/chat.rs`

**Complexité** : M
**Agent** : `voltagent-qa-sec:test-automator`
**Dépendances** : T4.3, T6.3

```rust
// tests/app_chat.rs

#[test]
fn room_added_to_app_state_after_create() {
    let (_dir, homedir) = setup_test_gnupghome();
    let mut app = make_test_app(&homedir);
    assert_eq!(app.rooms.len(), 0);
    // Simuler on_chat_room_create en appelant la logique directement
    // (sans Task async)
    app.rooms.push(Room::new("test".to_string(), "mqtt://localhost".to_string(), vec![]));
    assert_eq!(app.rooms.len(), 1);
}

#[test]
fn messages_bounded_at_500() {
    let (_dir, homedir) = setup_test_gnupghome();
    let mut app = make_test_app(&homedir);
    let room_id = "test-room".to_string();
    // Insérer 501 messages
    for i in 0..=500 {
        app.store_message(&room_id, make_test_message(i));
    }
    assert_eq!(app.chat_messages[&room_id].len(), 500);
}

#[test]
fn non_member_message_ignored() {
    let (_dir, homedir) = setup_test_gnupghome();
    let mut app = make_test_app(&homedir);
    let room = Room::new("test".to_string(), "mqtt://localhost".to_string(),
                         vec!["A".repeat(40)]);
    app.rooms.push(room.clone());

    // Message d'un fingerprint non membre
    let intruder_fp = "B".repeat(40);
    let msg = ChatMessage { sender_fp: intruder_fp, ..make_test_message(0) };

    // on_chat_received devrait ignorer ce message
    let count_before = app.chat_messages.get(&room.id).map(|v| v.len()).unwrap_or(0);
    app.store_if_member(&room.id, msg);
    let count_after = app.chat_messages.get(&room.id).map(|v| v.len()).unwrap_or(0);
    assert_eq!(count_before, count_after);
}

#[test]
fn ack_updates_message_status() {
    let (_dir, homedir) = setup_test_gnupghome();
    let mut app = make_test_app(&homedir);
    let room_id = "test-room".to_string();
    let msg = make_test_message(0);
    let msg_id = msg.id.clone();
    app.chat_messages.entry(room_id.clone()).or_default().push(msg);

    let bob_fp = "B".repeat(40);
    let _ = app.on_chat_ack_received(room_id.clone(), msg_id.clone(), bob_fp.clone());

    let messages = &app.chat_messages[&room_id];
    let updated = messages.iter().find(|m| m.id == msg_id).unwrap();
    assert_eq!(updated.acks[&bob_fp], AckStatus::Received);
}
```

**Commit** : `test(chat): App chat handlers — room create, message bounds, member filter, ACK update`

---

## Fichiers créés

```
tests/chat_rooms.rs
tests/chat_crypto.rs
tests/chat_presence.rs
tests/chat_ack.rs
tests/app_chat.rs
```

## Critères d'acceptation

- [ ] `cargo test --test chat_rooms` ✓
- [ ] `cargo test --test chat_crypto` ✓ (tests rapides)
- [ ] `cargo test --test chat_crypto -- --ignored` ✓ (tests GPG)
- [ ] `cargo test --test chat_presence` ✓
- [ ] `cargo test --test chat_ack` ✓
- [ ] `cargo test --test app_chat` ✓
- [ ] Aucun test ne touche le `$GNUPGHOME` réel
- [ ] Tests `#[ignore]` documentent pourquoi ils sont lents
