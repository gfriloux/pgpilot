#![allow(dead_code)]
//! Client MQTT asynchrone pour le chat PGPilot.
//!
//! Ce module expose :
//! - [`ChatTransport`] — trait d'abstraction permettant le mock dans les tests.
//! - [`MqttConfig`] — paramètres de connexion.
//! - [`MqttHandle`] — handle cloneable vers la tâche tokio propriétaire du
//!   client rumqttc.
//! - [`MqttEvent`] — évènements remontés vers l'UI.
//! - [`MqttCmd`] — commandes envoyées depuis l'UI vers la tâche tokio.
//! - [`subscription`] — factory d'un [`iced::Subscription`] à partir d'un
//!   [`MqttHandle`].
//!
//! ## Architecture
//!
//! Une seule tâche tokio possède le client rumqttc (`AsyncClient` + `EventLoop`).
//! Aucun `Arc<Mutex<AsyncClient>>` n'est exposé à l'extérieur — cela évite les
//! deadlocks entre l'eventloop et les publishers.
//!
//! La communication bi-directionnelle se fait via deux canaux mpsc :
//! - `cmd_tx` (UI → tâche) : illimité, commandes rares.
//! - `event_tx` (tâche → UI) : borné à [`MQTT_EVENT_CHANNEL_CAP`] ; au-delà,
//!   les évènements sont droppés pour éviter l'accumulation sur une UI gelée.

use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rumqttc::{AsyncClient, EventLoop, LastWill, MqttOptions, QoS, Transport};
use rustls::ClientConfig;
use tokio::sync::{mpsc, Mutex};

use crate::chat::{
  ChatError, ChatResult, MQTT_EVENT_CHANNEL_CAP, MQTT_KEEPALIVE_SECS, MQTT_RECONNECT_BASE_MS,
  MQTT_RECONNECT_LOG_EVERY, MQTT_RECONNECT_MAX_MS, PRESENCE_TOPIC_PREFIX,
};

// ---------------------------------------------------------------------------
// Trait d'abstraction du transport
// ---------------------------------------------------------------------------

/// Abstraction du transport MQTT permettant le mock dans les tests unitaires.
///
/// [`MqttHandle`] est l'implémentation de production. Les tests injectent un
/// `MockTransport` implémentant ce trait.
#[async_trait]
pub trait ChatTransport: Send + Sync {
  /// Souscrit à un topic MQTT avec le niveau QoS indiqué (0, 1 ou 2).
  async fn subscribe(&self, topic: &str, qos: u8) -> ChatResult<()>;

  /// Se désabonne d'un topic MQTT.
  async fn unsubscribe(&self, topic: &str) -> ChatResult<()>;

  /// Publie un message sur un topic MQTT.
  async fn publish(&self, topic: &str, payload: Vec<u8>, qos: u8, retain: bool) -> ChatResult<()>;
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Paramètres de connexion MQTT.
///
/// `relay` accepte `mqtts://host:8883` (TLS) ou `mqtt://host:1883` (plain).
/// Le plain-text est refusé sauf si l'hôte est `localhost` ou `127.x.x.x`
/// (usage dev uniquement).
#[derive(Debug, Clone)]
pub struct MqttConfig {
  /// URL du broker : `"mqtts://host:8883"` ou `"mqtt://host:1883"`.
  pub relay: String,
  /// Identifiant client MQTT (tronqué à 23 caractères — limite protocole).
  pub client_id: String,
  /// Fingerprint 40 hex utilisé pour le Last Will Testament.
  pub presence_fp: String,
}

// ---------------------------------------------------------------------------
// Commandes et évènements
// ---------------------------------------------------------------------------

/// Commandes envoyées depuis l'UI vers la tâche tokio MQTT.
#[derive(Debug)]
pub enum MqttCmd {
  /// Souscrit à un topic avec le niveau QoS indiqué.
  Subscribe { topic: String, qos: u8 },
  /// Se désabonne d'un topic.
  Unsubscribe { topic: String },
  /// Publie un payload sur un topic.
  Publish {
    topic: String,
    payload: Vec<u8>,
    qos: u8,
    retain: bool,
  },
  /// Arrête la tâche tokio proprement.
  Shutdown,
}

/// Évènements remontés depuis la tâche tokio MQTT vers l'UI.
#[derive(Debug, Clone)]
pub enum MqttEvent {
  /// Connexion (ou reconnexion) réussie au broker.
  Connected,
  /// Déconnexion détectée (raison incluse).
  Disconnected(String),
  /// Tentative de reconnexion en cours.
  Reconnecting {
    /// Numéro de la tentative (commence à 1).
    attempt: u32,
  },
  /// Message reçu sur un topic souscrit.
  MessageReceived {
    /// Topic sur lequel le message a été reçu.
    topic: String,
    /// Payload brut (JSON sérialisé).
    payload: Vec<u8>,
  },
}

// ---------------------------------------------------------------------------
// MqttHandle — état partagé interne
// ---------------------------------------------------------------------------

/// État partagé par toutes les copies de [`MqttHandle`].
struct MqttHandleInner {
  cmd_tx: mpsc::UnboundedSender<MqttCmd>,
  /// Receiver d'évènements, pris une seule fois par `take_event_stream`.
  event_rx: Mutex<Option<mpsc::Receiver<MqttEvent>>>,
}

// ---------------------------------------------------------------------------
// MqttHandle
// ---------------------------------------------------------------------------

/// Handle cloneable et thread-safe vers la tâche tokio MQTT.
///
/// Contient un `Arc` vers un état interne partagé. Cloneable à volonté dans
/// des `Task::perform` sans coût.
///
/// Le stream d'évènements sortant est accessible **une seule fois** via
/// [`MqttHandle::take_event_stream`] (sémantique `take`).
///
/// ## Identité pour iced `Subscription`
///
/// `MqttHandle` implémente [`Hash`] via l'adresse du pointeur `Arc` —
/// chaque session MQTT a une identité unique et stable, ce qui garantit
/// qu'iced ne relance pas la factory de subscription tant que le handle vit.
#[derive(Clone)]
pub struct MqttHandle {
  inner: Arc<MqttHandleInner>,
}

impl std::fmt::Debug for MqttHandle {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MqttHandle")
      .field("arc_ptr", &Arc::as_ptr(&self.inner))
      .finish()
  }
}

/// Identité stable basée sur l'adresse mémoire de l'Arc interne.
///
/// Deux clones du même handle ont la même identité ; deux handles distincts
/// (deux sessions MQTT) ont des identités différentes.
impl Hash for MqttHandle {
  fn hash<H: Hasher>(&self, state: &mut H) {
    Arc::as_ptr(&self.inner).hash(state);
  }
}

/// Deux clones du même handle sont égaux (même Arc interne).
impl PartialEq for MqttHandle {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.inner, &other.inner)
  }
}

impl Eq for MqttHandle {}

impl MqttHandle {
  /// Démarre la tâche tokio et retourne un handle vers elle.
  ///
  /// # Errors
  ///
  /// - [`ChatError::TlsError`] — URL `mqtt://` non-locale en production.
  /// - [`ChatError::InvalidConfig`] — URL malformée.
  pub fn spawn(config: MqttConfig) -> ChatResult<Self> {
    // Correction 8 : valider le fingerprint de présence avant tout traitement.
    if config.presence_fp.len() != 40 || !config.presence_fp.chars().all(|c| c.is_ascii_hexdigit())
    {
      return Err(ChatError::InvalidFingerprint(config.presence_fp.clone()));
    }

    let (host, port, use_tls) = parse_relay_url(&config.relay)?;

    // Tronquer le client_id à 23 caractères (limite du protocole MQTT 3.1).
    let client_id: String = config.client_id.chars().take(23).collect();

    let mut options = MqttOptions::new(&client_id, &host, port);
    options.set_keep_alive(Duration::from_secs(u64::from(MQTT_KEEPALIVE_SECS)));

    if use_tls {
      // Construire un ClientConfig rustls avec le bundle Mozilla (webpki-roots),
      // plus fiable que les certs système sur NixOS et autres distros non-standard.
      let mut root_store = rustls::RootCertStore::empty();
      root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
      let client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
      options.set_transport(Transport::Tls(rumqttc::TlsConfiguration::Rustls(Arc::new(
        client_config,
      ))));
    }

    // Last Will Testament — publié automatiquement par le broker en cas de
    // déconnexion brutale.
    // Le fingerprint a été validé à 40 chars ci-dessus ; on utilise .min()
    // pour satisfaire le borrow checker sans panic.
    let fp16 = &config.presence_fp[..16_usize.min(config.presence_fp.len())];
    let lwt_topic = format!("{PRESENCE_TOPIC_PREFIX}/{fp16}");
    let lwt = LastWill::new(lwt_topic, b"offline".to_vec(), QoS::AtLeastOnce, true);
    options.set_last_will(lwt);

    // Canaux de communication.
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<MqttCmd>();
    let (event_tx, event_rx) = mpsc::channel::<MqttEvent>(MQTT_EVENT_CHANNEL_CAP);

    // Le client rumqttc et son eventloop sont créés avec un channel interne
    // de 10 messages (suffisant pour les publishes en attente de drain).
    let (client, eventloop) = AsyncClient::new(options, 10);

    // Lancer la tâche tokio propriétaire du client.
    tokio::spawn(mqtt_task(client, eventloop, cmd_rx, event_tx));

    Ok(Self {
      inner: Arc::new(MqttHandleInner {
        cmd_tx,
        event_rx: Mutex::new(Some(event_rx)),
      }),
    })
  }

  /// Prend le receiver d'évènements (ne peut être appelé qu'une fois).
  ///
  /// Retourne `None` si le stream a déjà été pris.
  pub fn take_event_stream(&self) -> Option<mpsc::Receiver<MqttEvent>> {
    // try_lock est safe ici car la méthode est appelée lors de la construction
    // de la Subscription, jamais sur le chemin chaud de réception de messages.
    self.inner.event_rx.try_lock().ok()?.take()
  }

  /// Publie un message de façon synchrone en envoyant sur le canal `cmd_tx`.
  ///
  /// Peut être appelé depuis un `blocking_task` sans `block_on` — le canal
  /// `UnboundedSender` est synchrone côté émetteur.
  ///
  /// # Errors
  ///
  /// Retourne [`ChatError::MqttNotConnected`] si le canal interne est fermé
  /// (tâche tokio terminée).
  pub fn publish_sync(
    &self,
    topic: &str,
    payload: Vec<u8>,
    qos: u8,
    retain: bool,
  ) -> ChatResult<()> {
    self
      .inner
      .cmd_tx
      .send(MqttCmd::Publish {
        topic: topic.to_string(),
        payload,
        qos,
        retain,
      })
      .map_err(|_| ChatError::MqttNotConnected)
  }

  /// Envoie une commande d'arrêt propre à la tâche tokio.
  pub fn shutdown(&self) {
    let _ = self.inner.cmd_tx.send(MqttCmd::Shutdown);
  }
}

#[async_trait]
impl ChatTransport for MqttHandle {
  async fn subscribe(&self, topic: &str, qos: u8) -> ChatResult<()> {
    self
      .inner
      .cmd_tx
      .send(MqttCmd::Subscribe {
        topic: topic.to_string(),
        qos,
      })
      .map_err(|e| ChatError::MqttProtocolError(e.to_string()))
  }

  async fn unsubscribe(&self, topic: &str) -> ChatResult<()> {
    self
      .inner
      .cmd_tx
      .send(MqttCmd::Unsubscribe {
        topic: topic.to_string(),
      })
      .map_err(|e| ChatError::MqttProtocolError(e.to_string()))
  }

  async fn publish(&self, topic: &str, payload: Vec<u8>, qos: u8, retain: bool) -> ChatResult<()> {
    self
      .inner
      .cmd_tx
      .send(MqttCmd::Publish {
        topic: topic.to_string(),
        payload,
        qos,
        retain,
      })
      .map_err(|e| ChatError::MqttProtocolError(e.to_string()))
  }
}

// ---------------------------------------------------------------------------
// Subscription iced
// ---------------------------------------------------------------------------

/// Crée un [`iced::Subscription`] à partir d'un [`MqttHandle`].
///
/// Remonte chaque [`MqttEvent`] comme [`crate::app::Message::MqttEvent`].
///
/// Utilise [`iced::Subscription::run_with`] avec `MqttHandle` comme données
/// d'identité stables (l'adresse Arc est stable tant que le handle vit).
///
/// `build_mqtt_stream` est la fonction builder passée à `run_with` — elle ne
/// peut pas être une closure car `run_with` exige un `fn` pointer.
pub fn subscription(handle: MqttHandle) -> iced::Subscription<crate::app::Message> {
  iced::Subscription::run_with(handle, build_mqtt_stream)
}

/// Builder de stream MQTT passé à [`iced::Subscription::run_with`].
///
/// Prend un `&MqttHandle` et retourne un `Stream<Item = crate::app::Message>`
/// en consommant le receiver d'évènements tokio via `iced::stream::channel`.
fn build_mqtt_stream(
  handle: &MqttHandle,
) -> impl futures::stream::Stream<Item = crate::app::Message> {
  // On prend le receiver une seule fois (Option<mpsc::Receiver<MqttEvent>>).
  // Les appels suivants (re-rendus iced) retournent `None` → stream vide.
  let rx_opt = handle.take_event_stream();

  iced::stream::channel(MQTT_EVENT_CHANNEL_CAP, async move |mut output| {
    use futures::SinkExt as _;
    if let Some(mut rx) = rx_opt {
      while let Some(event) = rx.recv().await {
        let _ = output.send(crate::app::Message::MqttEvent(event)).await;
      }
    }
    // Maintient le stream vivant indéfiniment une fois le channel épuisé,
    // conformément au pattern websocket d'iced (évite un redémarrage spurieux
    // de la Subscription).
    futures::future::pending::<()>().await;
  })
}

// ---------------------------------------------------------------------------
// Tâche tokio interne
// ---------------------------------------------------------------------------

/// Boucle principale de la tâche tokio propriétaire du client rumqttc.
///
/// Sélectionne en concurrence :
/// - Les notifications de l'eventloop rumqttc (packets entrants, erreurs).
/// - Les commandes de l'UI (`MqttCmd`).
async fn mqtt_task(
  client: AsyncClient,
  mut eventloop: EventLoop,
  mut cmd_rx: mpsc::UnboundedReceiver<MqttCmd>,
  event_tx: mpsc::Sender<MqttEvent>,
) {
  let mut attempt: u32 = 0;
  let mut backoff = Duration::from_millis(MQTT_RECONNECT_BASE_MS);
  let max_backoff = Duration::from_millis(MQTT_RECONNECT_MAX_MS);

  loop {
    tokio::select! {
      // Branche commandes — priorité haute pour réagir vite au Shutdown.
      biased;

      cmd = cmd_rx.recv() => {
        match cmd {
          Some(MqttCmd::Subscribe { topic, qos }) => {
            let q = qos_from_u8(qos);
            let _ = client.subscribe(&topic, q).await;
          }
          Some(MqttCmd::Unsubscribe { topic }) => {
            let _ = client.unsubscribe(&topic).await;
          }
          Some(MqttCmd::Publish { topic, payload, qos, retain }) => {
            let q = qos_from_u8(qos);
            let _ = client.publish(&topic, q, retain, payload).await;
          }
          Some(MqttCmd::Shutdown) | None => break,
        }
      }

      notification = eventloop.poll() => {
        match notification {
          Ok(rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_))) => {
            attempt = 0;
            backoff = Duration::from_millis(MQTT_RECONNECT_BASE_MS);
            try_send(&event_tx, MqttEvent::Connected);
          }
          Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(p))) => {
            // Correction 6 : dropper les payloads trop longs AVANT de copier
            // en mémoire pour éviter les allocations DoS.
            if p.payload.len() > crate::chat::MAX_WIRE_MESSAGE_BYTES {
              // Ignorer silencieusement.
            } else {
              try_send(
                &event_tx,
                MqttEvent::MessageReceived {
                  topic: p.topic.clone(),
                  payload: p.payload.to_vec(),
                },
              );
            }
          }
          Ok(_) => {
            // Autres packets (SubAck, PingResp, etc.) — ignorés silencieusement.
          }
          Err(e) => {
            // rumqttc retry automatiquement — on gère seulement les méta-données
            // de progression pour l'UI.
            attempt = attempt.saturating_add(1);

            let should_log =
              attempt <= 3 || attempt.is_multiple_of(MQTT_RECONNECT_LOG_EVERY);
            if should_log {
              try_send(&event_tx, MqttEvent::Reconnecting { attempt });
            }

            // Backoff exponentiel borné.
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(max_backoff);

            try_send(&event_tx, MqttEvent::Disconnected(e.to_string()));
          }
        }
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Helpers internes
// ---------------------------------------------------------------------------

/// Convertit un niveau QoS numérique en [`QoS`] rumqttc.
///
/// Toute valeur > 2 est interprétée comme `ExactlyOnce`.
fn qos_from_u8(qos: u8) -> QoS {
  match qos {
    0 => QoS::AtMostOnce,
    1 => QoS::AtLeastOnce,
    _ => QoS::ExactlyOnce,
  }
}

/// Tente d'envoyer un [`MqttEvent`] vers l'UI.
///
/// Si le channel est plein (UI gelée), l'évènement est droppé — conforme à la
/// spec §8.3 (on drop les évènements anciens plutôt que de bloquer la tâche).
fn try_send(tx: &mpsc::Sender<MqttEvent>, event: MqttEvent) {
  let _ = tx.try_send(event);
}

/// Parse une URL de relay MQTT et retourne `(host, port, use_tls)`.
///
/// Accepte :
/// - `mqtts://host:port` → TLS obligatoire.
/// - `mqtt://localhost:port` → plain-text autorisé (dev).
/// - `mqtt://127.x.x.x:port` → plain-text autorisé (dev).
///
/// # Errors
///
/// - [`ChatError::InvalidConfig`] — URL malformée.
/// - [`ChatError::TlsError`] — `mqtt://` non-local refusé en production.
pub(crate) fn parse_relay_url(relay: &str) -> ChatResult<(String, u16, bool)> {
  let (scheme, rest) = relay
    .split_once("://")
    .ok_or_else(|| ChatError::InvalidConfig(format!("URL malformée : {relay}")))?;

  let use_tls = match scheme {
    "mqtts" => true,
    "mqtt" => false,
    other => {
      return Err(ChatError::InvalidConfig(format!(
        "Schéma inconnu : {other} (attendu mqtt:// ou mqtts://)"
      )));
    }
  };

  // Décompose host:port (utilise rsplit pour gérer les adresses IPv6 entre []).
  let (host_str, port_str) = rest
    .rsplit_once(':')
    .ok_or_else(|| ChatError::InvalidConfig(format!("Port manquant dans l'URL : {relay}")))?;

  let port: u16 = port_str
    .parse()
    .map_err(|_| ChatError::InvalidConfig(format!("Port invalide : {port_str}")))?;

  let host = host_str.to_string();

  // Refuser mqtt:// plain-text pour les hôtes distants.
  if !use_tls {
    let is_local = host == "localhost" || host.starts_with("127.") || host == "::1";
    if !is_local {
      return Err(ChatError::TlsError(format!(
        "Connexion non-chiffrée interdite vers {host}:{port} \
        (utiliser mqtts:// ou se limiter à localhost)"
      )));
    }
  }

  Ok((host, port, use_tls))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_mqtts_url() {
    let (host, port, tls) = parse_relay_url("mqtts://test.mosquitto.org:8883").expect("parse");
    assert_eq!(host, "test.mosquitto.org");
    assert_eq!(port, 8883);
    assert!(tls);
  }

  #[test]
  fn parse_mqtt_localhost() {
    let (host, port, tls) = parse_relay_url("mqtt://localhost:1883").expect("parse");
    assert_eq!(host, "localhost");
    assert_eq!(port, 1883);
    assert!(!tls);
  }

  #[test]
  fn parse_mqtt_127() {
    let (host, port, tls) = parse_relay_url("mqtt://127.0.0.1:1883").expect("parse");
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port, 1883);
    assert!(!tls);
  }

  #[test]
  fn parse_mqtt_remote_refused() {
    let err = parse_relay_url("mqtt://broker.example.com:1883");
    assert!(matches!(err, Err(ChatError::TlsError(_))));
  }

  #[test]
  fn parse_unknown_scheme() {
    let err = parse_relay_url("ws://broker.example.com:1883");
    assert!(matches!(err, Err(ChatError::InvalidConfig(_))));
  }

  #[test]
  fn parse_missing_port() {
    let err = parse_relay_url("mqtts://broker.example.com");
    assert!(matches!(err, Err(ChatError::InvalidConfig(_))));
  }

  #[test]
  fn qos_conversion() {
    assert_eq!(qos_from_u8(0), QoS::AtMostOnce);
    assert_eq!(qos_from_u8(1), QoS::AtLeastOnce);
    assert_eq!(qos_from_u8(2), QoS::ExactlyOnce);
    assert_eq!(qos_from_u8(99), QoS::ExactlyOnce);
  }
}
