//! Tests pour `PresenceTracker` (`src/chat/presence.rs`).
//!
//! Tout est en RAM — aucune dépendance GPG ni réseau.
//! Aucun test `#[ignore]` n'est nécessaire dans ce module.

#![allow(dead_code)]

use pgpilot::chat::{PresenceStatus, PresenceTracker, PresenceUpdate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Fingerprint de test valide (40 hex chars).
fn fp_a() -> String {
  "AAAA0000AAAA0000AAAA0000AAAA0000AAAA0000".to_string()
}

fn fp_b() -> String {
  "BBBB1111BBBB1111BBBB1111BBBB1111BBBB1111".to_string()
}

fn fp_c() -> String {
  "CCCC2222CCCC2222CCCC2222CCCC2222CCCC2222".to_string()
}

// ---------------------------------------------------------------------------
// 1. apply() + is_online() : online → true, offline → false
// ---------------------------------------------------------------------------

#[test]
fn tracker_apply_online_then_offline() {
  let fp = fp_a();
  let mut tracker = PresenceTracker::new();

  // État initial : participant inconnu → is_online == false.
  assert!(
    !tracker.is_online(&fp),
    "participant non encore connu → is_online doit être false"
  );

  // Passage Online.
  tracker.apply(PresenceUpdate {
    fp: fp.clone(),
    status: PresenceStatus::Online,
  });
  assert!(
    tracker.is_online(&fp),
    "après Online → is_online doit être true"
  );

  // Passage Offline.
  tracker.apply(PresenceUpdate {
    fp: fp.clone(),
    status: PresenceStatus::Offline,
  });
  assert!(
    !tracker.is_online(&fp),
    "après Offline → is_online doit être false"
  );
}

#[test]
fn tracker_multiple_participants() {
  let mut tracker = PresenceTracker::new();

  // Deux participants indépendants.
  tracker.apply(PresenceUpdate {
    fp: fp_a(),
    status: PresenceStatus::Online,
  });
  tracker.apply(PresenceUpdate {
    fp: fp_b(),
    status: PresenceStatus::Offline,
  });

  assert!(tracker.is_online(&fp_a()), "fp_a → Online");
  assert!(!tracker.is_online(&fp_b()), "fp_b → Offline");
  // fp_c jamais vu → false.
  assert!(!tracker.is_online(&fp_c()), "fp_c jamais vu → false");
}

// ---------------------------------------------------------------------------
// 2. mark_all_offline() : tous passent Offline
// ---------------------------------------------------------------------------

#[test]
fn mark_all_offline_resets_all_participants() {
  let mut tracker = PresenceTracker::new();

  // Mettre deux participants Online.
  tracker.apply(PresenceUpdate {
    fp: fp_a(),
    status: PresenceStatus::Online,
  });
  tracker.apply(PresenceUpdate {
    fp: fp_b(),
    status: PresenceStatus::Online,
  });

  assert!(
    tracker.is_online(&fp_a()),
    "fp_a Online avant mark_all_offline"
  );
  assert!(
    tracker.is_online(&fp_b()),
    "fp_b Online avant mark_all_offline"
  );

  tracker.mark_all_offline();

  assert!(
    !tracker.is_online(&fp_a()),
    "fp_a doit être Offline après mark_all_offline"
  );
  assert!(
    !tracker.is_online(&fp_b()),
    "fp_b doit être Offline après mark_all_offline"
  );
}

#[test]
fn mark_all_offline_on_empty_tracker_does_not_panic() {
  // Appel sur un tracker vide ne doit pas paniquer.
  let mut tracker = PresenceTracker::new();
  tracker.mark_all_offline(); // Ne doit pas paniquer.
}

// ---------------------------------------------------------------------------
// 3. decode_payload() : online / offline / garbage
// ---------------------------------------------------------------------------

#[test]
fn decode_payload_online() {
  let fp = fp_c();
  let update = PresenceTracker::decode_payload(&fp, b"online")
    .expect("b\"online\" doit produire Some(PresenceUpdate)");
  assert_eq!(
    update.status,
    PresenceStatus::Online,
    "\"online\" doit produire PresenceStatus::Online"
  );
  assert_eq!(
    update.fp, fp,
    "fingerprint doit être préservé dans le PresenceUpdate"
  );
}

#[test]
fn decode_payload_offline() {
  let fp = fp_a();
  let update = PresenceTracker::decode_payload(&fp, b"offline")
    .expect("b\"offline\" doit produire Some(PresenceUpdate)");
  assert_eq!(
    update.status,
    PresenceStatus::Offline,
    "\"offline\" doit produire PresenceStatus::Offline"
  );
}

#[test]
fn decode_payload_garbage_returns_none() {
  let fp = fp_b();
  // Payload inconnu → None (pas de fallback implicite vers Offline).
  let result = PresenceTracker::decode_payload(&fp, b"garbage");
  assert!(
    result.is_none(),
    "payload \"garbage\" doit retourner None, pas de fallback implicite"
  );
}

#[test]
fn decode_payload_empty_returns_none() {
  let fp = fp_a();
  let result = PresenceTracker::decode_payload(&fp, b"");
  assert!(result.is_none(), "payload vide doit retourner None");
}

#[test]
fn decode_payload_uppercase_online_returns_none() {
  // "ONLINE" n'est pas un payload valide (la comparaison est case-sensitive).
  let fp = fp_a();
  let result = PresenceTracker::decode_payload(&fp, b"ONLINE");
  assert!(
    result.is_none(),
    "\"ONLINE\" (majuscules) doit retourner None — comparaison case-sensitive"
  );
}

// ---------------------------------------------------------------------------
// 4. presence_topic() : format correct
// ---------------------------------------------------------------------------

#[test]
fn presence_topic_format() {
  let fp = "ABCDEF0123456789ABCDEF0123456789ABCDEF01";
  let topic = PresenceTracker::presence_topic(fp);
  // Forme attendue : "pgpilot/presence/" + fp[..16].
  assert_eq!(
    topic, "pgpilot/presence/ABCDEF0123456789",
    "presence_topic() doit produire \"pgpilot/presence/{{fp[..16]}}\""
  );
}

#[test]
fn presence_topic_starts_with_prefix() {
  let fp = fp_b();
  let topic = PresenceTracker::presence_topic(&fp);
  assert!(
    topic.starts_with("pgpilot/presence/"),
    "presence_topic() doit commencer par \"pgpilot/presence/\", obtenu : {topic}"
  );
}

#[test]
fn presence_topic_contains_first_16_hex() {
  let fp = "1234567890ABCDEF1234567890ABCDEF12345678";
  let topic = PresenceTracker::presence_topic(fp);
  assert!(
    topic.ends_with("1234567890ABCDEF"),
    "presence_topic() doit se terminer par les 16 premiers chars du fingerprint, obtenu : {topic}"
  );
}

#[test]
fn presence_topic_short_fp_does_not_panic() {
  // Un fingerprint plus court que 16 chars ne doit pas paniquer.
  let short_fp = "ABC";
  let topic = PresenceTracker::presence_topic(short_fp);
  assert!(
    topic.starts_with("pgpilot/presence/"),
    "presence_topic() doit fonctionner avec un fp court, obtenu : {topic}"
  );
}

// ---------------------------------------------------------------------------
// 5. get() retourne le bon PresenceStatus ou None
// ---------------------------------------------------------------------------

#[test]
fn tracker_get_returns_correct_status() {
  let mut tracker = PresenceTracker::new();

  // Aucun statut connu → None.
  assert!(
    tracker.get(&fp_a()).is_none(),
    "get() → None pour un participant inconnu"
  );

  tracker.apply(PresenceUpdate {
    fp: fp_a(),
    status: PresenceStatus::Online,
  });
  assert_eq!(
    tracker.get(&fp_a()),
    Some(&PresenceStatus::Online),
    "get() → Some(Online) après apply(Online)"
  );

  tracker.apply(PresenceUpdate {
    fp: fp_a(),
    status: PresenceStatus::Offline,
  });
  assert_eq!(
    tracker.get(&fp_a()),
    Some(&PresenceStatus::Offline),
    "get() → Some(Offline) après apply(Offline)"
  );
}
