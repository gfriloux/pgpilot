use std::path::Path;
use std::sync::Arc;

use tauri::Emitter as _;
use pgpilot::chat::{
  mqtt::ChatTransport as _,
  ChatCryptoCtx, ChatPayload, MqttConfig, MqttEvent, MqttHandle, Room, RoomParticipant, RoomStore,
  WireAck, WireMessage, ACK_TOPIC_PREFIX, MAX_WIRE_MESSAGE_BYTES, PRESENCE_TOPIC_PREFIX,
};
use pgpilot::gpg::card::{card_status as gpg_card_status, move_key_to_card};
use pgpilot::gpg::health::{run_all_checks, HealthCheck};
use pgpilot::gpg::gnupg_homedir;
use pgpilot::gpg::keyring::{
  add_subkey, backup_key as gpg_backup_key, check_keyserver as gpg_check_keyserver,
  create_key as gpg_create_key, decrypt_files as gpg_decrypt_files, delete_key as gpg_delete_key,
  encrypt_files as gpg_encrypt_files, export_public_key,
  export_public_key_armored as gpg_export_armored, generate_revocation_cert as gpg_gen_rev,
  import_key, import_key_from_keyserver, import_key_from_text, import_key_from_url,
  inspect_decrypt as gpg_inspect_decrypt, list_keys as gpg_list_keys,
  publish_key as gpg_publish_key, renew_subkey, revocation_cert_path,
  rotate_subkey as gpg_rotate_subkey, set_key_trust as gpg_set_trust, sign_file as gpg_sign_file,
  validate_fp, verify_signature as gpg_verify_signature,
};
use pgpilot::gpg::types::{CardInfo, DecryptStatus, KeyExpiry, KeyInfo, TrustLevel, VerifyResult};
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Chat state
// ---------------------------------------------------------------------------

/// État d'une session chat active (une seule connexion MQTT à la fois).
struct ChatSession {
  handle: MqttHandle,
  room: Room,
  crypto: Arc<ChatCryptoCtx>,
}

/// État Tauri géré : session chat active + RoomStore en cache.
///
/// Géré via `tauri::Manager::manage` — accessible depuis toutes les commandes
/// via `tauri::State<'_, ChatState>`.
struct ChatState {
  session: Mutex<Option<ChatSession>>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn days_to_expiry(days: u32) -> Result<KeyExpiry, String> {
  match days {
    365 => Ok(KeyExpiry::OneYear),
    730 => Ok(KeyExpiry::TwoYears),
    1825 => Ok(KeyExpiry::FiveYears),
    _ => Err(format!("invalid expiry: {days} days")),
  }
}

fn str_to_trust(s: &str) -> Result<TrustLevel, String> {
  match s {
    "undefined" => Ok(TrustLevel::Undefined),
    "marginal" => Ok(TrustLevel::Marginal),
    "full" => Ok(TrustLevel::Full),
    _ => Err(format!("invalid trust level: {s}")),
  }
}

fn str_to_usage(s: &str) -> (&'static str, &'static str) {
  match s {
    "S" => ("ed25519", "sign"),
    "A" => ("ed25519", "auth"),
    _ => ("cv25519", "encr"), // "E"
  }
}

// ---------------------------------------------------------------------------
// Commands — version / keys
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_version() -> String {
  env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
async fn list_keys() -> Result<Vec<KeyInfo>, String> {
  tokio::task::spawn_blocking(|| {
    gpg_list_keys()
      .map(|(keys, _card_connected)| keys)
      .map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — export
// ---------------------------------------------------------------------------

#[tauri::command]
async fn export_public_key_armored(fp: String) -> Result<String, String> {
  tokio::task::spawn_blocking(move || gpg_export_armored(&fp).map_err(|e| e.to_string()))
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn export_public_key_to_file(fp: String, dest_path: String) -> Result<(), String> {
  tokio::task::spawn_blocking(move || {
    export_public_key(&fp, Path::new(&dest_path)).map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — backup
// ---------------------------------------------------------------------------

/// Returns the file names created inside `dest_dir` (up to 2 entries).
#[tauri::command]
async fn backup_key(fp: String, dest_dir: String) -> Result<Vec<String>, String> {
  tokio::task::spawn_blocking(move || {
    // Derive the key_id (last 16 hex chars of the fingerprint).
    if fp.len() < 16 {
      return Err("fingerprint too short".to_string());
    }
    let key_id = fp[fp.len() - 16..].to_string();

    // Canonicalize and verify the destination directory to prevent path traversal
    let canon_dir = std::fs::canonicalize(&dest_dir)
      .map_err(|e| format!("Invalid destination: {e}"))?;
    if !canon_dir.is_dir() {
      return Err("Destination is not a directory".to_string());
    }
    let dir = canon_dir;

    gpg_backup_key(&fp, &dir, &key_id)
      .map(|(secret_name, rev_name)| {
        let mut files = vec![secret_name];
        if let Some(rev) = rev_name {
          files.push(rev);
        }
        files
      })
      .map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — deletion
// ---------------------------------------------------------------------------

#[tauri::command]
async fn delete_key(fp: String, has_secret: bool) -> Result<(), String> {
  tokio::task::spawn_blocking(move || gpg_delete_key(&fp, has_secret).map_err(|e| e.to_string()))
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — keyserver
// ---------------------------------------------------------------------------

#[tauri::command]
async fn publish_key(fp: String, keyserver_url: String) -> Result<String, String> {
  tokio::task::spawn_blocking(move || gpg_publish_key(&fp, &keyserver_url).map_err(|e| e.to_string()))
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn check_keyserver(fp: String) -> Result<bool, String> {
  tokio::task::spawn_blocking(move || {
    gpg_check_keyserver(&fp)
      .map(|(_fp, published)| published)
      .map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — subkeys
// ---------------------------------------------------------------------------

#[tauri::command]
async fn renew_subkey_cmd(key_fp: String, subkey_fp: String, expiry_days: u32) -> Result<(), String> {
  tokio::task::spawn_blocking(move || {
    let expiry = days_to_expiry(expiry_days)?;
    renew_subkey(&key_fp, &subkey_fp, &expiry).map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn rotate_subkey_cmd(
  key_fp: String,
  subkey_fp: String,
  usage: String,
  expiry_days: u32,
) -> Result<(), String> {
  tokio::task::spawn_blocking(move || {
    let expiry = days_to_expiry(expiry_days)?;
    let (algo, usage_str) = str_to_usage(&usage);
    gpg_rotate_subkey(&key_fp, &subkey_fp, algo, usage_str, &expiry).map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn add_subkey_cmd(master_fp: String, usage: String, expiry_days: u32) -> Result<(), String> {
  tokio::task::spawn_blocking(move || {
    let expiry = days_to_expiry(expiry_days)?;
    let (algo, usage_str) = str_to_usage(&usage);
    add_subkey(&master_fp, algo, usage_str, &expiry).map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — trust
// ---------------------------------------------------------------------------

#[tauri::command]
async fn set_key_trust(fp: String, trust: String) -> Result<(), String> {
  tokio::task::spawn_blocking(move || {
    let level = str_to_trust(&trust)?;
    gpg_set_trust(&fp, &level).map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — import
// ---------------------------------------------------------------------------

#[tauri::command]
async fn import_key_text(content: String) -> Result<(), String> {
  tokio::task::spawn_blocking(move || import_key_from_text(&content).map_err(|e| e.to_string()))
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_key_url(url: String) -> Result<(), String> {
  tokio::task::spawn_blocking(move || import_key_from_url(&url).map_err(|e| e.to_string()))
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_key_keyserver(query: String, keyserver_url: String) -> Result<(), String> {
  tokio::task::spawn_blocking(move || {
    import_key_from_keyserver(&query, &keyserver_url).map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_key_file(path: String) -> Result<(), String> {
  tokio::task::spawn_blocking(move || import_key(Path::new(&path)).map_err(|e| e.to_string()))
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — YubiKey / smartcard
// ---------------------------------------------------------------------------

#[tauri::command]
async fn card_status() -> Result<Option<CardInfo>, String> {
  tokio::task::spawn_blocking(|| Ok(gpg_card_status()))
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn move_to_card(fp: String) -> Result<(), String> {
  tokio::task::spawn_blocking(move || move_key_to_card(&fp).map_err(|e| e.to_string()))
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — key creation
// ---------------------------------------------------------------------------

/// Creates a key with default TwoYears subkey expiry and auth subkey included.
/// Returns the fingerprint of the newly created key.
#[tauri::command]
async fn create_key(name: String, email: String, expiry_days: u32) -> Result<String, String> {
  tokio::task::spawn_blocking(move || {
    let expiry = days_to_expiry(expiry_days)?;
    gpg_create_key(&name, &email, &expiry, true).map_err(|e| e.to_string())?;
    // Retrieve the fingerprint by finding the key matching the email.
    let (keys, _) = gpg_list_keys().map_err(|e| e.to_string())?;
    keys
      .into_iter()
      .find(|k| k.email == email)
      .map(|k| k.fingerprint)
      .ok_or_else(|| "key created but fingerprint not found".to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — encrypt / sign / verify / decrypt / health
// ---------------------------------------------------------------------------

/// files: absolute paths, recipients: 40-hex fingerprints.
/// Returns the file names of the created output files.
#[tauri::command]
async fn encrypt_files_cmd(
  files: Vec<String>,
  recipients: Vec<String>,
  armor: bool,
  force_trust: bool,
) -> Result<Vec<String>, String> {
  tokio::task::spawn_blocking(move || {
    let paths: Vec<std::path::PathBuf> = files.iter().map(std::path::PathBuf::from).collect();
    gpg_encrypt_files(&paths, &recipients, armor, force_trust).map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Returns the path of the created `.sig` file.
#[tauri::command]
async fn sign_file_cmd(file: String, signer_fp: String) -> Result<String, String> {
  tokio::task::spawn_blocking(move || {
    gpg_sign_file(std::path::PathBuf::from(file), &signer_fp)
      .map(|p| p.to_string_lossy().into_owned())
      .map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

/// `sig_file = None` — automatically looks for `<file>.sig`.
#[tauri::command]
async fn verify_signature_cmd(
  file: String,
  sig_file: Option<String>,
) -> Result<VerifyResult, String> {
  tokio::task::spawn_blocking(move || {
    let sig = sig_file.map(std::path::PathBuf::from);
    gpg_verify_signature(std::path::PathBuf::from(file), sig).map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Returns GPG diagnostic checks.
#[tauri::command]
async fn run_health_checks_cmd() -> Result<Vec<HealthCheck>, String> {
  tokio::task::spawn_blocking(|| {
    let (keys, _) = pgpilot::gpg::keyring::list_keys().map_err(|e| e.to_string())?;
    Ok(run_all_checks(&keys))
  })
  .await
  .map_err(|e| e.to_string())?
}

/// files: absolute paths. Returns result messages per file.
#[tauri::command]
async fn decrypt_files_cmd(files: Vec<String>) -> Result<Vec<String>, String> {
  tokio::task::spawn_blocking(move || {
    let paths: Vec<std::path::PathBuf> = files.iter().map(std::path::PathBuf::from).collect();
    gpg_decrypt_files(&paths).map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Inspects whether a file can be decrypted with the local keys.
#[tauri::command]
async fn inspect_decrypt_cmd(file: String) -> Result<DecryptStatus, String> {
  tokio::task::spawn_blocking(move || {
    gpg_inspect_decrypt(std::path::Path::new(&file)).map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — chat rooms (lecture/écriture rooms.yaml, synchrones)
// ---------------------------------------------------------------------------

/// Liste tous les salons persistés dans `rooms.yaml`.
#[tauri::command]
async fn chat_list_rooms() -> Result<Vec<Room>, String> {
  tokio::task::spawn_blocking(|| {
    RoomStore::load()
      .map(|s| s.rooms)
      .map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Crée un nouveau salon, l'insère dans `rooms.yaml` et retourne le salon créé.
#[tauri::command]
async fn chat_create_room(
  name: String,
  relay: String,
  my_fp: String,
) -> Result<Room, String> {
  tokio::task::spawn_blocking(move || {
    // Validate relay URL (must be mqtts:// or mqtt://localhost)
    pgpilot::chat::mqtt::parse_relay_url(&relay).map_err(|e| e.to_string())?;

    // Validate my_fp (must be 40 hex characters)
    validate_fp(&my_fp).map_err(|e| e.to_string())?;

    // Bound room name length
    if name.trim().is_empty() || name.len() > 256 {
      return Err("Room name must be 1–256 characters".to_string());
    }

    let mut store = RoomStore::load().map_err(|e| e.to_string())?;
    let room = Room {
      id: uuid::Uuid::new_v4().to_string(),
      name,
      relay,
      my_fp: my_fp.clone(),
      created_at: chrono::Utc::now(),
      participants: vec![RoomParticipant {
        fp: my_fp,
        joined_at: chrono::Utc::now(),
      }],
    };
    store.upsert(room.clone());
    store.save().map_err(|e| e.to_string())?;
    Ok(room)
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Supprime un salon de `rooms.yaml`.
///
/// Retourne une erreur si le salon n'existe pas.
#[tauri::command]
async fn chat_delete_room(room_id: String) -> Result<(), String> {
  tokio::task::spawn_blocking(move || {
    let mut store = RoomStore::load().map_err(|e| e.to_string())?;
    store
      .remove(&room_id)
      .ok_or_else(|| format!("room {room_id} not found"))?;
    store.save().map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Ajoute un participant à un salon existant et persiste la modification.
#[tauri::command]
async fn chat_add_participant(room_id: String, participant_fp: String) -> Result<(), String> {
  tokio::task::spawn_blocking(move || {
    // Validate fingerprint format before modifying any state
    validate_fp(&participant_fp).map_err(|e| e.to_string())?;

    let mut store = RoomStore::load().map_err(|e| e.to_string())?;
    let room = store
      .rooms
      .iter_mut()
      .find(|r| r.id == room_id)
      .ok_or_else(|| format!("room {room_id} not found"))?;
    // Eviter les doublons.
    if !room.participants.iter().any(|p| p.fp == participant_fp) {
      room.participants.push(RoomParticipant {
        fp: participant_fp,
        joined_at: chrono::Utc::now(),
      });
    }
    store.save().map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Commands — session chat (connexion MQTT + événements temps réel)
// ---------------------------------------------------------------------------

/// Démarre la connexion MQTT pour le salon donné.
///
/// Si une session était déjà active, elle est arrêtée proprement avant
/// d'ouvrir la nouvelle. Les événements MQTT sont poussés vers le WebView
/// via `app_handle.emit()`.
///
/// Événements émis (écoutables via `listen()` côté frontend) :
/// - `chat:connected` — connexion ou reconnexion réussie
/// - `chat:disconnected` — déconnexion (payload `{ reason: string }`)
/// - `chat:reconnecting` — tentative en cours (payload `{ attempt: number }`)
/// - `chat:message` — message reçu déchiffré et vérifié
///   (payload `{ msg_id, sender_fp, content, ts, room_id }`)
/// - `chat:presence` — mise à jour de présence brute
///   (payload `{ raw_topic, raw_payload }`)
/// - `chat:ack` — accusé de réception reçu
///   (payload `{ msg_id, from_fp }`)
#[tauri::command]
async fn chat_start(
  room_id: String,
  app_handle: tauri::AppHandle,
  state: tauri::State<'_, ChatState>,
) -> Result<(), String> {
  let mut session_guard = state.session.lock().await;

  // Arrêter la session précédente si elle existe.
  if let Some(old) = session_guard.take() {
    old.handle.shutdown();
  }

  // Charger la room depuis rooms.yaml.
  let room = tokio::task::spawn_blocking({
    let rid = room_id.clone();
    move || {
      RoomStore::load()
        .map_err(|e| e.to_string())?
        .rooms
        .into_iter()
        .find(|r| r.id == rid)
        .ok_or_else(|| format!("room {rid} not found"))
    }
  })
  .await
  .map_err(|e| e.to_string())??;

  // Créer le contexte crypto (vérifie que la clef secrète est disponible).
  let crypto = {
    let fp = room.my_fp.clone();
    let peers: Vec<String> = room.participants.iter().map(|p| p.fp.clone()).collect();
    tokio::task::spawn_blocking(move || {
      ChatCryptoCtx::load(&fp, &peers).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??
  };
  let crypto = Arc::new(crypto);

  // Construire la config MQTT (client_id tronqué à 23 chars par MqttHandle::spawn).
  let client_id = format!("pgp-{}", &room.my_fp[..15_usize.min(room.my_fp.len())]);
  let config = MqttConfig {
    relay: room.relay.clone(),
    client_id,
    presence_fp: room.my_fp.clone(),
  };

  // Spawn synchrone de la tâche tokio MQTT.
  let handle =
    tokio::task::spawn_blocking(move || MqttHandle::spawn(config).map_err(|e| e.to_string()))
      .await
      .map_err(|e| e.to_string())??;

  // Souscrire au topic de chat du salon (QoS 1).
  let chat_topic = room.chat_topic();
  handle
    .subscribe(&chat_topic, 1)
    .await
    .map_err(|e| e.to_string())?;

  // Souscrire aux topics de présence des autres participants (QoS 0, retained).
  for participant in &room.participants {
    if participant.fp == room.my_fp {
      continue;
    }
    let presence_topic = pgpilot::chat::presence::PresenceTracker::presence_topic(&participant.fp);
    handle
      .subscribe(&presence_topic, 0)
      .await
      .map_err(|e| e.to_string())?;
  }

  // Prendre le stream d'événements (une seule fois par session).
  let event_rx = handle.take_event_stream();

  // Spawn de la boucle de dispatch des événements MQTT vers le frontend.
  let crypto_clone = Arc::clone(&crypto);
  let room_clone = room.clone();
  let ah = app_handle.clone();

  tokio::spawn(async move {
    if let Some(mut rx) = event_rx {
      while let Some(event) = rx.recv().await {
        match event {
          MqttEvent::Connected => {
            let _ = ah.emit("chat:connected", ());
          }
          MqttEvent::Disconnected(reason) => {
            let _ = ah.emit(
              "chat:disconnected",
              serde_json::json!({ "reason": reason }),
            );
          }
          MqttEvent::Reconnecting { attempt } => {
            let _ = ah.emit(
              "chat:reconnecting",
              serde_json::json!({ "attempt": attempt }),
            );
          }
          MqttEvent::MessageReceived { topic, payload } => {
            dispatch_mqtt_message(
              &topic,
              &payload,
              &room_clone,
              &crypto_clone,
              &ah,
            )
            .await;
          }
        }
      }
    }
  });

  *session_guard = Some(ChatSession { handle, room, crypto });
  Ok(())
}

/// Génère un code d'invitation signé pour le salon donné.
#[tauri::command]
async fn chat_generate_join_code(room_id: String, my_fp: String) -> Result<String, String> {
  tokio::task::spawn_blocking(move || {
    let ctx = ChatCryptoCtx::load(&my_fp, &[]).map_err(|e| e.to_string())?;
    let store = RoomStore::load().map_err(|e| e.to_string())?;
    let room = store
      .get(&room_id)
      .ok_or_else(|| format!("room {room_id} not found"))?
      .clone();
    let mut join_code = pgpilot::chat::rooms::JoinCode {
      room_id: room.id.clone(),
      relay: room.relay.clone(),
      invited_by: my_fp.clone(),
      room_name: Some(room.name.clone()),
      sig: String::new(),
    };
    let sig = join_code.sign(&ctx.homedir, &my_fp).map_err(|e| e.to_string())?;
    join_code.sig = sig;
    join_code.encode().map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Rejoint un salon via un code d'invitation (vérifie la signature GPG).
#[tauri::command]
async fn chat_join_room(join_code_str: String, my_fp: String) -> Result<Room, String> {
  tokio::task::spawn_blocking(move || {
    let ctx = ChatCryptoCtx::load(&my_fp, &[]).map_err(|e| e.to_string())?;
    let join_code =
      pgpilot::chat::rooms::JoinCode::decode(&join_code_str).map_err(|e| e.to_string())?;
    join_code.verify(&ctx.homedir).map_err(|e| e.to_string())?;
    let mut store = RoomStore::load().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now();
    let room = Room {
      id: join_code.room_id,
      name: join_code.room_name.unwrap_or_else(|| "Room".to_string()),
      relay: join_code.relay,
      my_fp: my_fp.clone(),
      created_at: now,
      participants: vec![
        RoomParticipant { fp: join_code.invited_by, joined_at: now },
        RoomParticipant { fp: my_fp, joined_at: now },
      ],
    };
    store.upsert(room.clone());
    store.save().map_err(|e| e.to_string())?;
    Ok(room)
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Arrête la session chat active proprement (Shutdown MQTT + nettoyage état).
#[tauri::command]
async fn chat_stop(state: tauri::State<'_, ChatState>) -> Result<(), String> {
  let mut guard = state.session.lock().await;
  if let Some(session) = guard.take() {
    session.handle.shutdown();
  }
  Ok(())
}

/// Chiffre et publie un message sur le topic MQTT du salon actif.
///
/// Retourne l'UUID du message publié (utilisable pour tracker les ACK).
#[tauri::command]
async fn chat_send(
  room_id: String,
  content: String,
  state: tauri::State<'_, ChatState>,
) -> Result<String, String> {
  let session_guard = state.session.lock().await;
  let session = session_guard
    .as_ref()
    .ok_or("no active chat session")?;

  if session.room.id != room_id {
    return Err("room_id mismatch with active session".to_string());
  }

  let recipient_fps: Vec<String> = session
    .room
    .participants
    .iter()
    .map(|p| p.fp.clone())
    .collect();
  let local_fp = session.crypto.local_fp.clone();
  let crypto = Arc::clone(&session.crypto);

  // Chiffrement synchrone via subprocess GPG.
  let chat_payload = tokio::task::spawn_blocking(move || {
    crypto
      .encrypt_for_room(&content, &recipient_fps)
      .map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())??;

  // Construire le WireMessage.
  let msg_id = uuid::Uuid::new_v4().to_string();
  let wire = WireMessage {
    id: msg_id.clone(),
    sender: local_fp,
    ts: chrono::Utc::now().timestamp(),
    payload: chat_payload.ciphertext_armored,
    signature: chat_payload.signature_armored,
  };

  let payload_bytes = wire.to_json_bytes().map_err(|e| e.to_string())?;
  let topic = session.room.chat_topic();

  // Publier (QoS 1, non retained).
  session
    .handle
    .publish(&topic, payload_bytes, 1, false)
    .await
    .map_err(|e| e.to_string())?;

  Ok(msg_id)
}

// ---------------------------------------------------------------------------
// Helpers internes — dispatch des messages MQTT entrants
// ---------------------------------------------------------------------------

/// Dispatche un message MQTT reçu selon son topic.
///
/// Gère les messages de chat (déchiffrement + vérification), les mises à jour
/// de présence, et les ACK. Les erreurs de déchiffrement sont ignorées
/// silencieusement (invariant de sécurité : ne jamais exposer des données
/// partiellement vérifiées).
async fn dispatch_mqtt_message(
  topic: &str,
  payload: &[u8],
  room: &Room,
  crypto: &ChatCryptoCtx,
  ah: &tauri::AppHandle,
) {
  if payload.len() > MAX_WIRE_MESSAGE_BYTES {
    return;
  }

  if topic == room.chat_topic() {
    dispatch_chat_message(payload, room, crypto, ah).await;
  } else if topic.starts_with(PRESENCE_TOPIC_PREFIX) {
    let _ = ah.emit(
      "chat:presence",
      serde_json::json!({
        "raw_topic": topic,
        "raw_payload": String::from_utf8_lossy(payload).to_string(),
      }),
    );
  } else if topic.starts_with(ACK_TOPIC_PREFIX) {
    if let Ok(ack) = WireAck::from_json_bytes(payload) {
      let _ = ah.emit(
        "chat:ack",
        serde_json::json!({
          "msg_id": ack.msg_id,
          "from_fp": ack.from,
        }),
      );
    }
  }
}

/// Déchiffre et vérifie un message de chat, puis l'émet vers le frontend.
///
/// Invariants de sécurité appliqués :
/// 1. L'expéditeur déclaré (`wire.sender`) doit être un participant du salon.
/// 2. La signature doit être valide (`VALIDSIG` — géré par `ChatCryptoCtx`).
/// 3. Le fingerprint signataire vérifié doit correspondre à `wire.sender`.
/// 4. En cas d'échec de vérification, le message est ignoré silencieusement.
async fn dispatch_chat_message(
  payload: &[u8],
  room: &Room,
  crypto: &ChatCryptoCtx,
  ah: &tauri::AppHandle,
) {
  // Désérialiser le WireMessage.
  let wire = match WireMessage::from_json_bytes(payload) {
    Ok(m) => m,
    Err(_) => return,
  };

  // Vérifier que l'expéditeur déclaré est un participant connu.
  if !room.participants.iter().any(|p| p.fp == wire.sender) {
    return;
  }

  // Reconstruire le ChatPayload à partir du WireMessage.
  let chat_payload = ChatPayload {
    ciphertext_armored: wire.payload.clone(),
    signature_armored: wire.signature.clone(),
  };

  // Déchiffrement + vérification de signature dans un thread bloquant.
  let homedir = crypto.homedir.clone();
  let local_fp = crypto.local_fp.clone();
  let wire_clone = wire.clone();
  let result = tokio::task::spawn_blocking(move || {
    // Recréer un ChatCryptoCtx depuis les champs publics (pas de Clone derivé).
    let ctx = ChatCryptoCtx {
      homedir,
      local_fp,
    };
    ctx.decrypt_message(&chat_payload)
  })
  .await;

  // Déchiffrement échoué → ignoré silencieusement (Ok(Err(_)) | Err(_)).
  if let Ok(Ok(verified)) = result {
    // Invariant anti-usurpation : le signataire vérifié doit correspondre
    // à l'expéditeur déclaré dans le WireMessage.
    if verified.signer_fp != wire_clone.sender {
      return;
    }
    let _ = ah.emit(
      "chat:message",
      serde_json::json!({
        "msg_id": wire_clone.id,
        "sender_fp": verified.signer_fp,
        "content": verified.plaintext,
        "ts": wire_clone.ts,
        "room_id": room.id,
      }),
    );
  }
}

// ---------------------------------------------------------------------------
/// Checks if a revocation certificate exists for the given fingerprint.
/// Returns the path as a String if found, or null.
#[tauri::command]
async fn check_revocation_cert(fp: String) -> Result<Option<String>, String> {
  tokio::task::spawn_blocking(move || {
    let homedir = gnupg_homedir().map_err(|e| e.to_string())?;
    revocation_cert_path(&homedir, &fp)
      .map(|opt| opt.map(|p| p.to_string_lossy().into_owned()))
      .map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

/// Generates a revocation certificate for the given fingerprint.
#[tauri::command]
async fn generate_revocation_cert_cmd(fp: String) -> Result<String, String> {
  tokio::task::spawn_blocking(move || {
    let homedir = gnupg_homedir().map_err(|e| e.to_string())?;
    gpg_gen_rev(&homedir, &fp)
      .map(|p| p.to_string_lossy().into_owned())
      .map_err(|e| e.to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

// Entry point
// ---------------------------------------------------------------------------

pub fn run() {
  tauri::Builder::default()
    .manage(ChatState {
      session: Mutex::new(None),
    })
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
      get_version,
      list_keys,
      export_public_key_armored,
      export_public_key_to_file,
      backup_key,
      delete_key,
      publish_key,
      check_keyserver,
      renew_subkey_cmd,
      rotate_subkey_cmd,
      add_subkey_cmd,
      set_key_trust,
      import_key_text,
      import_key_url,
      import_key_keyserver,
      import_key_file,
      card_status,
      move_to_card,
      create_key,
      encrypt_files_cmd,
      sign_file_cmd,
      verify_signature_cmd,
      run_health_checks_cmd,
      decrypt_files_cmd,
      inspect_decrypt_cmd,
      chat_list_rooms,
      chat_create_room,
      chat_delete_room,
      chat_add_participant,
      chat_start,
      chat_stop,
      chat_send,
      chat_generate_join_code,
      chat_join_room,
      check_revocation_cert,
      generate_revocation_cert_cmd,
    ])
    .run(tauri::generate_context!())
    .expect("error while running pgpilot");
}
