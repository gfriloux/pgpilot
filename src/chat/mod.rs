// Items are used progressively as axes 4–8 are implemented.
#![allow(dead_code, unused_imports)]
//! Sous-système de chat PGP temps réel via MQTT (v0.6.0).
//!
//! ## Architecture
//!
//! Le chat repose sur six modules :
//!
//! - [`mod@mqtt`] — connexion MQTT, boucle de reconnexion, canal de commandes.
//! - [`mod@rooms`] — CRUD des salons persistés, encodage/vérification des
//!   codes d'invitation.
//! - [`mod@crypto`] — chiffrement/déchiffrement/signature PGP in-process via
//!   sequoia.
//! - [`mod@presence`] — tracking de présence en RAM, LWT, heartbeat.
//! - [`mod@wire`] — sérialisation/désérialisation des messages sur le wire.
//! - [`error`] (privé) — [`ChatError`] et [`ChatResult`].
//!
//! ## Démarrage paresseux
//!
//! Aucune connexion MQTT n'est établie au lancement de l'application.
//! La connexion est initiée par `App::ensure_chat_started` à la première
//! ouverture de salon.
//!
//! ## Éphémère par conception
//!
//! Les messages ne sont **jamais** persistés. Seuls les salons (`rooms.yaml`)
//! sont sauvegardés sur le disque.

pub mod crypto;
pub mod mqtt;
pub mod presence;
pub mod rooms;
pub mod wire;

mod error;

// ---------------------------------------------------------------------------
// Ré-exports publics
// ---------------------------------------------------------------------------

pub use crypto::ChatCryptoCtx;
pub use error::{ChatError, ChatResult};
pub use mqtt::{MqttConfig, MqttEvent, MqttHandle};
pub use presence::{
  publish_offline, publish_online, subscribe_room_presence, PresenceStatus, PresenceTracker,
  PresenceUpdate,
};
pub use rooms::{Room, RoomParticipant, RoomStore};
pub use wire::{publish_ack, subscribe_ack, WireAck, WireMessage};

// ---------------------------------------------------------------------------
// Types RAM uniquement (jamais sérialisés sur disque)
// ---------------------------------------------------------------------------

/// Sortie de `ChatCryptoCtx::encrypt_for_room` — input de `wire::build_wire_message`.
///
/// Pipeline complet :
/// ```text
/// plaintext → ChatCryptoCtx::encrypt_for_room → ChatPayload
///           → wire::build_wire_message → WireMessage → JSON → MQTT publish
/// ```
#[derive(Debug, Clone)]
pub struct ChatPayload {
  /// Blob PGP armored chiffré pour tous les destinataires du salon.
  pub ciphertext_armored: String,
  /// Signature PGP détachée armored couvrant la canonicalisation du message.
  pub signature_armored: String,
}

/// Message déchiffré et vérifié, produit par `ChatCryptoCtx::decrypt_message`.
#[derive(Debug, Clone)]
pub struct VerifiedMessage {
  /// Texte en clair déchiffré.
  pub plaintext: String,
  /// Fingerprint 40 hex du signataire vérifié.
  pub signer_fp: String,
  /// Horodatage de signature (UTC).
  pub signed_at: chrono::DateTime<chrono::Utc>,
}

/// Message de chat en mémoire vive.
///
/// Jamais persisté. Borné à [`MAX_MESSAGES_PER_ROOM`] par salon (FIFO via
/// `VecDeque::pop_front`).
#[derive(Debug, Clone)]
pub struct ChatMessage {
  /// UUID v4 du [`WireMessage`] correspondant.
  pub id: String,
  /// Fingerprint 40 hex de l'émetteur.
  pub sender_fp: String,
  /// Texte en clair déchiffré.
  pub text: String,
  /// Horodatage de l'émetteur (ts du WireMessage).
  pub ts: chrono::DateTime<chrono::Utc>,
  /// Horodatage de réception locale (permet de détecter les messages antidatés).
  pub received_at: chrono::DateTime<chrono::Utc>,
  /// Direction du message du point de vue de l'utilisateur local.
  pub direction: MessageDirection,
  /// ACK par fingerprint de confirmant.
  pub acks: std::collections::HashMap<String, AckStatus>,
}

/// Direction d'un message du point de vue de l'utilisateur local.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageDirection {
  /// Message envoyé par l'utilisateur local.
  Sent,
  /// Message reçu d'un autre participant.
  Received,
}

/// État de l'accusé de réception d'un pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AckStatus {
  /// ACK en attente (pas encore reçu).
  Pending,
  /// ACK reçu.
  Received,
}

// ---------------------------------------------------------------------------
// Constantes globales (spec §14)
// ---------------------------------------------------------------------------

/// Préfixe des topics MQTT pour les messages de chat.
pub const CHAT_TOPIC_PREFIX: &str = "pgpilot/chat";

/// Préfixe des topics MQTT pour la présence.
pub const PRESENCE_TOPIC_PREFIX: &str = "pgpilot/presence";

/// Préfixe des topics MQTT pour les ACK.
pub const ACK_TOPIC_PREFIX: &str = "pgpilot/ack";

/// Taille maximale d'un [`WireMessage`] sérialisé en JSON (64 Kio).
pub const MAX_WIRE_MESSAGE_BYTES: usize = 65_536;

/// Nombre maximal de messages conservés en RAM par salon (FIFO).
pub const MAX_MESSAGES_PER_ROOM: usize = 500;

/// Intervalle du heartbeat de présence en secondes.
pub const PRESENCE_HEARTBEAT_SECS: u64 = 30;

/// Délai LWT de présence en secondes (3 × heartbeat).
pub const PRESENCE_LWT_TIMEOUT_SECS: u16 = 90;

/// Intervalle de keepalive MQTT en secondes.
pub const MQTT_KEEPALIVE_SECS: u16 = 60;

/// Délai de base du backoff exponentiel de reconnexion (ms).
pub const MQTT_RECONNECT_BASE_MS: u64 = 1_000;

/// Délai maximal du backoff exponentiel de reconnexion (ms).
pub const MQTT_RECONNECT_MAX_MS: u64 = 60_000;

/// Capacité du canal mpsc tâche → UI (évènements MQTT).
pub const MQTT_EVENT_CHANNEL_CAP: usize = 256;

/// N'émettre `Reconnecting` que toutes les N tentatives au-delà de 3.
pub const MQTT_RECONNECT_LOG_EVERY: u32 = 5;

/// Préfixe de la canonicalisation pour la signature des messages.
///
/// Forme : `SIGN_CANONICAL_PREFIX || id || \x00 || sender || \x00 || ts || \x00 || payload`
pub const SIGN_CANONICAL_PREFIX: &[u8] = b"pgpilot-msg\x00";

/// Préfixe des codes d'invitation.
pub const JOIN_CODE_PREFIX: &str = "pgpilot:join:";
