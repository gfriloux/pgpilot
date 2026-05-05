#![allow(dead_code)]
//! Handlers du sous-système de chat PGP (v0.6.0).
//!
//! Ce module contient tous les handlers `on_*` liés au chat. L'orchestration
//! (qui appelle quoi quand) vit ici ; les types de données vivent dans
//! `src/chat/`.
//!
//! ## Conventions
//!
//! - Tout I/O bloquant passe par `blocking_task`.
//! - Les messages non déchiffrables sont ignorés silencieusement (log interne).
//! - Un seul contexte crypto par session (`chat_crypto: Option<Arc<ChatCryptoCtx>>`).

use std::sync::Arc;

use iced::Task;

use crate::chat::{
  AckStatus, ChatCryptoCtx, ChatMessage, MqttConfig, MqttEvent, MqttHandle, PresenceTracker,
  PresenceUpdate, Room, RoomStore, WireAck, WireMessage, ACK_TOPIC_PREFIX, CHAT_TOPIC_PREFIX,
  MAX_MESSAGES_PER_ROOM, PRESENCE_TOPIC_PREFIX,
};

use super::{blocking_task, App, ChatNewForm, Message, MqttState, PendingOp, StatusKind, View};

// ---------------------------------------------------------------------------
// Constante interne
// ---------------------------------------------------------------------------

/// Nombre de chars max pour un client_id MQTT (limite protocole 3.1).
const MQTT_CLIENT_ID_LEN: usize = 23;

// ---------------------------------------------------------------------------
// impl App — handlers chat
// ---------------------------------------------------------------------------

impl App {
  // -------------------------------------------------------------------------
  // Cycle de vie chat
  // -------------------------------------------------------------------------

  /// Idempotent. Démarre la connexion MQTT si absente, charge le contexte
  /// cryptographique si absent, et abonne le client aux topics nécessaires.
  ///
  /// Appelé par `on_nav_changed` lors d'un `View::ChatRoom`.
  pub(super) fn ensure_chat_started(&mut self, room_id: String) -> Task<Message> {
    // 1. Démarrer la connexion MQTT si elle n'existe pas encore.
    let mqtt_task = if self.mqtt.is_none() {
      // Trouver la room pour obtenir le relay.
      let relay = self
        .room_by_id(&room_id)
        .map(|r| r.relay.clone())
        .or_else(|| self.config.mqtt_default_relay.clone())
        // HiveMQ public broker — Let's Encrypt cert, compatible with webpki-roots.
        // test.mosquitto.org:8883 uses a v1 Mosquitto CA cert rejected by rustls.
        .unwrap_or_else(|| "mqtts://broker.hivemq.com:8883".to_string());

      // Déterminer le fingerprint local pour cette room.
      let presence_fp = self
        .room_by_id(&room_id)
        .map(|r| r.my_fp.clone())
        .or_else(|| self.config.chat_local_fp.clone())
        .unwrap_or_default();

      // Construire un client_id court (tronqué à 23 chars).
      let client_id: String = format!("pgpilot-{}", &presence_fp[..presence_fp.len().min(14)])
        .chars()
        .take(MQTT_CLIENT_ID_LEN)
        .collect();

      let config = MqttConfig {
        relay,
        client_id,
        presence_fp,
      };

      match MqttHandle::spawn(config) {
        Ok(handle) => {
          self.mqtt = Some(handle);
          self.mqtt_state = MqttState::Connecting;
          // Le stream MQTT est branché via App::subscription — pas de Task ici.
          Task::none()
        }
        Err(e) => {
          self.mqtt_state = MqttState::Failed(e.to_string());
          self.set_status(StatusKind::Error, format!("MQTT : {e}"))
        }
      }
    } else {
      Task::none()
    };

    // 2. Charger le contexte crypto si absent.
    let crypto_task = if self.chat_crypto.is_none() {
      let local_fp = self
        .room_by_id(&room_id)
        .map(|r| r.my_fp.clone())
        .or_else(|| self.config.chat_local_fp.clone())
        .or_else(|| {
          self
            .keys
            .iter()
            .find(|k| k.has_secret)
            .map(|k| k.fingerprint.clone())
        })
        .unwrap_or_default();

      let peer_fps: Vec<String> = self
        .room_by_id(&room_id)
        .map(|r| r.participants.iter().map(|p| p.fp.clone()).collect())
        .unwrap_or_default();

      Task::perform(
        blocking_task(move || {
          ChatCryptoCtx::load(&local_fp, &peer_fps)
            .map(Arc::new)
            .map_err(|e| anyhow::anyhow!("{e}"))
        }),
        Message::MqttCryptoLoaded,
      )
    } else {
      // Abonner aux topics de la room nouvellement sélectionnée.
      self.subscribe_all_known_topics()
    };

    Task::batch([mqtt_task, crypto_task])
  }

  /// Abonne le client MQTT aux topics de toutes les rooms connues et à la
  /// présence de leurs participants. Appelé au reconnect et après chargement
  /// du contexte crypto.
  fn subscribe_all_known_topics(&self) -> Task<Message> {
    let Some(mqtt) = &self.mqtt else {
      return Task::none();
    };

    let mut topics: Vec<(String, u8)> = Vec::new();
    for room in &self.rooms {
      // Topic de chat pour cette room (QoS 1).
      topics.push((room.chat_topic(), 1));
      // Topics de présence des participants (QoS 0).
      for p in &room.participants {
        let presence_topic = PresenceTracker::presence_topic(&p.fp);
        topics.push((presence_topic, 0));
      }
    }

    // Topic de présence local (QoS 0).
    if let Some(local_fp) = self.config.chat_local_fp.as_deref() {
      topics.push((PresenceTracker::presence_topic(local_fp), 0));
    }

    let mqtt = mqtt.clone();
    Task::perform(
      async move {
        use crate::chat::mqtt::ChatTransport as _;
        for (topic, qos) in topics {
          let _ = mqtt.subscribe(&topic, qos).await;
        }
        Ok::<(), String>(())
      },
      |_| Message::ChatAckSent(Ok(())), // réutilise ChatAckSent comme no-op
    )
  }

  // -------------------------------------------------------------------------
  // Identité

  /// Confirme l'identité sélectionnée dans le modal, sauvegarde en config,
  /// puis navigue vers la room si room_id non vide.
  pub(super) fn on_chat_identity_confirm(&mut self) -> Task<Message> {
    let Some(PendingOp::IdentitySelection {
      room_id,
      selected_fp: Some(fp),
    }) = self.pending.take()
    else {
      return Task::none();
    };

    self.config.chat_local_fp = Some(fp);
    let _ = self.config.save();

    if room_id.is_empty() {
      // Sélection globale — rester sur ChatList.
      Task::none()
    } else {
      self.on_chat_room_selected(room_id)
    }
  }

  // -------------------------------------------------------------------------
  // Création / jointure
  // -------------------------------------------------------------------------

  /// Traite le formulaire de création de salon.
  pub(super) fn on_chat_room_create(&mut self) -> Task<Message> {
    let name = self.chat_new_form.name.trim().to_string();
    let relay = self.chat_new_form.relay.trim().to_string();
    let selected = self.chat_new_form.selected_participants.clone();

    if name.is_empty() {
      return self.set_status(StatusKind::Error, "Le nom du salon est requis.".to_string());
    }
    if relay.is_empty() {
      return self.set_status(
        StatusKind::Error,
        "L'URL du broker MQTT est requise.".to_string(),
      );
    }

    // Fingerprint local : identité choisie dans le formulaire.
    let my_fp = self.chat_new_form.my_fp.clone().unwrap_or_default();

    if my_fp.is_empty() {
      return self.set_status(
        StatusKind::Error,
        "Please select your identity for this room.".to_string(),
      );
    }

    if my_fp.is_empty() {
      return self.set_status(
        StatusKind::Error,
        "Aucune clef privée disponible pour le chat.".to_string(),
      );
    }

    Task::perform(
      blocking_task(move || {
        use crate::chat::RoomParticipant;
        use chrono::Utc;
        use uuid::Uuid;

        let now = Utc::now();
        let room_id = Uuid::new_v4().to_string();

        // Parser les participants depuis le textarea.
        let mut participants: Vec<RoomParticipant> = vec![RoomParticipant {
          fp: my_fp.clone(),
          joined_at: now,
        }];

        for fp in &selected {
          let fp = fp.trim().to_string();
          if fp.len() == 40 && fp.chars().all(|c| c.is_ascii_hexdigit()) && fp != my_fp {
            participants.push(RoomParticipant { fp, joined_at: now });
          }
        }

        let room = Room {
          id: room_id,
          name,
          relay,
          my_fp,
          created_at: now,
          participants,
        };

        // Persister dans rooms.yaml.
        let mut store = RoomStore::load().unwrap_or_default();
        store.upsert(room.clone());
        store.save().map_err(|e| anyhow::anyhow!("{e}"))?;

        Ok(room)
      }),
      Message::ChatRoomCreated,
    )
  }

  /// Traite le résultat de la création de salon.
  pub(super) fn on_chat_room_created(&mut self, r: Result<Room, String>) -> Task<Message> {
    match r {
      Ok(room) => {
        let room_id = room.id.clone();
        // Réinitialiser le formulaire.
        self.chat_new_form = ChatNewForm {
          relay: self.config.mqtt_default_relay.clone().unwrap_or_default(),
          ..ChatNewForm::default()
        };
        // Ajouter la room à la liste en RAM.
        if !self.rooms.iter().any(|r| r.id == room.id) {
          self.rooms.push(room);
        }
        // Naviguer vers la room.
        self.on_nav_changed(View::ChatRoom(room_id))
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        format!("Impossible de créer le salon : {e}"),
      ),
    }
  }

  /// Traite le formulaire de jointure via join code.
  pub(super) fn on_chat_room_join(&mut self) -> Task<Message> {
    let code_str = self.chat_new_form.join_code.trim().to_string();

    if code_str.is_empty() {
      return self.set_status(
        StatusKind::Error,
        "Le code d'invitation est requis.".to_string(),
      );
    }

    let my_fp = self.chat_new_form.my_fp.clone().unwrap_or_default();

    if my_fp.is_empty() {
      return self.set_status(
        StatusKind::Error,
        "Please select your identity for this room.".to_string(),
      );
    }

    Task::perform(
      blocking_task(move || {
        use crate::chat::rooms::JoinCode;
        use crate::chat::RoomParticipant;
        use chrono::Utc;

        let join_code = JoinCode::decode(&code_str).map_err(|e| anyhow::anyhow!("{e}"))?;

        // La vérification de signature est déléguée à axe 4 (crypto).
        // Pour l'instant on accepte le join code sans vérifier la sig.
        // TODO(axe4): join_code.verify()?;

        let now = Utc::now();
        let room = Room {
          id: join_code.room_id.clone(),
          name: join_code
            .room_name
            .unwrap_or_else(|| "Salon partagé".to_string()),
          relay: join_code.relay,
          my_fp: my_fp.clone(),
          created_at: now,
          participants: vec![
            RoomParticipant {
              fp: join_code.invited_by,
              joined_at: now,
            },
            RoomParticipant {
              fp: my_fp,
              joined_at: now,
            },
          ],
        };

        let mut store = RoomStore::load().unwrap_or_default();
        store.upsert(room.clone());
        store.save().map_err(|e| anyhow::anyhow!("{e}"))?;

        Ok(room)
      }),
      Message::ChatRoomJoined,
    )
  }

  /// Traite le résultat de la jointure.
  pub(super) fn on_chat_room_joined(&mut self, r: Result<Room, String>) -> Task<Message> {
    match r {
      Ok(room) => {
        let room_id = room.id.clone();
        self.chat_new_form.join_code.clear();
        if !self.rooms.iter().any(|r| r.id == room.id) {
          self.rooms.push(room);
        }
        self.on_nav_changed(View::ChatRoom(room_id))
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        format!("Impossible de rejoindre le salon : {e}"),
      ),
    }
  }

  /// Sélectionne un salon et navigue vers lui.
  pub(super) fn on_chat_room_selected(&mut self, room_id: String) -> Task<Message> {
    self.on_nav_changed(View::ChatRoom(room_id))
  }

  /// Demande la confirmation de départ d'un salon.
  pub(super) fn on_chat_room_leave(&mut self, room_id: String) -> Task<Message> {
    Task::perform(
      blocking_task(move || {
        let mut store = RoomStore::load().unwrap_or_default();
        store.remove(&room_id);
        store.save().map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(room_id)
      }),
      Message::ChatRoomLeft,
    )
  }

  /// Traite le résultat du départ de salon.
  pub(super) fn on_chat_room_left(&mut self, r: Result<String, String>) -> Task<Message> {
    match r {
      Ok(room_id) => {
        self.rooms.retain(|r| r.id != room_id);
        self.chat_messages.remove(&room_id);
        if self.active_room.as_deref() == Some(&room_id) {
          self.active_room = None;
        }
        self.on_nav_changed(View::ChatList)
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        format!("Impossible de quitter le salon : {e}"),
      ),
    }
  }

  // -------------------------------------------------------------------------
  // Envoi / réception
  // -------------------------------------------------------------------------

  /// Chiffre et envoie le message courant dans la room active.
  pub(super) fn on_chat_send(&mut self) -> Task<Message> {
    let text = self.chat_input.trim().to_string();
    if text.is_empty() {
      return Task::none();
    }

    let Some(room_id) = self.active_room.clone() else {
      return Task::none();
    };

    let Some(room) = self.room_by_id(&room_id) else {
      return Task::none();
    };

    let Some(mqtt) = self.mqtt.clone() else {
      return self.set_status(StatusKind::Error, "MQTT non connecté.".to_string());
    };

    let Some(crypto) = self.chat_crypto.clone() else {
      return self.set_status(StatusKind::Error, "Contexte crypto non chargé.".to_string());
    };

    let recipient_fps: Vec<String> = room.participants.iter().map(|p| p.fp.clone()).collect();
    let my_fp = room.my_fp.clone();
    let chat_topic = room.chat_topic();
    self.chat_input.clear();

    Task::perform(
      blocking_task(move || {
        use crate::chat::MessageDirection;
        use chrono::Utc;
        use uuid::Uuid;

        let now = Utc::now();
        let msg_id = Uuid::new_v4().to_string();

        // Chiffrement + signature.
        let payload = crypto
          .encrypt_for_room(&text, &recipient_fps)
          .map_err(|e| anyhow::anyhow!("{e}"))?;

        // Construire le WireMessage.
        let wire = WireMessage {
          id: msg_id.clone(),
          sender: my_fp.clone(),
          ts: now.timestamp(),
          payload: payload.ciphertext_armored.clone(),
          signature: payload.signature_armored.clone(),
        };

        // Sérialiser + valider taille.
        let bytes = wire.to_json_bytes().map_err(|e| anyhow::anyhow!("{e}"))?;

        // Publier sur MQTT via le handle (cmd_tx est synchrone, pas besoin de block_on).
        // MqttHandle::publish envoie une MqttCmd::Publish sur le canal unbounded — c'est
        // synchrone côté émetteur (le vrai MQTT est dans la tâche tokio).
        {
          use crate::chat::mqtt::ChatTransport as _;
          let rt = tokio::runtime::Handle::try_current()
            .map_err(|e| anyhow::anyhow!("Runtime tokio manquant: {e}"))?;
          rt.block_on(mqtt.publish(&chat_topic, bytes, 1, false))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        }

        // Construire le ChatMessage local (Sent).
        let chat_msg = ChatMessage {
          id: msg_id,
          sender_fp: my_fp,
          text,
          ts: now,
          received_at: now,
          direction: MessageDirection::Sent,
          acks: std::collections::HashMap::new(),
        };

        Ok(chat_msg)
      }),
      Message::ChatSent,
    )
  }

  /// Traite le résultat de l'envoi.
  pub(super) fn on_chat_sent(&mut self, r: Result<ChatMessage, String>) -> Task<Message> {
    match r {
      Ok(msg) => {
        if let Some(room_id) = self.active_room.clone() {
          self.push_chat_message(&room_id, msg);
        }
        Task::none()
      }
      Err(e) => self.set_status(StatusKind::Error, format!("Envoi échoué : {e}")),
    }
  }

  /// Traite un message reçu déchiffré.
  pub(super) fn on_chat_received(&mut self, room_id: String, msg: ChatMessage) -> Task<Message> {
    let msg_id = msg.id.clone();
    self.push_chat_message(&room_id, msg);

    // Envoyer un ACK best-effort via la fonction standalone publish_ack.
    let Some(handle) = self.mqtt.clone() else {
      return Task::none();
    };

    let my_fp = self
      .room_by_id(&room_id)
      .map(|r| r.my_fp.clone())
      .or_else(|| self.config.chat_local_fp.clone())
      .unwrap_or_default();

    Task::perform(
      async move {
        crate::chat::publish_ack(&handle, &msg_id, &my_fp)
          .await
          .map_err(|e| e.to_string())
      },
      Message::ChatAckSent,
    )
  }

  // -------------------------------------------------------------------------
  // Join code
  // -------------------------------------------------------------------------

  /// Encode et copie le join code du salon dans le presse-papier.
  pub(super) fn on_chat_join_code_copy(&mut self, room_id: String) -> Task<Message> {
    let Some(room) = self.room_by_id(&room_id) else {
      return Task::none();
    };

    // Pour signer le join code, il faudrait le contexte crypto (axe 4).
    // Pour l'instant on produit un code non signé avec une sig vide.
    // TODO(axe4): signer le join code via ChatCryptoCtx.
    let join_code = crate::chat::rooms::JoinCode {
      room_id: room.id.clone(),
      relay: room.relay.clone(),
      invited_by: room.my_fp.clone(),
      room_name: Some(room.name.clone()),
      sig: String::new(), // TODO(axe4): signature réelle
    };

    Task::perform(
      async move {
        let encoded = join_code.encode().map_err(|e| e.to_string())?;
        Ok::<String, String>(encoded)
      },
      Message::ChatJoinCodeCopied,
    )
  }

  /// Traite le résultat de la copie du join code.
  pub(super) fn on_chat_join_code_copied(&mut self, r: Result<String, String>) -> Task<Message> {
    match r {
      Ok(code) => {
        // Copier dans le presse-papier via le handler existant.
        self.on_copy_to_clipboard(code)
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        format!("Impossible de générer le code : {e}"),
      ),
    }
  }

  // -------------------------------------------------------------------------
  // MQTT infra
  // -------------------------------------------------------------------------

  /// Route les évènements MQTT vers les handlers applicatifs.
  pub(super) fn on_mqtt_event(&mut self, event: MqttEvent) -> Task<Message> {
    match event {
      MqttEvent::Connected => {
        self.mqtt_state = MqttState::Connected;
        // Ré-abonner aux topics et publier la présence online pour chaque
        // fingerprint local connu dans les rooms actives.
        let subscribe_task = self.subscribe_all_known_topics();

        let online_task = if let Some(handle) = self.mqtt.clone() {
          // Collecter les fingerprints locaux uniques à annoncer.
          let mut fps: Vec<String> = self.rooms.iter().map(|r| r.my_fp.clone()).collect();
          fps.sort_unstable();
          fps.dedup();

          Task::perform(
            async move {
              for fp in &fps {
                let _ = crate::chat::publish_online(&handle, fp).await;
              }
              Ok::<(), String>(())
            },
            |_| Message::ChatAckSent(Ok(())),
          )
        } else {
          Task::none()
        };

        Task::batch([subscribe_task, online_task])
      }
      MqttEvent::Disconnected(reason) => {
        // Marquer tous les participants hors-ligne.
        self.presence.mark_all_offline();
        if matches!(
          self.mqtt_state,
          MqttState::Connected | MqttState::Connecting
        ) {
          self.mqtt_state = MqttState::Reconnecting { attempt: 1 };
        }
        // Rendre l'erreur visible pour faciliter le diagnostic.
        eprintln!("[MQTT] Disconnected: {reason}");
        self.set_status(StatusKind::Error, format!("MQTT: {reason}"))
      }
      MqttEvent::Reconnecting { attempt } => {
        self.mqtt_state = MqttState::Reconnecting { attempt };
        Task::none()
      }
      MqttEvent::MessageReceived { topic, payload } => self.dispatch_mqtt_payload(topic, payload),
    }
  }

  /// Stocke le contexte crypto chargé et abonne aux topics.
  pub(super) fn on_mqtt_crypto_loaded(
    &mut self,
    r: Result<Arc<ChatCryptoCtx>, String>,
  ) -> Task<Message> {
    match r {
      Ok(ctx) => {
        self.chat_crypto = Some(ctx);
        // Maintenant que le crypto est chargé, abonner aux topics.
        self.subscribe_all_known_topics()
      }
      Err(e) => {
        self.mqtt_state = MqttState::Failed(e.clone());
        self.set_status(
          StatusKind::Error,
          format!("Contexte crypto inutilisable : {e}"),
        )
      }
    }
  }

  /// Routage interne : déchiffre selon le préfixe topic.
  ///
  /// ```text
  /// pgpilot/chat/{hash}      → decrypt → Message::ChatReceived
  /// pgpilot/presence/{fp16}  → decode  → Message::PresenceUpdated
  /// pgpilot/ack/{msg_id16}   → parse   → Message::ChatAckReceived
  /// ```
  fn dispatch_mqtt_payload(&self, topic: String, payload: Vec<u8>) -> Task<Message> {
    if topic.starts_with(CHAT_TOPIC_PREFIX) {
      // Trouver la room correspondant au topic.
      let room_id = self
        .rooms
        .iter()
        .find(|r| r.chat_topic() == topic)
        .map(|r| r.id.clone());

      let Some(room_id) = room_id else {
        // Topic inconnu — race au reconnect, on jette.
        return Task::none();
      };

      let Some(crypto) = self.chat_crypto.clone() else {
        return Task::none();
      };

      let my_fp = self
        .room_by_id(&room_id)
        .map(|r| r.my_fp.clone())
        .unwrap_or_default();

      return Task::perform(
        blocking_task(move || {
          use crate::chat::{ChatPayload, MessageDirection};
          use chrono::Utc;

          let wire = WireMessage::from_json_bytes(&payload).map_err(|e| anyhow::anyhow!("{e}"))?;

          // Ignorer nos propres messages (déjà dans la liste côté Sent).
          if wire.sender.to_uppercase() == my_fp.to_uppercase() {
            anyhow::bail!("own_message"); // signal interne
          }

          let chat_payload = ChatPayload {
            ciphertext_armored: wire.payload.clone(),
            signature_armored: wire.signature.clone(),
          };

          let verified = crypto
            .decrypt_message(&chat_payload)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

          let now = Utc::now();
          let msg = ChatMessage {
            id: wire.id,
            sender_fp: wire.sender,
            text: verified.plaintext,
            ts: chrono::DateTime::<chrono::Utc>::from_timestamp(wire.ts, 0).unwrap_or(now),
            received_at: now,
            direction: MessageDirection::Received,
            acks: std::collections::HashMap::new(),
          };

          Ok((room_id, msg))
        }),
        |r: Result<(String, ChatMessage), String>| match r {
          Ok((rid, msg)) => Message::ChatReceived(rid, msg),
          Err(_) => {
            // Message non déchiffrable ou propre message : ignoré silencieusement.
            Message::ChatAckSent(Ok(())) // no-op
          }
        },
      );
    }

    if topic.starts_with(PRESENCE_TOPIC_PREFIX) {
      // Extraire le fp16 du topic.
      let fp16 = topic
        .strip_prefix(&format!("{PRESENCE_TOPIC_PREFIX}/"))
        .unwrap_or("")
        .to_string();

      // Trouver le fingerprint complet dans les rooms.
      let full_fp = self
        .rooms
        .iter()
        .flat_map(|r| r.participants.iter())
        .find(|p| p.fp.starts_with(&fp16) || p.fp[..p.fp.len().min(16)] == fp16)
        .map(|p| p.fp.clone())
        .unwrap_or(fp16);

      if let Some(update) = PresenceTracker::decode_payload(&full_fp, &payload) {
        return Task::perform(async move { update }, Message::PresenceUpdated);
      }
      return Task::none();
    }

    if topic.starts_with(ACK_TOPIC_PREFIX) {
      if let Ok(ack) = WireAck::from_json_bytes(&payload) {
        // Trouver la room qui contient ce msg_id.
        let room_id = self
          .chat_messages
          .iter()
          .find(|(_, msgs)| {
            msgs.iter().any(|m| {
              m.id.starts_with(&ack.msg_id[..ack.msg_id.len().min(16)])
                || ack.msg_id.starts_with(&m.id[..m.id.len().min(16)])
                || m.id == ack.msg_id
            })
          })
          .map(|(rid, _)| rid.clone());

        if let Some(rid) = room_id {
          return Task::perform(
            async move { (rid, ack.msg_id, ack.from) },
            |(rid, mid, from)| Message::ChatAckReceived(rid, mid, from),
          );
        }
      }
    }

    Task::none()
  }

  // -------------------------------------------------------------------------
  // Présence
  // -------------------------------------------------------------------------

  /// Applique une mise à jour de présence.
  pub(super) fn on_presence_updated(&mut self, update: PresenceUpdate) -> Task<Message> {
    self.presence.apply(update);
    Task::none()
  }

  // -------------------------------------------------------------------------
  // ACK
  // -------------------------------------------------------------------------

  /// Met à jour l'état ACK d'un message.
  pub(super) fn on_chat_ack_received(
    &mut self,
    room_id: String,
    msg_id: String,
    sender_fp: String,
  ) -> Task<Message> {
    if let Some(msgs) = self.chat_messages.get_mut(&room_id) {
      for msg in msgs.iter_mut() {
        if msg.id == msg_id || msg.id.starts_with(&msg_id[..msg_id.len().min(16)]) {
          msg.acks.insert(sender_fp.clone(), AckStatus::Received);
          break;
        }
      }
    }
    Task::none()
  }

  /// Traite le résultat de la publication d'un ACK.
  pub(super) fn on_chat_ack_sent(&mut self, r: Result<(), String>) -> Task<Message> {
    // ACK best-effort : on log l'erreur mais on ne bloque pas.
    if let Err(_e) = r {
      // Log interne uniquement — pas d'affichage UI.
    }
    Task::none()
  }

  // -------------------------------------------------------------------------
  // Helpers privés
  // -------------------------------------------------------------------------

  /// Insère un message en RAM et applique le bornage `MAX_MESSAGES_PER_ROOM` (FIFO).
  fn push_chat_message(&mut self, room_id: &str, msg: ChatMessage) {
    let queue = self.chat_messages.entry(room_id.to_string()).or_default();
    if queue.len() >= MAX_MESSAGES_PER_ROOM {
      queue.pop_front();
    }
    queue.push_back(msg);
  }

  /// Cherche un salon par identifiant (équivalent de `key_by_fp`).
  fn room_by_id(&self, id: &str) -> Option<&Room> {
    self.rooms.iter().find(|r| r.id == id)
  }
}

// ---------------------------------------------------------------------------
// Extension de l'import pour on_nav_changed
// ---------------------------------------------------------------------------

// NOTE : on_nav_changed est dans nav.rs — l'extension pour les vues chat
// est appliquée directement dans nav.rs (Mission 5).
