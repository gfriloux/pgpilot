//! Tests pour `WireMessage` et `WireAck` (`src/chat/wire.rs`).
//!
//! Les tests de ce module n'ont pas besoin de GPG ni de réseau — tout est
//! en mémoire (sérialisation JSON, canonicalisation).

#![allow(dead_code)]

use pgpilot::chat::{
  ChatError, WireAck, WireMessage, MAX_WIRE_MESSAGE_BYTES, SIGN_CANONICAL_PREFIX,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Construit un `WireMessage` valide avec des valeurs contrôlées.
fn sample_message() -> WireMessage {
  WireMessage {
    id: "11111111-1111-4111-8111-111111111111".to_string(),
    sender: "A".repeat(40),
    ts: 1_700_000_000,
    payload: "-----BEGIN PGP MESSAGE-----\nhello\n-----END PGP MESSAGE-----".to_string(),
    signature: "-----BEGIN PGP SIGNATURE-----\nsig\n-----END PGP SIGNATURE-----".to_string(),
  }
}

// ---------------------------------------------------------------------------
// 1. WireMessage roundtrip JSON
// ---------------------------------------------------------------------------

#[test]
fn wire_message_round_trip() {
  let msg = sample_message();
  let bytes = msg
    .to_json_bytes()
    .expect("sérialisation infaillible pour un message valide");
  let decoded = WireMessage::from_json_bytes(&bytes)
    .expect("désérialisation infaillible pour un JSON bien formé");

  assert_eq!(decoded.id, msg.id, "id préservé");
  assert_eq!(decoded.sender, msg.sender, "sender préservé");
  assert_eq!(decoded.ts, msg.ts, "ts préservé");
  assert_eq!(decoded.payload, msg.payload, "payload préservé");
  assert_eq!(decoded.signature, msg.signature, "signature préservée");
}

// ---------------------------------------------------------------------------
// 2. WireMessage::to_json_bytes() rejette si JSON > MAX_WIRE_MESSAGE_BYTES
// ---------------------------------------------------------------------------

#[test]
fn wire_message_emit_too_large() {
  // Un payload de MAX + 1 octets garantit que le JSON résultant dépasse la
  // limite quelle que soit la taille des autres champs.
  let mut msg = sample_message();
  msg.payload = "x".repeat(MAX_WIRE_MESSAGE_BYTES + 1);

  let result = msg.to_json_bytes();
  assert_eq!(
    result,
    Err(ChatError::MessageTooLarge),
    "to_json_bytes doit retourner MessageTooLarge quand le JSON dépasse {MAX_WIRE_MESSAGE_BYTES} octets"
  );
}

// ---------------------------------------------------------------------------
// 3. WireMessage::from_json_bytes() rejette si bytes > MAX_WIRE_MESSAGE_BYTES
// ---------------------------------------------------------------------------

#[test]
fn wire_message_receive_too_large() {
  let oversized = vec![b'x'; MAX_WIRE_MESSAGE_BYTES + 1];
  let result = WireMessage::from_json_bytes(&oversized);
  assert_eq!(
    result,
    Err(ChatError::MessageTooLarge),
    "from_json_bytes doit retourner MessageTooLarge pour un payload entrant dépassant la limite"
  );
}

// ---------------------------------------------------------------------------
// 4. WireMessage::from_json_bytes() rejette un JSON invalide
// ---------------------------------------------------------------------------

#[test]
fn wire_message_rejects_malformed_json() {
  let bad = b"not valid json";
  let result = WireMessage::from_json_bytes(bad);
  assert!(
    matches!(result, Err(ChatError::MalformedWireMessage(_))),
    "from_json_bytes doit retourner MalformedWireMessage pour un JSON invalide"
  );
}

// ---------------------------------------------------------------------------
// 5. WireMessage::canonical_bytes() — préfixe et contenu des champs
// ---------------------------------------------------------------------------

#[test]
fn canonical_bytes_starts_with_prefix() {
  let msg = sample_message();
  let canon = msg.canonical_bytes();
  assert!(
    canon.starts_with(SIGN_CANONICAL_PREFIX),
    "canonical_bytes() doit commencer par SIGN_CANONICAL_PREFIX"
  );
}

#[test]
fn canonical_bytes_contains_all_signed_fields() {
  let msg = WireMessage {
    id: "myid".to_string(),
    sender: "SENDER".repeat(7)[..40].to_string(), // exactement 40 chars
    ts: 9_999_999,
    payload: "MYPAYLOAD".to_string(),
    signature: String::new(),
  };
  let canon = msg.canonical_bytes();

  // id
  assert!(
    canon.windows(msg.id.len()).any(|w| w == msg.id.as_bytes()),
    "canonical_bytes doit contenir id"
  );
  // sender
  assert!(
    canon
      .windows(msg.sender.len())
      .any(|w| w == msg.sender.as_bytes()),
    "canonical_bytes doit contenir sender"
  );
  // ts en décimal
  let ts_str = msg.ts.to_string();
  assert!(
    canon.windows(ts_str.len()).any(|w| w == ts_str.as_bytes()),
    "canonical_bytes doit contenir ts en décimal"
  );
  // payload
  assert!(
    canon
      .windows(msg.payload.len())
      .any(|w| w == msg.payload.as_bytes()),
    "canonical_bytes doit contenir payload"
  );
}

#[test]
fn canonical_bytes_has_null_separators() {
  let msg = sample_message();
  let canon = msg.canonical_bytes();
  // SIGN_CANONICAL_PREFIX se termine par \x00 (1) + 3 séparateurs
  // inter-champs (après id, après sender, après ts) = 4 octets nuls minimum.
  let null_count = canon.iter().filter(|&&b| b == 0).count();
  assert!(
    null_count >= 4,
    "canonical_bytes doit contenir au moins 4 octets \\x00 (prefix \\x00 + 3 séparateurs), trouvé {null_count}"
  );
}

// ---------------------------------------------------------------------------
// 6. WireAck roundtrip JSON
// ---------------------------------------------------------------------------

#[test]
fn wire_ack_round_trip() {
  let ack = WireAck {
    msg_id: "22222222-2222-4222-8222-222222222222".to_string(),
    from: "B".repeat(40),
    ts: 1_700_000_001,
  };
  let bytes = ack
    .to_json_bytes()
    .expect("sérialisation WireAck infaillible");
  let decoded = WireAck::from_json_bytes(&bytes).expect("désérialisation WireAck infaillible");

  assert_eq!(decoded.msg_id, ack.msg_id, "msg_id préservé");
  assert_eq!(decoded.from, ack.from, "from préservé");
  assert_eq!(decoded.ts, ack.ts, "ts préservé");
}

#[test]
fn wire_ack_rejects_malformed_json() {
  let bad = b"{invalid json}";
  let result = WireAck::from_json_bytes(bad);
  assert!(
    matches!(result, Err(ChatError::MalformedWireMessage(_))),
    "from_json_bytes WireAck doit retourner MalformedWireMessage pour un JSON invalide"
  );
}
