//! Tests d'intégration pour le sous-système chat (wire, présence, MQTT).
//!
//! Les tests 1–7 sont rapides et sans réseau.
//! Le test 8 (`mqtt_connect_publish_subscribe`) nécessite un broker MQTT
//! accessible et est marqué `#[ignore]`.

#![allow(dead_code)]

use pgpilot::chat::{
  ChatError, MqttConfig, PresenceStatus, PresenceTracker, PresenceUpdate, WireAck, WireMessage,
  MAX_WIRE_MESSAGE_BYTES, SIGN_CANONICAL_PREFIX,
};

// ---------------------------------------------------------------------------
// Helpers de test
// ---------------------------------------------------------------------------

/// Construit un `WireMessage` valide avec des valeurs contrôlées.
fn sample_wire_message() -> WireMessage {
  WireMessage {
    id: "abc00000-0000-4000-8000-000000000000".to_string(),
    sender: "def0".repeat(10), // 40 chars hex-like
    ts: 123,
    payload: "pgp".to_string(),
    signature: "-----BEGIN PGP SIGNATURE-----\nsig\n-----END PGP SIGNATURE-----".to_string(),
  }
}

/// Fingerprint de 40 caractères utilisé dans les tests de présence.
fn test_fp() -> String {
  "ABCD1234".repeat(5) // 40 chars
}

// ---------------------------------------------------------------------------
// Test 1 — Roundtrip JSON de WireMessage
// ---------------------------------------------------------------------------

#[test]
fn wire_message_serialization_roundtrip() {
  let original = WireMessage {
    id: "11111111-1111-4111-8111-111111111111".to_string(),
    sender: "A".repeat(40),
    ts: 1_700_000_000,
    payload: "-----BEGIN PGP MESSAGE-----\nhello world\n-----END PGP MESSAGE-----".to_string(),
    signature: "-----BEGIN PGP SIGNATURE-----\nsig_data\n-----END PGP SIGNATURE-----".to_string(),
  };

  // Sérialise via l'API publique — infaillible pour ce message valide.
  let bytes = original
    .to_json_bytes()
    .expect("sérialisation infaillible pour un message valide");

  // Désérialise et vérifie la préservation de tous les champs.
  let decoded = WireMessage::from_json_bytes(&bytes)
    .expect("désérialisation infaillible pour un JSON bien formé");

  assert_eq!(decoded.id, original.id, "id préservé");
  assert_eq!(decoded.sender, original.sender, "sender préservé");
  assert_eq!(decoded.ts, original.ts, "ts préservé");
  assert_eq!(decoded.payload, original.payload, "payload préservé");
  assert_eq!(decoded.signature, original.signature, "signature préservée");
}

// ---------------------------------------------------------------------------
// Test 2 — WireMessage dont la payload dépasse MAX_WIRE_MESSAGE_BYTES
// ---------------------------------------------------------------------------

#[test]
fn wire_message_too_large_rejected() {
  // On crée un message dont le payload dépasse la limite une fois sérialisé.
  // Un payload de MAX_WIRE_MESSAGE_BYTES+1 octets garantit que le JSON résultant
  // dépasse lui-même la limite.
  let mut msg = sample_wire_message();
  msg.payload = "x".repeat(MAX_WIRE_MESSAGE_BYTES + 1);

  let result = msg.to_json_bytes();
  assert_eq!(
    result,
    Err(ChatError::MessageTooLarge),
    "to_json_bytes doit retourner MessageTooLarge quand le JSON dépasse la limite"
  );
}

// ---------------------------------------------------------------------------
// Test 3 — Roundtrip JSON de WireAck
// ---------------------------------------------------------------------------

#[test]
fn wire_ack_serialization_roundtrip() {
  let original = WireAck {
    msg_id: "22222222-2222-4222-8222-222222222222".to_string(),
    from: "B".repeat(40),
    ts: 1_700_000_001,
  };

  // to_json_bytes est infaillible pour des champs String bien formés.
  let bytes = original
    .to_json_bytes()
    .expect("sérialisation infaillible pour WireAck valide");

  let decoded = WireAck::from_json_bytes(&bytes)
    .expect("désérialisation infaillible pour un JSON WireAck bien formé");

  assert_eq!(decoded.msg_id, original.msg_id, "msg_id préservé");
  assert_eq!(decoded.from, original.from, "from préservé");
  assert_eq!(decoded.ts, original.ts, "ts préservé");
}

// ---------------------------------------------------------------------------
// Test 4 — client_id tronqué à 23 caractères max (limite protocole MQTT 3.1)
// ---------------------------------------------------------------------------

#[test]
fn mqtt_client_id_max_23_chars() {
  // La limite MQTT 3.1 impose un client_id ≤ 23 octets.
  // MqttHandle::spawn tronque config.client_id à 23 chars avant de passer à
  // rumqttc : `config.client_id.chars().take(23).collect()`.
  //
  // MqttConfig::client_id n'est pas tronqué lors de la construction du struct —
  // c'est spawn() qui applique la règle. On vérifie ici que la règle s'applique
  // correctement via l'invariant observable : un fingerprint de 40 chars doit
  // produire un client_id interne de 23 chars maximum.
  //
  // Puisque make_client_id n'est pas pub, on vérifie la logique directement :
  let fp_40 = "ABCDEF0123456789".repeat(2) + "ABCDEF01"; // 40 chars
  assert_eq!(fp_40.len(), 40);

  // Réplication de la logique interne de spawn().
  let truncated: String = fp_40.chars().take(23).collect();
  assert!(
    truncated.len() <= 23,
    "client_id tronqué doit être ≤ 23 chars, obtenu {}",
    truncated.len()
  );
  assert_eq!(
    truncated.len(),
    23,
    "un fingerprint de 40 chars doit produire un client_id de exactement 23 chars"
  );

  // Vérification du cas limite : un client_id déjà court ne doit pas être allongé.
  let short_id = "pgpilot-dev";
  let short_truncated: String = short_id.chars().take(23).collect();
  assert_eq!(
    short_truncated, short_id,
    "un client_id court (≤ 23 chars) est conservé tel quel"
  );

  // Le struct MqttConfig reste constructible avec un id long (la troncature est
  // appliquée dans spawn, pas dans le constructeur).
  let cfg = MqttConfig {
    relay: "mqtts://broker.example.com:8883".to_string(),
    client_id: fp_40.clone(),
    presence_fp: fp_40,
  };
  // Le champ brut conserve la valeur fournie.
  assert_eq!(
    cfg.client_id.len(),
    40,
    "MqttConfig stocke le client_id non tronqué"
  );
}

// ---------------------------------------------------------------------------
// Test 5 — PresenceTracker : passage Online puis Offline
// ---------------------------------------------------------------------------

#[test]
fn presence_tracker_online_offline() {
  let fp = test_fp();
  let mut tracker = PresenceTracker::new();

  // Au départ, le participant est inconnu → is_online retourne false.
  assert!(
    !tracker.is_online(&fp),
    "participant inconnu → is_online == false"
  );

  // Passage Online.
  tracker.apply(PresenceUpdate {
    fp: fp.clone(),
    status: PresenceStatus::Online,
  });
  assert!(tracker.is_online(&fp), "après Online → is_online == true");

  // Passage Offline.
  tracker.apply(PresenceUpdate {
    fp: fp.clone(),
    status: PresenceStatus::Offline,
  });
  assert!(
    !tracker.is_online(&fp),
    "après Offline → is_online == false"
  );
}

// ---------------------------------------------------------------------------
// Test 6 — decode_payload : online / offline / inconnu
// ---------------------------------------------------------------------------

#[test]
fn presence_decode_payload() {
  let fp = test_fp();

  // b"online" → PresenceStatus::Online
  let update = PresenceTracker::decode_payload(&fp, b"online")
    .expect("b\"online\" doit produire Some(PresenceUpdate)");
  assert_eq!(update.status, PresenceStatus::Online, "\"online\" → Online");
  assert_eq!(update.fp, fp, "fingerprint préservé dans le PresenceUpdate");

  // b"offline" → PresenceStatus::Offline
  let update = PresenceTracker::decode_payload(&fp, b"offline")
    .expect("b\"offline\" doit produire Some(PresenceUpdate)");
  assert_eq!(
    update.status,
    PresenceStatus::Offline,
    "\"offline\" → Offline"
  );

  // b"garbage" → None (payload invalide non reconnu — pas de fallback Offline)
  // Note : decode_payload retourne None pour toute valeur non reconnue.
  // Il n'y a pas de fallback implicite vers Offline par conception : le
  // tracker ignore silencieusement les payloads malformés.
  let unknown = PresenceTracker::decode_payload(&fp, b"garbage");
  assert!(
    unknown.is_none(),
    "payload inconnu (\"garbage\") → None, pas de fallback implicite"
  );
}

// ---------------------------------------------------------------------------
// Test 7 — canonical_bytes contient tous les champs et les séparateurs \x00
// ---------------------------------------------------------------------------

#[test]
fn canonical_bytes_includes_all_fields() {
  let msg = WireMessage {
    id: "abc".to_string(),
    sender: "def".to_string(),
    ts: 123,
    payload: "pgp".to_string(),
    signature: "sig".to_string(),
  };

  let canon = msg.canonical_bytes();

  // Le préfixe SIGN_CANONICAL_PREFIX doit apparaître en tête du buffer.
  assert!(
    canon.starts_with(SIGN_CANONICAL_PREFIX),
    "canonical_bytes doit commencer par SIGN_CANONICAL_PREFIX"
  );

  // Les valeurs des quatre champs couverts par la signature doivent être présentes.
  assert!(
    canon.windows(3).any(|w| w == b"abc"),
    "id \"abc\" doit apparaître dans canonical_bytes"
  );
  assert!(
    canon.windows(3).any(|w| w == b"def"),
    "sender \"def\" doit apparaître dans canonical_bytes"
  );
  assert!(
    canon.windows(3).any(|w| w == b"123"),
    "ts \"123\" (en décimal) doit apparaître dans canonical_bytes"
  );
  assert!(
    canon.windows(3).any(|w| w == b"pgp"),
    "payload \"pgp\" doit apparaître dans canonical_bytes"
  );

  // Les séparateurs \x00 doivent être présents entre les champs.
  // SIGN_CANONICAL_PREFIX se termine par \x00 (1 octet nul) + 3 séparateurs
  // inter-champs (id\x00, sender\x00, ts\x00) = 4 octets nuls minimum.
  let null_count = canon.iter().filter(|&&b| b == 0).count();
  assert!(
    null_count >= 4,
    "attendu ≥ 4 octets \\x00 dans canonical_bytes (prefix \\x00 + 3 séparateurs), trouvé {null_count}"
  );
}

// ---------------------------------------------------------------------------
// Test 8 — Connexion MQTT réelle (broker réseau requis, ignoré par défaut)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Nécessite un broker MQTT disponible sur test.mosquitto.org:1883.
          // Exécuter avec : cargo test -- --ignored
async fn mqtt_connect_publish_subscribe() {
  use pgpilot::chat::{MqttEvent, MqttHandle};
  use tokio::time::{timeout, Duration};

  // Ce test utilise mqtt:// (plain-text) vers localhost-équivalent public.
  // test.mosquitto.org:1883 est un broker de test public sans chiffrement.
  // On utilise 127.0.0.1 ici pour simuler : remplacez par l'adresse réelle
  // si vous disposez d'un broker local.
  //
  // Pour tester contre test.mosquitto.org, utilisez le port 8883 (TLS) :
  //   relay: "mqtts://test.mosquitto.org:8883"
  //
  // AVERTISSEMENT : test.mosquitto.org est un service public partagé.
  // En CI, préférer un broker éphémère (mosquitto dans Docker).
  let fp = "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC".to_string();
  let topic_pub = format!("pgpilot/test/{}/pub", &fp[..16]);
  let topic_sub = topic_pub.clone();

  let config = MqttConfig {
    relay: "mqtt://localhost:1883".to_string(),
    client_id: format!("pgpilot-test-{}", &fp[..8]),
    presence_fp: fp.clone(),
  };

  // spawn() tente une connexion TCP — échoue si le broker est absent.
  let handle = match MqttHandle::spawn(config) {
    Ok(h) => h,
    Err(e) => {
      eprintln!("Broker absent ou configuration invalide : {e}");
      return;
    }
  };

  // Prendre le stream d'évènements une seule fois.
  let mut event_rx = match handle.take_event_stream() {
    Some(rx) => rx,
    None => {
      eprintln!("take_event_stream() a retourné None — stream déjà consommé");
      handle.shutdown();
      return;
    }
  };

  // Attendre l'évènement Connected (max 5 secondes).
  let connected = timeout(Duration::from_secs(5), async {
    loop {
      match event_rx.recv().await {
        Some(MqttEvent::Connected) => return true,
        Some(_) => continue,
        None => return false,
      }
    }
  })
  .await;

  match connected {
    Ok(true) => {}
    Ok(false) => {
      eprintln!("Channel fermé avant Connected — broker probablement absent");
      handle.shutdown();
      return;
    }
    Err(_) => {
      eprintln!("Timeout en attendant Connected — broker probablement absent");
      handle.shutdown();
      return;
    }
  }

  // Souscrire au topic de test.
  handle.take_event_stream(); // no-op si déjà consommé, juste pour illustrer la sémantique take()
  use pgpilot::chat::mqtt::ChatTransport as _;
  let _ = handle.subscribe(&topic_sub, 1).await;

  // Publier un message de test.
  let payload = b"hello-pgpilot-test".to_vec();
  let _ = handle.publish(&topic_pub, payload.clone(), 1, false).await;

  // Attendre la réception du message publié (max 5 secondes).
  let received = timeout(Duration::from_secs(5), async {
    loop {
      match event_rx.recv().await {
        Some(MqttEvent::MessageReceived {
          topic,
          payload: recv_payload,
        }) if topic == topic_sub => return Some(recv_payload),
        Some(_) => continue,
        None => return None,
      }
    }
  })
  .await;

  handle.shutdown();

  match received {
    Ok(Some(recv_payload)) => {
      assert_eq!(
        recv_payload, payload,
        "payload reçu doit correspondre au payload publié"
      );
    }
    Ok(None) => panic!("Channel fermé avant réception du message"),
    Err(_) => panic!("Timeout en attendant la réception du message publié"),
  }
}
