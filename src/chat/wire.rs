#![allow(dead_code)]
//! Formats de données sur le wire MQTT.
//!
//! Ce module définit les types sérialisés échangés entre clients :
//! - [`WireMessage`] — message chiffré + signé publié sur le topic chat.
//! - [`WireAck`] — accusé de réception best-effort publié sur le topic ack.
//!
//! La canonicalisation pour la signature couvre `id + sender + ts + payload`
//! en les concaténant avec des séparateurs `\x00` après le préfixe
//! `SIGN_CANONICAL_PREFIX`.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::chat::{ChatError, ChatResult, MAX_WIRE_MESSAGE_BYTES, SIGN_CANONICAL_PREFIX};

/// Message chiffré + signé publié sur `pgpilot/chat/{hash}`.
///
/// Aucun champ `recipients_fps` n'est présent sur le wire afin de minimiser
/// les métadonnées exposées au broker. Les destinataires sont implicites dans
/// les session keys PGP embarquées dans `payload`.
///
/// # Contrainte de taille
///
/// La représentation JSON sérialisée doit être ≤ [`MAX_WIRE_MESSAGE_BYTES`]
/// (64 Kio). Validation côté émetteur **et** récepteur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireMessage {
  /// Identifiant unique du message (UUID v4).
  pub id: String,
  /// Fingerprint 40 hex de l'émetteur.
  pub sender: String,
  /// Horodatage Unix en secondes UTC (horloge de l'émetteur).
  pub ts: i64,
  /// Blob PGP armored chiffré pour tous les destinataires du salon.
  /// Commence par `"-----BEGIN PGP MESSAGE-----\n"`.
  pub payload: String,
  /// Signature PGP détachée couvrant la canonicalisation du message.
  /// Commence par `"-----BEGIN PGP SIGNATURE-----\n"`.
  pub signature: String,
}

/// Accusé de réception best-effort publié sur `pgpilot/ack/{msg_id[..16]}`.
///
/// Non signé : un ACK forgé n'a aucun impact sur la confidentialité ou
/// l'authenticité des messages (l'UUID v4 à 122 bits d'entropie rend les
/// ACK forgés aléatoirement statistiquement impossibles).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireAck {
  /// UUID du [`WireMessage`] acquitté.
  pub msg_id: String,
  /// Fingerprint 40 hex du client qui confirme la réception.
  pub from: String,
  /// Horodatage Unix en secondes UTC.
  pub ts: i64,
}

impl WireMessage {
  /// Sérialise le message en JSON et valide la contrainte de taille.
  ///
  /// # Errors
  ///
  /// Retourne [`ChatError::MessageTooLarge`] si le JSON dépasse
  /// `MAX_WIRE_MESSAGE_BYTES`.
  pub fn to_json_bytes(&self) -> ChatResult<Vec<u8>> {
    let bytes =
      serde_json::to_vec(self).map_err(|e| ChatError::MalformedWireMessage(e.to_string()))?;
    if bytes.len() > MAX_WIRE_MESSAGE_BYTES {
      return Err(ChatError::MessageTooLarge);
    }
    Ok(bytes)
  }

  /// Désérialise depuis JSON et valide la contrainte de taille.
  ///
  /// # Errors
  ///
  /// - [`ChatError::MessageTooLarge`] si le payload entrant dépasse la limite.
  /// - [`ChatError::MalformedWireMessage`] si le JSON est invalide.
  pub fn from_json_bytes(bytes: &[u8]) -> ChatResult<Self> {
    if bytes.len() > MAX_WIRE_MESSAGE_BYTES {
      return Err(ChatError::MessageTooLarge);
    }
    serde_json::from_slice(bytes).map_err(|e| ChatError::MalformedWireMessage(e.to_string()))
  }

  /// Retourne la forme canonique à signer/vérifier.
  ///
  /// ```text
  /// SIGN_CANONICAL_PREFIX || id || \x00 || sender || \x00 || ts_decimal || \x00 || payload
  /// ```
  ///
  /// La signature couvre `id + sender + ts + payload` (pas seulement
  /// `payload`) afin d'empêcher la substitution d'émetteur ou
  /// d'horodatage.
  #[must_use]
  pub fn canonical_bytes(&self) -> Vec<u8> {
    let mut buf = Vec::with_capacity(
      SIGN_CANONICAL_PREFIX.len()
        + self.id.len()
        + 1
        + self.sender.len()
        + 1
        + 20  // ts décimal max ~20 chars
        + 1
        + self.payload.len(),
    );
    buf.extend_from_slice(SIGN_CANONICAL_PREFIX);
    buf.extend_from_slice(self.id.as_bytes());
    buf.push(b'\x00');
    buf.extend_from_slice(self.sender.as_bytes());
    buf.push(b'\x00');
    buf.extend_from_slice(self.ts.to_string().as_bytes());
    buf.push(b'\x00');
    buf.extend_from_slice(self.payload.as_bytes());
    buf
  }
}

impl WireAck {
  /// Sérialise l'ACK en JSON.
  ///
  /// # Errors
  ///
  /// Retourne [`ChatError::MalformedWireMessage`] en cas d'erreur de
  /// sérialisation (ne devrait jamais se produire avec des champs `String`).
  pub fn to_json_bytes(&self) -> ChatResult<Vec<u8>> {
    serde_json::to_vec(self).map_err(|e| ChatError::MalformedWireMessage(e.to_string()))
  }

  /// Désérialise un ACK depuis JSON.
  ///
  /// # Errors
  ///
  /// Retourne [`ChatError::MalformedWireMessage`] si le JSON est invalide.
  pub fn from_json_bytes(bytes: &[u8]) -> ChatResult<Self> {
    serde_json::from_slice(bytes).map_err(|e| ChatError::MalformedWireMessage(e.to_string()))
  }
}

// ---------------------------------------------------------------------------
// Fonctions standalone de publication / subscription des ACK
// ---------------------------------------------------------------------------

/// Publie un ACK best-effort (QoS 0, non retained, non signé).
///
/// L'UUID v4 à 122 bits d'entropie rend les ACK forgés statistiquement
/// impossibles — aucune signature n'est donc nécessaire.
///
/// # Errors
///
/// Retourne une [`ChatError`] si la sérialisation ou l'envoi échoue.
pub async fn publish_ack(
  handle: &crate::chat::MqttHandle,
  msg_id: &str,
  from_fp: &str,
) -> crate::chat::ChatResult<()> {
  use crate::chat::mqtt::ChatTransport as _;
  let ack = WireAck {
    msg_id: msg_id.to_string(),
    from: from_fp.to_string(),
    ts: chrono::Utc::now().timestamp(),
  };
  let bytes = ack.to_json_bytes()?;
  let topic = format!(
    "{}/{}",
    crate::chat::ACK_TOPIC_PREFIX,
    &msg_id[..msg_id.len().min(16)]
  );
  handle.publish(&topic, bytes, 0, false).await
}

/// Souscrit au topic ACK pour un message envoyé.
///
/// Doit être appelé juste après la publication du [`WireMessage`] afin de
/// recevoir les accusés de réception des destinataires.
///
/// # Errors
///
/// Retourne une [`ChatError`] si la souscription échoue.
pub async fn subscribe_ack(
  handle: &crate::chat::MqttHandle,
  msg_id: &str,
) -> crate::chat::ChatResult<()> {
  use crate::chat::mqtt::ChatTransport as _;
  let topic = format!(
    "{}/{}",
    crate::chat::ACK_TOPIC_PREFIX,
    &msg_id[..msg_id.len().min(16)]
  );
  handle.subscribe(&topic, 0).await
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_message() -> WireMessage {
    WireMessage {
      id: "11111111-1111-4111-8111-111111111111".to_string(),
      sender: "A".repeat(40),
      ts: 1_700_000_000,
      payload: "-----BEGIN PGP MESSAGE-----\nhello\n-----END PGP MESSAGE-----".to_string(),
      signature: "-----BEGIN PGP SIGNATURE-----\nsig\n-----END PGP SIGNATURE-----".to_string(),
    }
  }

  #[test]
  fn wire_message_round_trip() {
    let msg = sample_message();
    let bytes = msg.to_json_bytes().expect("serialize");
    let decoded = WireMessage::from_json_bytes(&bytes).expect("deserialize");
    assert_eq!(decoded.id, msg.id);
    assert_eq!(decoded.sender, msg.sender);
    assert_eq!(decoded.ts, msg.ts);
  }

  #[test]
  fn wire_message_too_large() {
    let mut msg = sample_message();
    msg.payload = "x".repeat(MAX_WIRE_MESSAGE_BYTES + 1);
    assert_eq!(msg.to_json_bytes(), Err(ChatError::MessageTooLarge));
  }

  #[test]
  fn wire_message_receive_too_large() {
    let oversized = vec![b'x'; MAX_WIRE_MESSAGE_BYTES + 1];
    assert_eq!(
      WireMessage::from_json_bytes(&oversized),
      Err(ChatError::MessageTooLarge)
    );
  }

  #[test]
  fn canonical_bytes_contains_separators() {
    let msg = sample_message();
    let canon = msg.canonical_bytes();
    // Le préfixe doit être présent en tête.
    assert!(canon.starts_with(SIGN_CANONICAL_PREFIX));
    // Les séparateurs \x00 doivent être présents entre les champs.
    let null_count = canon.iter().filter(|&&b| b == 0).count();
    // SIGN_CANONICAL_PREFIX se termine par \x00 (1) + 3 séparateurs inter-champs = 4 minimum.
    assert!(
      null_count >= 4,
      "attendu ≥ 4 octets nuls, trouvé {null_count}"
    );
  }

  #[test]
  fn wire_ack_round_trip() {
    let ack = WireAck {
      msg_id: "22222222-2222-4222-8222-222222222222".to_string(),
      from: "B".repeat(40),
      ts: 1_700_000_001,
    };
    let bytes = ack.to_json_bytes().expect("serialize");
    let decoded = WireAck::from_json_bytes(&bytes).expect("deserialize");
    assert_eq!(decoded.msg_id, ack.msg_id);
    assert_eq!(decoded.from, ack.from);
    assert_eq!(decoded.ts, ack.ts);
  }
}
