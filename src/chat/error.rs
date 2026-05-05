#![allow(dead_code)]
/// Toutes les erreurs du sous-système chat.
///
/// Granularité fine pour les tests unitaires et la localisation (i18n axe 5).
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum ChatError {
  // --- Connectivité MQTT ---
  /// Client MQTT non connecté au moment de l'opération.
  #[error("MQTT non connecté")]
  MqttNotConnected,

  /// Broker réseau injoignable.
  #[error("Broker injoignable : {0}")]
  BrokerUnreachable(String),

  /// Erreur de protocole MQTT (packet malformé, etc.).
  #[error("Erreur protocole MQTT : {0}")]
  MqttProtocolError(String),

  /// Connexion TLS refusée (cert invalide, plain-text interdit, etc.).
  #[error("Erreur TLS : {0}")]
  TlsError(String),

  // --- Cryptographie ---
  /// Échec du chiffrement PGP multi-destinataires.
  #[error("Échec chiffrement : {0}")]
  EncryptFailed(String),

  /// Échec du déchiffrement PGP (clef absente, blob corrompu…).
  #[error("Échec déchiffrement : {0}")]
  DecryptFailed(String),

  /// La signature PGP du message est invalide ou ne correspond pas au contenu.
  #[error("Signature invalide")]
  SignatureInvalid,

  /// Échec de la signature (séquoia, passphrase manquante, etc.).
  #[error("Échec signature : {0}")]
  SignFailed(String),

  // --- Identité et salons ---
  /// L'émetteur du message n'est pas dans le keyring local.
  #[error("Émetteur inconnu : {0}")]
  UnknownSender(String),

  /// L'identifiant de salon demandé est introuvable dans rooms.yaml.
  #[error("Salon introuvable : {0}")]
  RoomNotFound(String),

  /// Aucune clef de signature utilisable pour l'identité locale.
  #[error("Aucune clef de signature utilisable")]
  NoSigningKey,

  /// Un participant attendu est absent du keyring local.
  #[error("Participant absent du keyring : {0}")]
  ParticipantNotInKeyring(String),

  // --- Code d'invitation (JoinCode) ---
  /// Le join code ne peut pas être décodé (base64 invalide, JSON malformé).
  #[error("Code d'invitation invalide")]
  InvalidJoinCode,

  /// La signature PGP portée par le join code est invalide.
  #[error("Signature du code d'invitation invalide")]
  JoinCodeSignatureInvalid,

  /// La clef publique de l'invitant est absente du keyring local.
  #[error("Clef de l'invitant absente du keyring")]
  JoinCodeInviterUnknown,

  // --- Validation message ---
  /// Le message sérialisé dépasse `MAX_WIRE_MESSAGE_BYTES` (64 Kio).
  #[error(
    "Message trop volumineux (max {} octets)",
    crate::chat::MAX_WIRE_MESSAGE_BYTES
  )]
  MessageTooLarge,

  /// Un message portant cet UUID a déjà été traité (dédup).
  #[error("Message dupliqué : {0}")]
  MessageIdDuplicate(String),

  /// Le JSON du WireMessage ou WireAck est malformé.
  #[error("Message wire malformé : {0}")]
  MalformedWireMessage(String),

  /// Un fingerprint reçu ne respecte pas le format 40 hex attendu.
  #[error("Fingerprint invalide : {0}")]
  InvalidFingerprint(String),

  // --- Persistance ---
  /// Lecture du fichier rooms.yaml impossible.
  #[error("Lecture rooms.yaml impossible : {0}")]
  RoomsYamlLoadFailed(String),

  /// Écriture du fichier rooms.yaml impossible.
  #[error("Écriture rooms.yaml impossible : {0}")]
  RoomsYamlSaveFailed(String),

  // --- Configuration ---
  /// Valeur de configuration invalide ou manquante.
  #[error("Configuration invalide : {0}")]
  InvalidConfig(String),
}

/// Alias de résultat pour le sous-système chat.
pub type ChatResult<T> = std::result::Result<T, ChatError>;
