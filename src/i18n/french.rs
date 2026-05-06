use super::Strings;

pub struct FrenchStrings;

impl Strings for FrenchStrings {
  fn nav_my_keys(&self) -> &'static str {
    "Mes clefs"
  }
  fn nav_public_keys(&self) -> &'static str {
    "Clefs publiques"
  }
  fn nav_import(&self) -> &'static str {
    "Importer"
  }
  fn nav_create_key(&self) -> &'static str {
    "Creer une clef"
  }
  fn nav_encrypt(&self) -> &'static str {
    "Chiffrer"
  }
  fn nav_decrypt(&self) -> &'static str {
    "Dechiffrer"
  }
  fn nav_sign(&self) -> &'static str {
    "Signer"
  }
  fn nav_verify(&self) -> &'static str {
    "Verifier"
  }
  fn nav_health(&self) -> &'static str {
    "Diagnostic"
  }
  fn nav_settings(&self) -> &'static str {
    "Parametres"
  }
  fn sidebar_section_keys(&self) -> &'static str {
    "CLEFS"
  }
  fn sidebar_section_operations(&self) -> &'static str {
    "OPERATIONS"
  }
  fn sidebar_section_tools(&self) -> &'static str {
    "OUTILS"
  }

  fn btn_ok(&self) -> &'static str {
    "OK"
  }
  fn btn_cancel(&self) -> &'static str {
    "Annuler"
  }
  fn btn_confirm(&self) -> &'static str {
    "Confirmer"
  }
  fn btn_back(&self) -> &'static str {
    "Retour"
  }
  fn btn_create(&self) -> &'static str {
    "Creer"
  }
  fn btn_delete(&self) -> &'static str {
    "Supprimer"
  }
  fn btn_export(&self) -> &'static str {
    "Exporter"
  }
  fn btn_import(&self) -> &'static str {
    "Importer"
  }
  fn btn_copy(&self) -> &'static str {
    "Copier"
  }
  fn btn_publish(&self) -> &'static str {
    "Publier"
  }
  fn btn_backup(&self) -> &'static str {
    "Sauvegarder"
  }
  fn btn_migrate(&self) -> &'static str {
    "Migrer vers la carte"
  }
  fn btn_renew(&self) -> &'static str {
    "Renouveler"
  }
  fn btn_rotate(&self) -> &'static str {
    "Remplacer"
  }
  fn btn_add_subkey(&self) -> &'static str {
    "Ajouter une sous-clef"
  }

  fn key_fingerprint(&self) -> &'static str {
    "Empreinte"
  }
  fn key_created(&self) -> &'static str {
    "Creee le"
  }
  fn key_expires(&self) -> &'static str {
    "Expire le"
  }
  fn key_never_expires(&self) -> &'static str {
    "N'expire pas"
  }
  fn key_trust(&self) -> &'static str {
    "Confiance"
  }
  fn key_subkeys(&self) -> &'static str {
    "Sous-clefs"
  }
  fn key_no_subkeys(&self) -> &'static str {
    "Aucune sous-clef"
  }

  fn trust_undefined(&self) -> &'static str {
    "Non definie"
  }
  fn trust_marginal(&self) -> &'static str {
    "Marginale"
  }
  fn trust_full(&self) -> &'static str {
    "Totale"
  }
  fn trust_ultimate(&self) -> &'static str {
    "Ultime"
  }

  fn status_key_created(&self) -> &'static str {
    "Clef creee"
  }
  fn status_key_deleted(&self) -> &'static str {
    "Clef supprimee"
  }
  fn status_key_exported(&self) -> &'static str {
    "Clef exportee"
  }
  fn status_key_imported(&self) -> &'static str {
    "Clef importee"
  }
  fn status_published(&self) -> &'static str {
    "Publie sur le serveur de clefs"
  }
  fn status_publish_failed(&self) -> &'static str {
    "Echec de la publication"
  }
  fn status_backup_done(&self) -> &'static str {
    "Sauvegarde terminee"
  }
  fn status_preferences_saved(&self) -> &'static str {
    "Preferences enregistrees"
  }

  fn err_gpg_not_found(&self) -> &'static str {
    "gpg introuvable"
  }
  fn err_invalid_key(&self) -> &'static str {
    "Clef invalide"
  }
  fn err_import_not_pgp(&self) -> &'static str {
    "Le contenu ne contient pas de clef PGP"
  }
  fn err_export_failed(&self) -> &'static str {
    "Echec de l'export"
  }

  fn encrypt_title(&self) -> &'static str {
    "Chiffrer des fichiers"
  }
  fn encrypt_add_files(&self) -> &'static str {
    "Ajouter des fichiers"
  }
  fn encrypt_recipients(&self) -> &'static str {
    "Destinataires"
  }
  fn encrypt_no_recipients(&self) -> &'static str {
    "Aucun destinataire selectionne"
  }
  fn encrypt_trust_warning_title(&self) -> &'static str {
    "Destinataires non fiables"
  }
  fn encrypt_trust_warning_body(&self) -> &'static str {
    "Certains destinataires ont un niveau de confiance insuffisant. Chiffrer quand meme ?"
  }
  fn encrypt_format_binary(&self) -> &'static str {
    ".gpg (binaire)"
  }
  fn encrypt_format_armor(&self) -> &'static str {
    ".asc (armored)"
  }

  fn sign_title(&self) -> &'static str {
    "Signer un fichier"
  }
  fn sign_select_file(&self) -> &'static str {
    "Selectionner un fichier a signer"
  }
  fn sign_select_key(&self) -> &'static str {
    "Clef de signature"
  }

  fn verify_title(&self) -> &'static str {
    "Verifier une signature"
  }
  fn verify_select_file(&self) -> &'static str {
    "Selectionner un fichier a verifier"
  }
  fn verify_outcome_valid(&self) -> &'static str {
    "Signature valide"
  }
  fn verify_outcome_bad_sig(&self) -> &'static str {
    "Signature invalide"
  }
  fn verify_outcome_unknown_key(&self) -> &'static str {
    "Clef inconnue"
  }
  fn verify_outcome_expired_key(&self) -> &'static str {
    "Clef expiree"
  }
  fn verify_outcome_revoked_key(&self) -> &'static str {
    "Clef revoquee"
  }

  fn health_title(&self) -> &'static str {
    "Diagnostic"
  }
  fn health_ok(&self) -> &'static str {
    "OK"
  }
  fn health_warning(&self) -> &'static str {
    "Attention"
  }
  fn health_error(&self) -> &'static str {
    "Erreur"
  }
  fn health_info(&self) -> &'static str {
    "Info"
  }

  fn import_title(&self) -> &'static str {
    "Importer une clef"
  }
  fn import_tab_file(&self) -> &'static str {
    "Depuis un fichier"
  }
  fn import_tab_url(&self) -> &'static str {
    "Depuis une URL"
  }
  fn import_tab_keyserver(&self) -> &'static str {
    "Serveur de clefs"
  }
  fn import_tab_paste(&self) -> &'static str {
    "Coller"
  }

  fn keyserver_openpgp(&self) -> &'static str {
    "keys.openpgp.org"
  }
  fn keyserver_ubuntu(&self) -> &'static str {
    "keyserver.ubuntu.com"
  }
  fn keyserver_status_unknown(&self) -> &'static str {
    "Inconnu"
  }
  fn keyserver_status_published(&self) -> &'static str {
    "Publie"
  }
  fn keyserver_status_not_published(&self) -> &'static str {
    "Non publie"
  }

  fn settings_title(&self) -> &'static str {
    "Parametres"
  }
  fn settings_language(&self) -> &'static str {
    "Langue"
  }
  fn settings_language_english(&self) -> &'static str {
    "English"
  }
  fn settings_language_french(&self) -> &'static str {
    "Francais"
  }
  fn settings_scale_factor(&self) -> &'static str {
    "Echelle de l'interface"
  }
  fn settings_scale_factor_hint(&self) -> &'static str {
    "Ajuste l'echelle (utile sur ecrans HiDPI ou 1080p)"
  }
  fn settings_theme(&self) -> &'static str {
    "Theme"
  }
  fn settings_theme_catppuccin(&self) -> &'static str {
    "Catppuccin Frappe"
  }
  fn settings_theme_ussr(&self) -> &'static str {
    "USSR"
  }

  fn health_diagnostics_title(&self) -> &'static str {
    "Diagnostic GPG"
  }
  fn health_diagnostics_desc(&self) -> &'static str {
    "Etat de votre installation et de votre configuration GnuPG."
  }
  fn health_checking(&self) -> &'static str {
    "Verification en cours…"
  }

  fn status_key_copied(&self) -> &'static str {
    "Clef copiee dans le presse-papier"
  }
  fn status_link_copied(&self) -> &'static str {
    "Lien copie dans le presse-papier"
  }
  fn status_card_migrated(&self) -> &'static str {
    "Migration vers la carte terminee"
  }
  fn status_subkey_renewed(&self) -> &'static str {
    "Sous-clef renouvelee"
  }
  fn status_subkey_rotated(&self) -> &'static str {
    "Sous-clef remplacee"
  }
  fn status_file_signed(&self) -> &'static str {
    "Fichier signe"
  }
  fn status_files_encrypted(&self) -> &'static str {
    "Fichiers chiffres"
  }

  fn err_delete_failed(&self) -> &'static str {
    "Echec de la suppression"
  }
  fn err_create_failed(&self) -> &'static str {
    "Echec de la creation"
  }
  fn err_import_failed(&self) -> &'static str {
    "Echec de l'import"
  }
  fn err_subkey_renew_failed(&self) -> &'static str {
    "Echec du renouvellement"
  }
  fn err_sign_failed(&self) -> &'static str {
    "Echec de la signature"
  }
  fn err_encrypt_failed(&self) -> &'static str {
    "Echec du chiffrement"
  }
  fn err_backup_failed(&self) -> &'static str {
    "Echec de la sauvegarde"
  }
  fn err_upload_failed(&self) -> &'static str {
    "Echec de l'upload"
  }
  fn err_save_config_failed(&self) -> &'static str {
    "Echec de l'enregistrement des preferences"
  }

  fn modal_delete_title(&self) -> &'static str {
    "Supprimer la clef ?"
  }
  fn modal_delete_stub_only(&self) -> &'static str {
    "Seul le stub local sera supprime."
  }
  fn modal_delete_stub_body(&self) -> &'static str {
    "La clef physique sur la YubiKey ne sera pas affectee."
  }
  fn modal_delete_secret(&self) -> &'static str {
    "Operation irreversible : la clef privee sera detruite."
  }
  fn modal_delete_secret_body(&self) -> &'static str {
    "Sans sauvegarde, vos donnees chiffrees seront definitivement irrecuperables."
  }
  fn modal_delete_public(&self) -> &'static str {
    "La clef publique sera supprimee de votre trousseau."
  }
  fn modal_delete_public_body(&self) -> &'static str {
    "Cette operation peut etre annulee en reimportant la clef."
  }
  fn modal_migration_irreversible(&self) -> &'static str {
    "Operation irreversible : la clef privee sera deplacee sur la YubiKey."
  }
  fn modal_migration_backup_warning(&self) -> &'static str {
    "Sans sauvegarde, si la YubiKey est perdue ou detruite, les donnees chiffrees seront irrecuperables."
  }
  fn modal_migration_backup_btn(&self) -> &'static str {
    "Sauvegarder d'abord"
  }
  fn modal_migration_confirm_btn(&self) -> &'static str {
    "J'ai un backup → Continuer"
  }
  fn modal_migration_cancel_btn(&self) -> &'static str {
    "Annuler"
  }
  fn modal_delete_export_first_btn(&self) -> &'static str {
    "Exporter d'abord"
  }
  fn modal_delete_confirm_btn(&self) -> &'static str {
    "Confirmer la suppression"
  }
  fn modal_delete_cancel_btn(&self) -> &'static str {
    "Annuler"
  }

  fn keyserver_badge_published(&self) -> &'static str {
    "Publiee sur keys.openpgp.org"
  }
  fn keyserver_badge_not_published(&self) -> &'static str {
    "Pas encore publiee"
  }
  fn keyserver_badge_checking(&self) -> &'static str {
    "Verification sur keys.openpgp.org…"
  }
  fn keyserver_badge_link_btn(&self) -> &'static str {
    "Lien"
  }

  fn btn_export_public(&self) -> &'static str {
    "Exporter pub"
  }
  fn btn_backup_key(&self) -> &'static str {
    "Sauvegarder"
  }
  fn btn_migrate_yubikey(&self) -> &'static str {
    "Migrer vers YubiKey"
  }

  fn decrypt_title(&self) -> &'static str {
    "Dechiffrer des fichiers"
  }
  fn decrypt_add_files(&self) -> &'static str {
    "Ajouter des fichiers"
  }
  fn decrypt_in_progress(&self) -> &'static str {
    "Dechiffrement en cours..."
  }
  fn btn_decrypt(&self) -> &'static str {
    "Dechiffrer"
  }

  fn verify_no_file(&self) -> &'static str {
    "Aucun fichier selectionne"
  }
  fn verify_sig_auto_hint(&self) -> &'static str {
    "Optionnel — cherche automatiquement <fichier>.sig"
  }
  fn verify_signed_by(&self) -> &'static str {
    "Signe par"
  }
  fn verify_signed_on(&self) -> &'static str {
    "le"
  }
  fn verify_in_progress(&self) -> &'static str {
    "Verification en cours..."
  }
  fn verify_error_prefix(&self) -> &'static str {
    "Erreur"
  }
  fn btn_verify(&self) -> &'static str {
    "Verifier"
  }
  fn btn_sign(&self) -> &'static str {
    "Signer"
  }
  fn no_file_selected(&self) -> &'static str {
    "Aucun fichier selectionne"
  }
  fn loading(&self) -> &'static str {
    "Chargement..."
  }
  fn no_keys(&self) -> &'static str {
    "Aucune clef"
  }

  fn modal_publish_recommended(&self) -> &'static str {
    "Recommande · Respecte le RGPD."
  }
  fn modal_publish_openpgp_desc(&self) -> &'static str {
    "Un email de validation sera envoye a votre adresse email d'identite pour rendre votre cle visible dans les recherches."
  }
  fn modal_publish_ubuntu_desc(&self) -> &'static str {
    "Un serveur de clefs ouvert. Votre cle et votre identite seront visibles publiquement."
  }
  fn modal_publish_privacy(&self) -> &'static str {
    "Avis de confidentialite"
  }
  fn modal_publish_confirm_btn(&self) -> &'static str {
    "Publier"
  }
  fn modal_publish_select_keyserver(&self) -> &'static str {
    "Selectionner le serveur de clefs"
  }

  fn verify_valid_full_trust(&self) -> &'static str {
    "Signature valide"
  }
  fn verify_valid_marginal_trust(&self) -> &'static str {
    "Valide (confiance marginale)"
  }
  fn verify_valid_no_trust(&self) -> &'static str {
    "Signature correcte — identite non verifiee"
  }

  fn status_trust_updated(&self) -> &'static str {
    "Niveau de confiance mis a jour"
  }
  fn err_trust_failed(&self) -> &'static str {
    "Echec de la mise a jour de confiance"
  }

  fn err_diagnostic_failed(&self) -> &'static str {
    "Erreur diagnostic"
  }

  fn status_subkey_created(&self) -> &'static str {
    "Sous-clef creee"
  }
  fn err_subkey_add_failed(&self) -> &'static str {
    "Echec de l'ajout de sous-clef"
  }

  fn status_published_openpgp_email(&self) -> &'static str {
    "Clef publiee. Verifiez votre email pour valider la publication sur keys.openpgp.org."
  }
  fn err_republish_failed(&self) -> &'static str {
    "Echec de la republication"
  }

  fn btn_encrypt(&self) -> &'static str {
    "Chiffrer"
  }
  fn encrypt_in_progress(&self) -> &'static str {
    "Chiffrement en cours..."
  }

  fn status_files_decrypted(&self) -> &'static str {
    "Fichiers dechiffres"
  }
  fn err_decrypt_failed(&self) -> &'static str {
    "Echec du dechiffrement"
  }
  fn err_no_decryptable_file(&self) -> &'static str {
    "Aucun fichier dechiffrable selectionne."
  }

  // key_list.rs
  fn key_list_error(&self, err: &str) -> String {
    format!("Erreur : {err}")
  }
  fn key_list_header_name(&self) -> &'static str {
    "Nom / Email"
  }
  fn key_list_header_expires(&self) -> &'static str {
    "Expire"
  }
  fn key_list_header_status(&self) -> &'static str {
    "Etat"
  }
  fn key_list_select_hint(&self) -> &'static str {
    "Selectionnez une clef pour voir les details."
  }

  // key_detail.rs
  fn key_type_on_card(&self) -> &'static str {
    "Sur YubiKey"
  }
  fn key_type_public_private(&self) -> &'static str {
    "Publique + Privee"
  }
  fn key_type_public_only(&self) -> &'static str {
    "Publique"
  }
  fn subkey_type_signature(&self) -> &'static str {
    "Signature"
  }
  fn subkey_type_encryption(&self) -> &'static str {
    "Chiffrement"
  }
  fn subkey_type_ssh_auth(&self) -> &'static str {
    "Auth SSH"
  }
  fn export_menu_save_disk(&self) -> &'static str {
    "Enregistrer sur le disque"
  }
  fn export_menu_copy_clipboard(&self) -> &'static str {
    "Copier dans le presse-papier"
  }
  fn export_menu_paste_link(&self) -> &'static str {
    "Obtenir un lien public (paste.rs)"
  }
  fn subkey_expiry_1_year(&self) -> &'static str {
    "1 an"
  }
  fn subkey_expiry_2_years(&self) -> &'static str {
    "2 ans"
  }
  fn subkey_expiry_5_years(&self) -> &'static str {
    "5 ans"
  }

  // create_key.rs
  fn create_key_generating(&self) -> &'static str {
    "Generation..."
  }
  fn create_key_title(&self) -> &'static str {
    "Nouvelle clef PGP"
  }
  fn create_key_subtitle(&self) -> &'static str {
    "Genere une clef maitre et ses sous-clefs dediees."
  }
  fn create_key_section_identity(&self) -> &'static str {
    "Identite"
  }
  fn create_key_field_name(&self) -> &'static str {
    "Nom"
  }
  fn create_key_field_email(&self) -> &'static str {
    "Email"
  }
  fn create_key_section_subkeys(&self) -> &'static str {
    "Sous-clefs"
  }
  fn create_key_section_expiration(&self) -> &'static str {
    "Expiration"
  }
  fn create_key_include_ssh(&self) -> &'static str {
    "Inclure une clef d'authentification SSH"
  }
  fn create_key_about_master(&self) -> &'static str {
    "A propos de la clef maitre"
  }
  fn create_key_hint_expiry(&self) -> &'static str {
    "Les sous-clefs expirent automatiquement. Une courte duree limite les degats en cas de compromission — vous pourrez les renouveler avant echeance."
  }
  fn create_key_hint_ssh(&self) -> &'static str {
    "Permet de vous authentifier sur des serveurs distants sans mot de passe, en utilisant votre clef PGP comme clef SSH."
  }
  fn create_key_hint_master(&self) -> &'static str {
    "La clef maitre definit votre identite PGP a long terme — elle ne sert qu'a certifier vos sous-clefs. Elle n'expire jamais. Conservez-la hors ligne avec son certificat de revocation."
  }

  // encrypt.rs
  fn encrypt_tab_my_keys(&self) -> &'static str {
    "Mes clefs"
  }
  fn encrypt_tab_public_keys(&self) -> &'static str {
    "Clefs publiques"
  }
  fn encrypt_no_keys(&self) -> &'static str {
    "Aucune clef avec capacite de chiffrement."
  }
  fn encrypt_choose_files(&self) -> &'static str {
    "Choisir des fichiers..."
  }
  fn encrypt_drop_hint(&self) -> &'static str {
    "Glissez des fichiers ici"
  }
  fn encrypt_format_ascii_desc(&self) -> &'static str {
    "Texte ASCII — pour coller dans un email ou un message."
  }
  fn encrypt_format_binary_desc(&self) -> &'static str {
    "Binaire compact — pour pieces jointes et stockage."
  }
  fn encrypt_multi_recipient_hint(&self) -> &'static str {
    "Chaque destinataire peut dechiffrer le fichier independamment avec sa propre clef. \
     Pensez a vous ajouter pour conserver un acces au fichier chiffre."
  }
  fn encrypt_select_hint(&self) -> &'static str {
    "Selectionnez les destinataires et les fichiers."
  }

  // sign.rs
  fn sign_no_keys(&self) -> &'static str {
    "Aucune clef privee avec capacite de signature."
  }
  fn sign_about(&self) -> &'static str {
    "Signer un fichier cree une preuve cryptographique que vous en etes l'auteur. \
     Le fichier original n'est pas modifie — la signature est enregistree dans un fichier .sig separe."
  }

  // verify.rs
  fn verify_sig_file_placeholder(&self) -> &'static str {
    "Fichier .sig..."
  }
  fn verify_trust_warning(&self) -> &'static str {
    "L'identite affichee n'est pas verifiee par votre toile de confiance."
  }
  fn verify_fingerprint_label(&self) -> &'static str {
    "Fingerprint :"
  }
  fn verify_bad_sig_desc(&self) -> &'static str {
    "La signature ne correspond pas a ce fichier. \
     Verifiez que vous avez selectionne le bon fichier et la bonne signature."
  }
  fn verify_unknown_key_desc(&self) -> &'static str {
    "La clef publique du signataire n'est pas dans votre trousseau. \
     Importez-la pour verifier l'identite du signataire."
  }
  fn verify_expired_key_desc(&self) -> &'static str {
    "La signature est mathematiquement valide, mais la clef du signataire \
     etait expiree au moment de la verification."
  }
  fn verify_revoked_key_desc(&self) -> &'static str {
    "La clef ayant signe ce fichier a ete revoquee. \
     La signature n'est plus consideree comme fiable."
  }
  fn verify_about(&self) -> &'static str {
    "Verifier une signature confirme que le fichier n'a pas ete modifie et identifie son auteur."
  }
  fn verify_sig_auto_hint_with_name(&self, auto_name: &str) -> String {
    format!("Optionnel — cherchera automatiquement {auto_name}")
  }

  // import.rs
  fn import_source_from_file(&self) -> &'static str {
    "Depuis un fichier"
  }
  fn import_select_source(&self) -> &'static str {
    "Choisissez la source de la clef a importer."
  }
  fn import_url_hint(&self) -> &'static str {
    "Collez une URL pointant vers une clef armored (paste.rs, page web, etc.)."
  }
  fn import_url_button(&self) -> &'static str {
    "Importer depuis l'URL"
  }
  fn import_keyserver_hint(&self) -> &'static str {
    "Fingerprint complet (40 hex), ID long (16 hex) ou adresse email."
  }
  fn import_keyserver_button(&self) -> &'static str {
    "Importer depuis le keyserver"
  }
  fn import_paste_hint(&self) -> &'static str {
    "Collez directement le contenu d'une clef PGP armored (-----BEGIN PGP...)."
  }
  fn import_paste_button(&self) -> &'static str {
    "Importer la clef collee"
  }

  // health.rs
  fn health_category_installation(&self) -> &'static str {
    "Installation"
  }
  fn health_category_agent(&self) -> &'static str {
    "Agent GPG"
  }
  fn health_category_security(&self) -> &'static str {
    "Securite"
  }

  // decrypt.rs
  fn decrypt_auto_key_hint(&self) -> &'static str {
    "GPG utilisera automatiquement votre clef privee. \
     Si elle est protegee par un mot de passe, une fenetre s'ouvrira pour vous le demander."
  }
  fn decrypt_drop_hint(&self) -> &'static str {
    "Glissez des fichiers .gpg ou .asc ici, ou utilisez le bouton ci-dessous."
  }
  fn decrypt_key_available(&self) -> &'static str {
    "Clef disponible"
  }
  fn decrypt_key_missing(&self) -> &'static str {
    "Clef manquante"
  }
  fn decrypt_key_checking(&self) -> &'static str {
    "Verification..."
  }
  fn decrypt_no_key_warning(&self) -> &'static str {
    "Certains fichiers ne peuvent pas etre dechiffres — vous ne possedez pas \
     la clef privee correspondante. Ces fichiers seront ignores."
  }
  fn decrypt_about(&self) -> &'static str {
    "Dechiffrez des fichiers chiffres avec GPG."
  }

  // Expiry warning banner
  fn expiry_warning_title(&self) -> &'static str {
    "Sous-clef(s) expirant dans 90 jours"
  }
  fn expiry_warning_renew(&self) -> &'static str {
    "Renouveler"
  }

  // File dialog titles
  fn dialog_choose_files_encrypt(&self) -> &'static str {
    "Choisir des fichiers a chiffrer"
  }
  fn dialog_choose_files_decrypt(&self) -> &'static str {
    "Choisir des fichiers a dechiffrer"
  }
  fn dialog_filter_gpg_files(&self) -> &'static str {
    "Fichiers GPG"
  }
  fn dialog_choose_file_sign(&self) -> &'static str {
    "Choisir un fichier a signer"
  }
  fn dialog_choose_file_verify(&self) -> &'static str {
    "Choisir le fichier a verifier"
  }
  fn dialog_choose_sig_file(&self) -> &'static str {
    "Choisir le fichier de signature (.sig)"
  }
  fn dialog_choose_backup_folder(&self) -> &'static str {
    "Choisir un dossier de sauvegarde"
  }

  // Revocation certificate section
  fn revocation_cert_title(&self) -> &'static str {
    "Certificat de revocation"
  }
  fn revocation_cert_found(&self) -> &'static str {
    "Certificat present"
  }
  fn revocation_cert_missing(&self) -> &'static str {
    "Certificat absent"
  }
  fn revocation_cert_export(&self) -> &'static str {
    "Exporter .rev"
  }
  fn revocation_cert_generate(&self) -> &'static str {
    "Generer"
  }
  fn revocation_cert_copy_path(&self) -> &'static str {
    "Copier le chemin"
  }
  fn status_revocation_cert_generated(&self) -> &'static str {
    "Certificat de revocation genere"
  }

  // --- Chat v0.6.0 ---

  fn nav_section_chat(&self) -> &'static str {
    "CHAT"
  }
  fn nav_chat_rooms(&self) -> &'static str {
    "Salons"
  }
  fn nav_chat_rooms_ussr(&self) -> &'static str {
    "TRANSMISSIONS"
  }

  fn chat_no_rooms(&self) -> &'static str {
    "Aucune conversation."
  }
  fn chat_no_rooms_ussr(&self) -> &'static str {
    "Pas encore de camarades. Etablissez des communications securisees."
  }
  fn chat_create_room(&self) -> &'static str {
    "+ Nouveau"
  }
  fn chat_join_room(&self) -> &'static str {
    "Rejoindre"
  }

  fn chat_mqtt_connected(&self) -> &'static str {
    "Connecte"
  }
  fn chat_mqtt_connecting(&self) -> &'static str {
    "Connexion..."
  }
  fn chat_mqtt_reconnecting(&self) -> &'static str {
    "Reconnexion..."
  }
  fn chat_mqtt_disconnected(&self) -> &'static str {
    "Deconnecte"
  }
  fn chat_mqtt_failed(&self) -> &'static str {
    "Echec de connexion"
  }
  fn chat_mqtt_disconnected_banner(&self) -> &'static str {
    "Deconnecte — reconnexion en cours..."
  }

  fn chat_copy_invite(&self) -> &'static str {
    "Copier l'invitation"
  }
  fn chat_leave_room(&self) -> &'static str {
    "Quitter"
  }

  fn chat_decrypt_failed(&self) -> &'static str {
    "Impossible de dechiffrer ce message"
  }
  fn chat_type_message(&self) -> &'static str {
    "Ecrire un message..."
  }
  fn chat_select_room(&self) -> &'static str {
    "Selectionnez un salon pour discuter."
  }

  fn chat_send(&self) -> &'static str {
    "Envoyer"
  }

  fn chat_create_room_title(&self) -> &'static str {
    "Creer un salon"
  }
  fn chat_create_room_title_ussr(&self) -> &'static str {
    "ETABLIR UN CANAL SECURISE"
  }
  fn chat_room_name_label(&self) -> &'static str {
    "Nom du salon"
  }
  fn chat_room_name_placeholder(&self) -> &'static str {
    "ex. Equipe ops"
  }
  fn chat_relay_label(&self) -> &'static str {
    "Relais MQTT"
  }
  fn chat_relay_placeholder(&self) -> &'static str {
    "mqtts://host:8883"
  }
  fn chat_relay_hint(&self) -> &'static str {
    "TLS requis. Utilisez votre propre broker pour une confidentialite maximale."
  }
  fn chat_participants_label(&self) -> &'static str {
    "Participants (un fingerprint par ligne)"
  }
  fn chat_participants_hint(&self) -> &'static str {
    "Ajoutez les fingerprints PGP des participants, un par ligne."
  }
  fn chat_create_room_btn(&self) -> &'static str {
    "Creer le salon"
  }

  fn chat_join_room_title(&self) -> &'static str {
    "Rejoindre un salon"
  }
  fn chat_join_code_label(&self) -> &'static str {
    "Code d'invitation"
  }
  fn chat_join_code_placeholder(&self) -> &'static str {
    "pgpilot:join:..."
  }
  fn chat_join_code_hint(&self) -> &'static str {
    "Collez le code d'invitation que vous avez recu."
  }
  fn chat_join_btn(&self) -> &'static str {
    "Rejoindre le salon"
  }

  fn chat_choose_identity_title(&self) -> &'static str {
    "Choisissez votre identite"
  }
  fn chat_choose_identity_hint(&self) -> &'static str {
    "Vous avez plusieurs clefs privees. Selectionnez celle a utiliser dans ce salon :"
  }
  fn chat_enter_room_btn(&self) -> &'static str {
    "Entrer dans le salon"
  }
  fn chat_confirm_identity_btn(&self) -> &'static str {
    "Utiliser cette identité"
  }

  fn chat_leave_confirm_title(&self) -> &'static str {
    "Quitter le salon ?"
  }
  fn chat_leave_confirm_body_with_name(&self, name: &str) -> String {
    format!(
      "Vous ne recevrez plus les messages de \"{name}\". \
       Cette action est irreversible — vous auriez besoin d'une nouvelle invitation pour rejoindre."
    )
  }
  fn chat_leave_room_btn(&self) -> &'static str {
    "Quitter le salon"
  }

  fn status_chat_room_created(&self) -> &'static str {
    "Salon cree."
  }
  fn status_chat_room_joined(&self) -> &'static str {
    "Salon rejoint."
  }
  fn status_chat_room_left(&self) -> &'static str {
    "Vous avez quitte le salon."
  }
  fn status_chat_invite_copied(&self) -> &'static str {
    "Code d'invitation copie."
  }
  fn status_chat_message_sent(&self) -> &'static str {
    "Message envoye."
  }

  fn err_chat_room_create_failed(&self) -> &'static str {
    "Echec de creation du salon."
  }
  fn err_chat_room_join_failed(&self) -> &'static str {
    "Echec de rejoindre le salon."
  }
  fn err_chat_room_leave_failed(&self) -> &'static str {
    "Echec de quitter le salon."
  }
  fn err_chat_send_failed(&self) -> &'static str {
    "Echec d'envoi du message."
  }
  fn err_chat_invite_copy_failed(&self) -> &'static str {
    "Echec de copie du code d'invitation."
  }
}
