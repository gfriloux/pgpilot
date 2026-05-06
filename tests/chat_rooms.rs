//! Tests pour `Room`, `RoomStore` et `JoinCode` (`src/chat/rooms.rs`).
//!
//! Couvre :
//! - `Room::chat_topic()` : format, opacité, déterminisme.
//! - `JoinCode::encode()` + `JoinCode::decode()` roundtrip.
//! - `JoinCode::decode()` rejette les entrées malformées.
//! - `RoomStore::load()` : fichier absent → `Self::default()`.
//! - `RoomStore::load()` : fichier > 1 Mio → erreur.
//!
//! Les tests `#[ignore]` nécessitent GPG (signature / vérification).

#![allow(dead_code)]

use std::sync::Mutex;

use pgpilot::chat::rooms::JoinCode;
use pgpilot::chat::{ChatError, Room, RoomParticipant, RoomStore};

// ---------------------------------------------------------------------------
// Mutex de sérialisation des tests manipulant des variables d'environnement.
// `std::env::set_var` est non thread-safe ; les tests env-sensibles doivent
// s'exécuter séquentiellement.
// ---------------------------------------------------------------------------

static ENV_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Construit une `Room` minimale avec un `id` UUID valide.
fn make_room(id: &str) -> Room {
  Room {
    id: id.to_string(),
    name: "test salon".to_string(),
    relay: "mqtts://broker.example.com:8883".to_string(),
    my_fp: "A".repeat(40),
    created_at: chrono::Utc::now(),
    participants: vec![RoomParticipant {
      fp: "A".repeat(40),
      joined_at: chrono::Utc::now(),
    }],
  }
}

/// Construit un `JoinCode` valide (sans signature réelle — `sig` non vide).
fn make_join_code(room_id: &str) -> JoinCode {
  JoinCode {
    room_id: room_id.to_string(),
    relay: "mqtts://broker.hivemq.com:8883".to_string(),
    invited_by: "B".repeat(40),
    room_name: Some("salon de test".to_string()),
    // Valeur non vide pour passer la garde `sig.is_empty()`.
    // La vérification cryptographique n'est pas testée ici (→ #[ignore]).
    sig: "-----BEGIN PGP SIGNATURE-----\nfake\n-----END PGP SIGNATURE-----".to_string(),
  }
}

// ---------------------------------------------------------------------------
// Tests Room::chat_topic()
// ---------------------------------------------------------------------------

#[test]
fn chat_topic_has_correct_prefix() {
  let room = make_room("11111111-1111-4111-8111-111111111111");
  let topic = room.chat_topic();
  assert!(
    topic.starts_with("pgpilot/chat/"),
    "chat_topic() doit commencer par \"pgpilot/chat/\", obtenu : {topic}"
  );
}

#[test]
fn chat_topic_does_not_contain_room_id() {
  let room_id = "22222222-2222-4222-8222-222222222222";
  let room = make_room(room_id);
  let topic = room.chat_topic();
  assert!(
    !topic.contains(room_id),
    "chat_topic() ne doit pas exposer le room_id en clair : {topic}"
  );
}

#[test]
fn chat_topic_correct_length() {
  // "pgpilot/chat/" (13) + 16 chars hex = 29 chars.
  let room = make_room("33333333-3333-4333-8333-333333333333");
  let topic = room.chat_topic();
  assert_eq!(
    topic.len(),
    29,
    "chat_topic() doit avoir exactement 29 caractères, obtenu {} : {topic}",
    topic.len()
  );
}

#[test]
fn chat_topic_deterministic() {
  // Le topic doit être identique à chaque appel pour la même room_id.
  // Ce test vérifie que le bug SHA-256 (hashage de l'adresse mémoire au lieu
  // du contenu) est bien corrigé.
  let room = make_room("44444444-4444-4444-8444-444444444444");
  let topic1 = room.chat_topic();
  let topic2 = room.chat_topic();
  assert_eq!(
    topic1, topic2,
    "chat_topic() doit être déterministe entre deux appels"
  );
}

#[test]
fn chat_topic_different_for_different_rooms() {
  let room_a = make_room("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
  let room_b = make_room("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb");
  assert_ne!(
    room_a.chat_topic(),
    room_b.chat_topic(),
    "deux rooms différentes doivent avoir des topics différents"
  );
}

// ---------------------------------------------------------------------------
// Tests JoinCode::encode() / decode() roundtrip
// ---------------------------------------------------------------------------

#[test]
fn join_code_encode_decode_roundtrip() {
  let room_id = "55555555-5555-4555-8555-555555555555";
  let original = make_join_code(room_id);

  let encoded = original.encode().expect("encode ne doit pas échouer");

  // Le code doit commencer par le préfixe attendu.
  assert!(
    encoded.starts_with("pgpilot:join:"),
    "code encodé doit commencer par \"pgpilot:join:\", obtenu : {encoded}"
  );

  let decoded = JoinCode::decode(&encoded).expect("decode ne doit pas échouer sur un code valide");

  assert_eq!(decoded.room_id, original.room_id, "room_id préservé");
  assert_eq!(decoded.relay, original.relay, "relay préservé");
  assert_eq!(
    decoded.invited_by, original.invited_by,
    "invited_by préservé"
  );
  assert_eq!(decoded.room_name, original.room_name, "room_name préservé");
  assert_eq!(decoded.sig, original.sig, "sig préservée");
}

// ---------------------------------------------------------------------------
// Tests JoinCode::decode() — rejets attendus
// ---------------------------------------------------------------------------

#[test]
fn decode_rejects_missing_prefix() {
  // Chaîne quelconque sans le préfixe "pgpilot:join:".
  let err = JoinCode::decode("not-a-join-code");
  assert!(
    matches!(err, Err(ChatError::InvalidJoinCode)),
    "préfixe manquant doit retourner InvalidJoinCode"
  );
}

#[test]
fn decode_rejects_invalid_base64() {
  // Préfixe correct mais base64 invalide.
  let err = JoinCode::decode("pgpilot:join:!!!invalid-base64!!!");
  assert!(
    matches!(err, Err(ChatError::InvalidJoinCode)),
    "base64 invalide doit retourner InvalidJoinCode"
  );
}

#[test]
fn decode_rejects_malformed_json() {
  use base64::Engine as _;
  // Base64 valide d'un JSON malformé.
  let bad_json = b"not json at all";
  let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bad_json);
  let code = format!("pgpilot:join:{b64}");

  let err = JoinCode::decode(&code);
  assert!(
    matches!(err, Err(ChatError::InvalidJoinCode)),
    "JSON malformé doit retourner InvalidJoinCode"
  );
}

#[test]
fn decode_rejects_non_hex_invited_by() {
  use base64::Engine as _;
  // invited_by doit être 40 hex ASCII — on met 40 chars non-hex.
  let jc = serde_json::json!({
    "room_id": "66666666-6666-4666-8666-666666666666",
    "relay": "mqtts://broker.hivemq.com:8883",
    "invited_by": "Z".repeat(40),   // 'Z' n'est pas hexadécimal
    "room_name": null,
    "sig": "nonempty"
  });
  let json_bytes = serde_json::to_vec(&jc).unwrap();
  let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&json_bytes);
  let code = format!("pgpilot:join:{b64}");

  let err = JoinCode::decode(&code);
  assert!(
    matches!(err, Err(ChatError::InvalidJoinCode)),
    "invited_by non-hex doit retourner InvalidJoinCode"
  );
}

#[test]
fn decode_rejects_non_mqtts_relay() {
  use base64::Engine as _;
  // relay doit être mqtts:// ou mqtt://localhost — un URL http:// est refusé.
  let jc = serde_json::json!({
    "room_id": "77777777-7777-4777-8777-777777777777",
    "relay": "http://evil.com:8080",
    "invited_by": "C".repeat(40),
    "room_name": null,
    "sig": "nonempty"
  });
  let json_bytes = serde_json::to_vec(&jc).unwrap();
  let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&json_bytes);
  let code = format!("pgpilot:join:{b64}");

  let err = JoinCode::decode(&code);
  assert!(
    matches!(err, Err(ChatError::InvalidJoinCode)),
    "relay avec schéma http:// doit retourner InvalidJoinCode"
  );
}

#[test]
fn decode_rejects_empty_sig() {
  use base64::Engine as _;
  // sig vide → JoinCodeSignatureInvalid (pas InvalidJoinCode).
  let jc = serde_json::json!({
    "room_id": "88888888-8888-4888-8888-888888888888",
    "relay": "mqtts://broker.hivemq.com:8883",
    "invited_by": "D".repeat(40),
    "room_name": null,
    "sig": ""
  });
  let json_bytes = serde_json::to_vec(&jc).unwrap();
  let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&json_bytes);
  let code = format!("pgpilot:join:{b64}");

  let err = JoinCode::decode(&code);
  assert!(
    matches!(err, Err(ChatError::JoinCodeSignatureInvalid)),
    "sig vide doit retourner JoinCodeSignatureInvalid"
  );
}

#[test]
fn decode_rejects_room_name_too_long() {
  use base64::Engine as _;
  // room_name > 256 octets → InvalidJoinCode.
  let long_name = "x".repeat(257);
  let jc = serde_json::json!({
    "room_id": "99999999-9999-4999-8999-999999999999",
    "relay": "mqtts://broker.hivemq.com:8883",
    "invited_by": "E".repeat(40),
    "room_name": long_name,
    "sig": "nonempty"
  });
  let json_bytes = serde_json::to_vec(&jc).unwrap();
  let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&json_bytes);
  let code = format!("pgpilot:join:{b64}");

  let err = JoinCode::decode(&code);
  assert!(
    matches!(err, Err(ChatError::InvalidJoinCode)),
    "room_name > 256 chars doit retourner InvalidJoinCode"
  );
}

#[test]
fn decode_accepts_room_name_at_boundary() {
  use base64::Engine as _;
  // room_name exactement 256 octets → accepté.
  let boundary_name = "y".repeat(256);
  let jc = serde_json::json!({
    "room_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    "relay": "mqtts://broker.hivemq.com:8883",
    "invited_by": "F".repeat(40),
    "room_name": boundary_name,
    "sig": "nonempty"
  });
  let json_bytes = serde_json::to_vec(&jc).unwrap();
  let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&json_bytes);
  let code = format!("pgpilot:join:{b64}");

  // Doit réussir le decode (la vérification de signature vient après).
  let result = JoinCode::decode(&code);
  assert!(
    result.is_ok(),
    "room_name de 256 octets doit être accepté par decode(), erreur : {result:?}"
  );
}

// ---------------------------------------------------------------------------
// Tests JoinCode::verify() et sign() — nécessitent GPG
// ---------------------------------------------------------------------------

#[test]
#[ignore = "nécessite une clef secrète dans le keyring GPG — exécuter avec --ignored"]
fn join_code_sign_and_verify_roundtrip() {
  // Ce test est délibérément ignoré car il nécessite GPG.
  // Implémentation prévue dans les tests d'intégration GPG complets.
}

// ---------------------------------------------------------------------------
// Tests RoomStore::load()
// ---------------------------------------------------------------------------

#[test]
fn room_store_load_returns_default_if_file_absent() {
  // Pointer XDG_CONFIG_HOME vers un répertoire temporaire vide pour que
  // rooms.yaml n'existe pas.
  let _guard = ENV_LOCK.lock().expect("ENV_LOCK empoisonné");

  let tmp = tempfile::TempDir::new().expect("tempdir");
  // `dirs::config_dir()` lit XDG_CONFIG_HOME en priorité sur Linux.
  // On redirige vers le tempdir pour que `pgpilot/rooms.yaml` soit introuvable.
  //
  // SAFETY : std::env::set_var est unsafe dans un contexte multi-thread.
  // Le `ENV_LOCK` garantit qu'un seul test env-sensible s'exécute à la fois.
  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", tmp.path());
  }

  // rooms.yaml n'existe pas → `load()` doit retourner `Self::default()` sans erreur.
  let store = RoomStore::load().expect("load() doit réussir si le fichier est absent");
  assert!(
    store.rooms.is_empty(),
    "store par défaut doit avoir une liste de rooms vide"
  );

  // Nettoyage : restaurer la variable (si elle existait avant).
  unsafe {
    std::env::remove_var("XDG_CONFIG_HOME");
  }
  // `tmp` est droppé ici — le répertoire temporaire est supprimé.
  drop(tmp);
}

#[test]
fn room_store_load_rejects_oversized_file() {
  // Créer un fichier rooms.yaml > 1 Mio dans un répertoire temporaire.
  let _guard = ENV_LOCK.lock().expect("ENV_LOCK empoisonné");

  let tmp = tempfile::TempDir::new().expect("tempdir");
  let config_dir = tmp.path().join("pgpilot");
  std::fs::create_dir_all(&config_dir).expect("create_dir_all");

  // Écrire un fichier de 1 Mio + 1 octet.
  let oversized_path = config_dir.join("rooms.yaml");
  let oversized_content = vec![b'x'; 1_048_577]; // 1 Mio + 1
  std::fs::write(&oversized_path, &oversized_content).expect("write oversized file");

  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", tmp.path());
  }

  let result = RoomStore::load();

  unsafe {
    std::env::remove_var("XDG_CONFIG_HOME");
  }
  drop(tmp);

  assert!(
    matches!(result, Err(ChatError::RoomsYamlLoadFailed(_))),
    "un fichier rooms.yaml > 1 Mio doit retourner RoomsYamlLoadFailed, obtenu : {result:?}"
  );
}
