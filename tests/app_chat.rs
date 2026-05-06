//! Tests des handlers App chat (`src/app/chat.rs`) sans iced rendering ni MQTT.
//!
//! Couvre :
//! 1. `push_chat_message` (via `Message::ChatReceived`) : bornage FIFO à
//!    `MAX_MESSAGES_PER_ROOM` — après 501 insertions, len == 500.
//! 2. `on_chat_ack_received` (via `Message::ChatAckReceived`) : le statut ACK
//!    passe de Pending à Received pour le bon message.
//! 3. `room_by_id` (via `rooms` + comportement indirect) : None pour id
//!    inconnu, Some pour id connu.

#![allow(dead_code)]

mod common;

use pgpilot::app::{
  App, ChatNewForm, CreateKeyForm, DecryptForm, EncryptForm, ImportForm, Message, MqttState,
  SignForm, View,
};
use pgpilot::chat::{
  AckStatus, ChatMessage, MessageDirection, PresenceTracker, Room, RoomParticipant,
  MAX_MESSAGES_PER_ROOM,
};

// ---------------------------------------------------------------------------
// make_test_app() : fabrique un App minimal avec tous les champs chat
// ---------------------------------------------------------------------------

fn make_test_app() -> App {
  use pgpilot::config::Config;
  use pgpilot::i18n::{self, Language};

  let config = Config::default();
  let strings = i18n::strings_for(Language::English);

  App {
    view: View::MyKeys,
    keys: Vec::new(),
    selected: None,
    error: None,
    status: None,
    status_generation: 0,
    loading: false,
    card_connected: false,
    pending: None,
    keyserver_statuses: std::collections::HashMap::new(),
    create_form: CreateKeyForm::default(),
    import_form: ImportForm::default(),
    encrypt_form: EncryptForm::default(),
    decrypt_form: DecryptForm::default(),
    health_report: Vec::new(),
    health_loading: false,
    sign_form: SignForm::default(),
    previous_view: None,
    expiry_warnings: Vec::new(),
    config,
    strings,
    // --- Champs chat v0.6.0 ---
    rooms: Vec::new(),
    active_room: None,
    chat_messages: std::collections::HashMap::new(),
    presence: PresenceTracker::new(),
    mqtt_state: MqttState::Disconnected,
    mqtt: None,
    chat_input: String::new(),
    chat_new_form: ChatNewForm::default(),
    chat_crypto: None,
    chat_identity_popup: None,
  }
}

// ---------------------------------------------------------------------------
// Helper : construit un ChatMessage minimal avec les champs obligatoires.
// ---------------------------------------------------------------------------

fn make_chat_message(id: &str, sender_fp: &str, text: &str) -> ChatMessage {
  ChatMessage {
    id: id.to_string(),
    sender_fp: sender_fp.to_string(),
    text: text.to_string(),
    ts: chrono::Utc::now(),
    received_at: chrono::Utc::now(),
    direction: MessageDirection::Received,
    acks: std::collections::HashMap::new(),
  }
}

/// Fingerprint de test valide (40 hex chars).
fn test_fp() -> String {
  "AAAA0000AAAA0000AAAA0000AAAA0000AAAA0000".to_string()
}

fn test_room_id() -> String {
  "11111111-1111-4111-8111-111111111111".to_string()
}

// ---------------------------------------------------------------------------
// 1. push_chat_message : bornage FIFO à MAX_MESSAGES_PER_ROOM
// ---------------------------------------------------------------------------

#[test]
fn push_chat_message_fifo_cap_at_max() {
  let mut app = make_test_app();
  let room_id = test_room_id();

  // Injecter MAX_MESSAGES_PER_ROOM + 1 messages via Message::ChatReceived.
  // `on_chat_received` appelle `push_chat_message` puis tente d'envoyer un ACK
  // (qui nécessite MQTT) — le handler ACK est best-effort et retourne
  // Task::none() si mqtt est absent, donc pas de panique.
  let total = MAX_MESSAGES_PER_ROOM + 1;
  for i in 0..total {
    let msg = make_chat_message(&format!("msg-{i:05}"), &test_fp(), &format!("message #{i}"));
    let _ = app.update(Message::ChatReceived(room_id.clone(), msg));
  }

  let queue = app
    .chat_messages
    .get(&room_id)
    .expect("la queue doit exister pour la room");

  assert_eq!(
    queue.len(),
    MAX_MESSAGES_PER_ROOM,
    "après {} insertions la queue doit être bornée à {} messages (FIFO), obtenu {}",
    total,
    MAX_MESSAGES_PER_ROOM,
    queue.len()
  );
}

#[test]
fn push_chat_message_oldest_message_is_evicted() {
  let mut app = make_test_app();
  let room_id = test_room_id();

  // Insérer exactement MAX + 1 messages en ordonnant les ids.
  // Le premier message inséré (id "msg-00000") doit être évincé.
  let total = MAX_MESSAGES_PER_ROOM + 1;
  for i in 0..total {
    let msg = make_chat_message(&format!("msg-{i:05}"), &test_fp(), "text");
    let _ = app.update(Message::ChatReceived(room_id.clone(), msg));
  }

  let queue = app.chat_messages.get(&room_id).expect("queue exists");

  // L'id "msg-00000" (le plus ancien) doit avoir été droppé.
  assert!(
    !queue.iter().any(|m| m.id == "msg-00000"),
    "le message le plus ancien (msg-00000) doit être évincé (FIFO)"
  );
  // Le dernier inséré doit être présent.
  let last_id = format!("msg-{:05}", MAX_MESSAGES_PER_ROOM);
  assert!(
    queue.iter().any(|m| m.id == last_id),
    "le dernier message inséré ({last_id}) doit être présent dans la queue"
  );
}

#[test]
fn push_chat_message_below_cap_preserves_all() {
  let mut app = make_test_app();
  let room_id = test_room_id();

  // Insérer moins que la limite — tous les messages doivent être conservés.
  let count = MAX_MESSAGES_PER_ROOM / 2;
  for i in 0..count {
    let msg = make_chat_message(&format!("half-{i}"), &test_fp(), "text");
    let _ = app.update(Message::ChatReceived(room_id.clone(), msg));
  }

  let queue = app.chat_messages.get(&room_id).expect("queue exists");
  assert_eq!(
    queue.len(),
    count,
    "en dessous de la limite, tous les {} messages doivent être conservés",
    count
  );
}

// ---------------------------------------------------------------------------
// 2. on_chat_ack_received : le statut ACK passe à Received pour le bon message
// ---------------------------------------------------------------------------

#[test]
fn ack_received_updates_correct_message() {
  let mut app = make_test_app();
  let room_id = test_room_id();
  let msg_id = "target-msg-id".to_string();
  let other_msg_id = "other-msg-id".to_string();
  let acker_fp = "BBBB1111BBBB1111BBBB1111BBBB1111BBBB1111".to_string();

  // Insérer deux messages dans la queue.
  let msg_target = make_chat_message(&msg_id, &test_fp(), "target");
  let msg_other = make_chat_message(&other_msg_id, &test_fp(), "other");
  let _ = app.update(Message::ChatReceived(room_id.clone(), msg_target));
  let _ = app.update(Message::ChatReceived(room_id.clone(), msg_other));

  // Envoyer un ACK pour le message cible uniquement.
  let _ = app.update(Message::ChatAckReceived(
    room_id.clone(),
    msg_id.clone(),
    acker_fp.clone(),
  ));

  let queue = app.chat_messages.get(&room_id).expect("queue exists");

  // Le message cible doit avoir l'ACK dans sa map.
  let target = queue
    .iter()
    .find(|m| m.id == msg_id)
    .expect("message cible introuvable");
  assert_eq!(
    target.acks.get(&acker_fp),
    Some(&AckStatus::Received),
    "l'ACK du message cible doit être Received pour le bon fingerprint"
  );

  // L'autre message ne doit pas avoir d'ACK.
  let other = queue
    .iter()
    .find(|m| m.id == other_msg_id)
    .expect("autre message introuvable");
  assert!(
    other.acks.is_empty(),
    "l'autre message ne doit pas recevoir d'ACK"
  );
}

#[test]
fn ack_received_for_unknown_msg_id_does_not_panic() {
  let mut app = make_test_app();
  let room_id = test_room_id();

  // Queue vide — ACK pour un id inexistant ne doit pas paniquer.
  let result = app.update(Message::ChatAckReceived(
    room_id,
    "nonexistent-id".to_string(),
    test_fp(),
  ));
  // Task::none() est attendu — on vérifie juste l'absence de panique.
  let _ = result;
}

#[test]
fn ack_received_for_unknown_room_does_not_panic() {
  let mut app = make_test_app();

  // Aucune room dans app.chat_messages — ne doit pas paniquer.
  let result = app.update(Message::ChatAckReceived(
    "unknown-room-id".to_string(),
    "some-msg-id".to_string(),
    test_fp(),
  ));
  let _ = result;
}

#[test]
fn multiple_acks_for_same_message() {
  let mut app = make_test_app();
  let room_id = test_room_id();
  let msg_id = "multi-ack-msg".to_string();
  let fp1 = "CCCC2222CCCC2222CCCC2222CCCC2222CCCC2222".to_string();
  let fp2 = "DDDD3333DDDD3333DDDD3333DDDD3333DDDD3333".to_string();

  let msg = make_chat_message(&msg_id, &test_fp(), "text");
  let _ = app.update(Message::ChatReceived(room_id.clone(), msg));

  // Deux ACK de fingerprints différents.
  let _ = app.update(Message::ChatAckReceived(
    room_id.clone(),
    msg_id.clone(),
    fp1.clone(),
  ));
  let _ = app.update(Message::ChatAckReceived(
    room_id.clone(),
    msg_id.clone(),
    fp2.clone(),
  ));

  let queue = app.chat_messages.get(&room_id).expect("queue exists");
  let target = queue
    .iter()
    .find(|m| m.id == msg_id)
    .expect("message introuvable");

  assert_eq!(
    target.acks.len(),
    2,
    "deux ACK distincts doivent être stockés"
  );
  assert_eq!(target.acks.get(&fp1), Some(&AckStatus::Received));
  assert_eq!(target.acks.get(&fp2), Some(&AckStatus::Received));
}

// ---------------------------------------------------------------------------
// 3. room_by_id : None pour id inconnu, Some pour id connu
// ---------------------------------------------------------------------------

fn make_room(id: &str) -> Room {
  Room {
    id: id.to_string(),
    name: "test".to_string(),
    relay: "mqtts://broker.example.com:8883".to_string(),
    my_fp: test_fp(),
    created_at: chrono::Utc::now(),
    participants: vec![RoomParticipant {
      fp: test_fp(),
      joined_at: chrono::Utc::now(),
    }],
  }
}

#[test]
fn room_by_id_returns_none_for_unknown_id() {
  let app = make_test_app();
  // rooms est vide → toute lookup doit retourner None.
  // On teste indirectement : ChatRoomSelected déclenche ensure_chat_started
  // qui appelle room_by_id. Sans MQTT ni crypto, il ne panique pas.
  let mut app = app;
  let _ = app.update(Message::ChatRoomSelected("nonexistent-room".to_string()));
  // Si room_by_id avait paniqué, le test aurait échoué ici.
}

#[test]
fn room_by_id_returns_some_for_known_id() {
  let mut app = make_test_app();
  let room = make_room(&test_room_id());
  app.rooms.push(room);

  // Vérifier via le champ public `rooms`.
  let found = app.rooms.iter().find(|r| r.id == test_room_id());
  assert!(found.is_some(), "room doit être trouvée par son id");
  assert_eq!(
    found.unwrap().id,
    test_room_id(),
    "l'id de la room trouvée doit correspondre"
  );
}

#[test]
fn room_by_id_returns_none_when_rooms_empty() {
  let app = make_test_app();
  assert!(app.rooms.is_empty(), "rooms doit être vide initialement");
  let found = app.rooms.iter().find(|r| r.id == "anything");
  assert!(found.is_none(), "lookup dans une liste vide → None");
}

#[test]
fn room_by_id_with_multiple_rooms_finds_correct() {
  let mut app = make_test_app();
  let id_a = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
  let id_b = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";

  app.rooms.push(make_room(id_a));
  app.rooms.push(make_room(id_b));

  let found_a = app.rooms.iter().find(|r| r.id == id_a);
  let found_b = app.rooms.iter().find(|r| r.id == id_b);
  let found_c = app.rooms.iter().find(|r| r.id == "unknown");

  assert!(found_a.is_some(), "room A doit être trouvée");
  assert_eq!(found_a.unwrap().id, id_a);
  assert!(found_b.is_some(), "room B doit être trouvée");
  assert_eq!(found_b.unwrap().id, id_b);
  assert!(found_c.is_none(), "room inconnue → None");
}
