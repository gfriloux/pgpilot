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
    "Sauvegarde terminie"
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
    "Migration vers la carte terminie"
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
    "Sans sauvegarde, vos donnees chiffrees seront definitivement irrrecuperables."
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
    "Sans sauvegarde, si la YubiKey est perdue ou detruite, les donnees chiffrees seront irrrecuperables."
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
}
