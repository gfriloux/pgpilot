#![allow(dead_code)]
//! Tracking de présence des participants.
//!
//! La présence est communiquée via deux mécanismes MQTT :
//! - **Heartbeat** : publication périodique toutes les
//!   [`PRESENCE_HEARTBEAT_SECS`] secondes sur le topic
//!   `pgpilot/presence/{fp[..16]}` avec retain=true.
//! - **Last Will Testament (LWT)** : le broker publie automatiquement
//!   `"offline"` avec retain=true si la connexion est perdue sans DISCONNECT
//!   propre.
//!
//! [`PresenceTracker`] agrège les mises à jour en RAM pour toute la session.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::chat::PRESENCE_TOPIC_PREFIX;

/// État de présence d'un participant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresenceStatus {
  /// Le participant est connecté et actif.
  Online,
  /// Le participant est hors-ligne (déconnexion propre ou LWT broker).
  Offline,
}

/// Mise à jour de présence reçue depuis un topic MQTT.
#[derive(Debug, Clone)]
pub struct PresenceUpdate {
  /// Fingerprint 40 hex du participant concerné.
  pub fp: String,
  /// Nouvel état de présence.
  pub status: PresenceStatus,
}

/// Agrégateur en RAM de l'état de présence de tous les participants connus.
///
/// Mis à jour par [`PresenceTracker::apply`] à chaque [`PresenceUpdate`] reçu.
/// Interrogé par la vue UI pour afficher les badges Online/Offline.
#[derive(Debug, Default)]
pub struct PresenceTracker {
  /// État courant par fingerprint 40 hex.
  statuses: HashMap<String, PresenceStatus>,
}

impl PresenceTracker {
  /// Crée un tracker vide.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Applique une mise à jour de présence.
  pub fn apply(&mut self, update: PresenceUpdate) {
    self.statuses.insert(update.fp, update.status);
  }

  /// Retourne l'état de présence d'un participant, ou `None` s'il est inconnu.
  #[must_use]
  pub fn get(&self, fp: &str) -> Option<&PresenceStatus> {
    self.statuses.get(fp)
  }

  /// Retourne `true` si le participant est en ligne.
  #[must_use]
  pub fn is_online(&self, fp: &str) -> bool {
    matches!(self.statuses.get(fp), Some(PresenceStatus::Online))
  }

  /// Marque tous les participants comme hors-ligne.
  ///
  /// Appelé lors d'une déconnexion MQTT pour éviter des données périmées.
  pub fn mark_all_offline(&mut self) {
    for status in self.statuses.values_mut() {
      *status = PresenceStatus::Offline;
    }
  }

  /// Retourne le topic de présence pour un fingerprint donné.
  ///
  /// Forme : `pgpilot/presence/{fp[..16]}`
  #[must_use]
  pub fn presence_topic(fp: &str) -> String {
    let prefix = if fp.len() >= 16 { &fp[..16] } else { fp };
    format!("{PRESENCE_TOPIC_PREFIX}/{prefix}")
  }

  /// Désérialise un payload de présence reçu depuis MQTT.
  ///
  /// Retourne `None` si le payload est invalide (non bloquant).
  #[must_use]
  pub fn decode_payload(fp: &str, bytes: &[u8]) -> Option<PresenceUpdate> {
    let status = match bytes {
      b"online" => PresenceStatus::Online,
      b"offline" => PresenceStatus::Offline,
      _ => return None,
    };
    Some(PresenceUpdate {
      fp: fp.to_string(),
      status,
    })
  }

  /// Retourne le payload heartbeat (bytes à publier).
  #[must_use]
  pub fn online_payload() -> &'static [u8] {
    b"online"
  }

  /// Retourne le payload LWT (bytes à configurer comme Last Will).
  #[must_use]
  pub fn offline_payload() -> &'static [u8] {
    b"offline"
  }
}

// ---------------------------------------------------------------------------
// Fonctions standalone de publication / subscription de présence
// ---------------------------------------------------------------------------

/// Publie le statut online sur le topic de présence (retained = true, QoS 0).
///
/// Appelé à la connexion MQTT réussie pour chaque fingerprint local actif.
///
/// # Errors
///
/// Retourne une [`crate::chat::ChatError`] si l'envoi de la commande MQTT échoue.
pub async fn publish_online(
  handle: &crate::chat::MqttHandle,
  fp: &str,
) -> crate::chat::ChatResult<()> {
  use crate::chat::mqtt::ChatTransport as _;
  let topic = PresenceTracker::presence_topic(fp);
  handle
    .publish(&topic, PresenceTracker::online_payload().to_vec(), 0, true)
    .await
}

/// Publie le statut offline sur le topic de présence (retained = true, QoS 0).
///
/// Utilisé pour une déconnexion propre. La déconnexion brutale est gérée
/// automatiquement par le Last Will Testament configuré dans [`crate::chat::MqttHandle::spawn`].
///
/// # Errors
///
/// Retourne une [`crate::chat::ChatError`] si l'envoi de la commande MQTT échoue.
pub async fn publish_offline(
  handle: &crate::chat::MqttHandle,
  fp: &str,
) -> crate::chat::ChatResult<()> {
  use crate::chat::mqtt::ChatTransport as _;
  let topic = PresenceTracker::presence_topic(fp);
  handle
    .publish(&topic, PresenceTracker::offline_payload().to_vec(), 0, true)
    .await
}

/// Souscrit aux topics de présence de tous les participants d'un salon
/// (en excluant `room.my_fp`).
///
/// # Errors
///
/// Retourne une [`crate::chat::ChatError`] dès la première souscription qui échoue.
pub async fn subscribe_room_presence(
  handle: &crate::chat::MqttHandle,
  room: &crate::chat::Room,
) -> crate::chat::ChatResult<()> {
  use crate::chat::mqtt::ChatTransport as _;
  for participant in &room.participants {
    if participant.fp == room.my_fp {
      continue;
    }
    let topic = PresenceTracker::presence_topic(&participant.fp);
    handle.subscribe(&topic, 0).await?;
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tracker_apply_and_query() {
    let mut tracker = PresenceTracker::new();
    assert!(!tracker.is_online("A".repeat(40).as_str()));

    tracker.apply(PresenceUpdate {
      fp: "A".repeat(40),
      status: PresenceStatus::Online,
    });
    assert!(tracker.is_online(&"A".repeat(40)));

    tracker.apply(PresenceUpdate {
      fp: "A".repeat(40),
      status: PresenceStatus::Offline,
    });
    assert!(!tracker.is_online(&"A".repeat(40)));
  }

  #[test]
  fn mark_all_offline() {
    let mut tracker = PresenceTracker::new();
    tracker.apply(PresenceUpdate {
      fp: "B".repeat(40),
      status: PresenceStatus::Online,
    });
    tracker.mark_all_offline();
    assert!(!tracker.is_online(&"B".repeat(40)));
  }

  #[test]
  fn presence_topic_format() {
    let fp = "ABCDEF0123456789ABCDEF0123456789ABCDEF01";
    let topic = PresenceTracker::presence_topic(fp);
    assert_eq!(topic, "pgpilot/presence/ABCDEF0123456789");
  }

  #[test]
  fn decode_payload_valid() {
    let fp = "C".repeat(40);
    let update = PresenceTracker::decode_payload(&fp, b"online").unwrap();
    assert_eq!(update.status, PresenceStatus::Online);

    let update = PresenceTracker::decode_payload(&fp, b"offline").unwrap();
    assert_eq!(update.status, PresenceStatus::Offline);
  }

  #[test]
  fn decode_payload_invalid() {
    assert!(PresenceTracker::decode_payload(&"D".repeat(40), b"unknown").is_none());
  }
}
