pub mod english;
pub mod french;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum Language {
  #[default]
  English,
  French,
}

#[allow(dead_code)]
pub trait Strings: Send + Sync {
  // Navigation
  fn nav_my_keys(&self) -> &'static str;
  fn nav_public_keys(&self) -> &'static str;
  fn nav_import(&self) -> &'static str;
  fn nav_create_key(&self) -> &'static str;
  fn nav_encrypt(&self) -> &'static str;
  fn nav_decrypt(&self) -> &'static str;
  fn nav_sign(&self) -> &'static str;
  fn nav_verify(&self) -> &'static str;
  fn nav_health(&self) -> &'static str;
  fn nav_settings(&self) -> &'static str;
  fn sidebar_section_keys(&self) -> &'static str;
  fn sidebar_section_operations(&self) -> &'static str;
  fn sidebar_section_tools(&self) -> &'static str;

  // Boutons communs
  fn btn_ok(&self) -> &'static str;
  fn btn_cancel(&self) -> &'static str;
  fn btn_confirm(&self) -> &'static str;
  fn btn_back(&self) -> &'static str;
  fn btn_create(&self) -> &'static str;
  fn btn_delete(&self) -> &'static str;
  fn btn_export(&self) -> &'static str;
  fn btn_import(&self) -> &'static str;
  fn btn_copy(&self) -> &'static str;
  fn btn_publish(&self) -> &'static str;
  fn btn_backup(&self) -> &'static str;
  fn btn_migrate(&self) -> &'static str;
  fn btn_renew(&self) -> &'static str;
  fn btn_rotate(&self) -> &'static str;
  fn btn_add_subkey(&self) -> &'static str;

  // Clefs / détail
  fn key_fingerprint(&self) -> &'static str;
  fn key_created(&self) -> &'static str;
  fn key_expires(&self) -> &'static str;
  fn key_never_expires(&self) -> &'static str;
  fn key_trust(&self) -> &'static str;
  fn key_subkeys(&self) -> &'static str;
  fn key_no_subkeys(&self) -> &'static str;

  // Trust levels
  fn trust_undefined(&self) -> &'static str;
  fn trust_marginal(&self) -> &'static str;
  fn trust_full(&self) -> &'static str;
  fn trust_ultimate(&self) -> &'static str;

  // Status messages
  fn status_key_created(&self) -> &'static str;
  fn status_key_deleted(&self) -> &'static str;
  fn status_key_exported(&self) -> &'static str;
  fn status_key_imported(&self) -> &'static str;
  fn status_published(&self) -> &'static str;
  fn status_publish_failed(&self) -> &'static str;
  fn status_backup_done(&self) -> &'static str;
  fn status_preferences_saved(&self) -> &'static str;

  // Erreurs
  fn err_gpg_not_found(&self) -> &'static str;
  fn err_invalid_key(&self) -> &'static str;
  fn err_import_not_pgp(&self) -> &'static str;
  fn err_export_failed(&self) -> &'static str;

  // Encrypt
  fn encrypt_title(&self) -> &'static str;
  fn encrypt_add_files(&self) -> &'static str;
  fn encrypt_recipients(&self) -> &'static str;
  fn encrypt_no_recipients(&self) -> &'static str;
  fn encrypt_trust_warning_title(&self) -> &'static str;
  fn encrypt_trust_warning_body(&self) -> &'static str;
  fn encrypt_format_binary(&self) -> &'static str;
  fn encrypt_format_armor(&self) -> &'static str;

  // Sign / Verify
  fn sign_title(&self) -> &'static str;
  fn sign_select_file(&self) -> &'static str;
  fn sign_select_key(&self) -> &'static str;
  fn verify_title(&self) -> &'static str;
  fn verify_select_file(&self) -> &'static str;
  fn verify_outcome_valid(&self) -> &'static str;
  fn verify_outcome_bad_sig(&self) -> &'static str;
  fn verify_outcome_unknown_key(&self) -> &'static str;
  fn verify_outcome_expired_key(&self) -> &'static str;
  fn verify_outcome_revoked_key(&self) -> &'static str;

  // Health / Diagnostic
  fn health_title(&self) -> &'static str;
  fn health_ok(&self) -> &'static str;
  fn health_warning(&self) -> &'static str;
  fn health_error(&self) -> &'static str;
  fn health_info(&self) -> &'static str;

  // Import
  fn import_title(&self) -> &'static str;
  fn import_tab_file(&self) -> &'static str;
  fn import_tab_url(&self) -> &'static str;
  fn import_tab_keyserver(&self) -> &'static str;
  fn import_tab_paste(&self) -> &'static str;

  // Keyserver
  fn keyserver_openpgp(&self) -> &'static str;
  fn keyserver_ubuntu(&self) -> &'static str;
  fn keyserver_status_unknown(&self) -> &'static str;
  fn keyserver_status_published(&self) -> &'static str;
  fn keyserver_status_not_published(&self) -> &'static str;

  // Settings
  fn settings_title(&self) -> &'static str;
  fn settings_language(&self) -> &'static str;
  fn settings_language_english(&self) -> &'static str;
  fn settings_language_french(&self) -> &'static str;
  fn settings_scale_factor(&self) -> &'static str;
  fn settings_scale_factor_hint(&self) -> &'static str;
  fn settings_theme(&self) -> &'static str;
  fn settings_theme_catppuccin(&self) -> &'static str;
  fn settings_theme_ussr(&self) -> &'static str;

  // Health / Diagnostics titles
  fn health_diagnostics_title(&self) -> &'static str;
  fn health_diagnostics_desc(&self) -> &'static str;
  fn health_checking(&self) -> &'static str;

  // Status messages (extended)
  fn status_key_copied(&self) -> &'static str;
  fn status_link_copied(&self) -> &'static str;
  fn status_card_migrated(&self) -> &'static str;
  fn status_subkey_renewed(&self) -> &'static str;
  fn status_subkey_rotated(&self) -> &'static str;
  fn status_file_signed(&self) -> &'static str;
  fn status_files_encrypted(&self) -> &'static str;

  // Error messages (extended)
  fn err_delete_failed(&self) -> &'static str;
  fn err_create_failed(&self) -> &'static str;
  fn err_import_failed(&self) -> &'static str;
  fn err_subkey_renew_failed(&self) -> &'static str;
  fn err_sign_failed(&self) -> &'static str;
  fn err_encrypt_failed(&self) -> &'static str;
  fn err_backup_failed(&self) -> &'static str;
  fn err_upload_failed(&self) -> &'static str;
  fn err_save_config_failed(&self) -> &'static str;

  // Modal titles / bodies
  fn modal_delete_title(&self) -> &'static str;
  fn modal_delete_stub_only(&self) -> &'static str;
  fn modal_delete_stub_body(&self) -> &'static str;
  fn modal_delete_secret(&self) -> &'static str;
  fn modal_delete_secret_body(&self) -> &'static str;
  fn modal_delete_public(&self) -> &'static str;
  fn modal_delete_public_body(&self) -> &'static str;
  fn modal_migration_irreversible(&self) -> &'static str;
  fn modal_migration_backup_warning(&self) -> &'static str;
  fn modal_migration_backup_btn(&self) -> &'static str;
  fn modal_migration_confirm_btn(&self) -> &'static str;
  fn modal_migration_cancel_btn(&self) -> &'static str;
  fn modal_delete_export_first_btn(&self) -> &'static str;
  fn modal_delete_confirm_btn(&self) -> &'static str;
  fn modal_delete_cancel_btn(&self) -> &'static str;

  // Key detail labels
  fn keyserver_badge_published(&self) -> &'static str;
  fn keyserver_badge_not_published(&self) -> &'static str;
  fn keyserver_badge_checking(&self) -> &'static str;
  fn keyserver_badge_link_btn(&self) -> &'static str;

  // Action button labels
  fn btn_export_public(&self) -> &'static str;
  fn btn_backup_key(&self) -> &'static str;
  fn btn_migrate_yubikey(&self) -> &'static str;

  // Decrypt
  fn decrypt_title(&self) -> &'static str;
  fn decrypt_add_files(&self) -> &'static str;
  fn decrypt_in_progress(&self) -> &'static str;
  fn btn_decrypt(&self) -> &'static str;

  // Verify / Sign extended
  fn verify_no_file(&self) -> &'static str;
  fn verify_sig_auto_hint(&self) -> &'static str;
  fn verify_signed_by(&self) -> &'static str;
  fn verify_signed_on(&self) -> &'static str;
  fn verify_in_progress(&self) -> &'static str;
  fn verify_error_prefix(&self) -> &'static str;
  fn btn_verify(&self) -> &'static str;
  fn btn_sign(&self) -> &'static str;
  fn no_file_selected(&self) -> &'static str;
  fn loading(&self) -> &'static str;
  fn no_keys(&self) -> &'static str;

  // Publish modal
  fn modal_publish_recommended(&self) -> &'static str;
  fn modal_publish_openpgp_desc(&self) -> &'static str;
  fn modal_publish_ubuntu_desc(&self) -> &'static str;
  fn modal_publish_privacy(&self) -> &'static str;
  fn modal_publish_confirm_btn(&self) -> &'static str;
  fn modal_publish_select_keyserver(&self) -> &'static str;

  // Valid signature trust levels
  fn verify_valid_full_trust(&self) -> &'static str;
  fn verify_valid_marginal_trust(&self) -> &'static str;
  fn verify_valid_no_trust(&self) -> &'static str;

  // Trust update
  fn status_trust_updated(&self) -> &'static str;
  fn err_trust_failed(&self) -> &'static str;

  // Health / diagnostic error
  fn err_diagnostic_failed(&self) -> &'static str;

  // Subkey created
  fn status_subkey_created(&self) -> &'static str;
  fn err_subkey_add_failed(&self) -> &'static str;

  // Publish extended
  fn status_published_openpgp_email(&self) -> &'static str;
  fn err_republish_failed(&self) -> &'static str;

  // Encrypt button
  fn btn_encrypt(&self) -> &'static str;
  fn encrypt_in_progress(&self) -> &'static str;

  // Decrypt status
  fn status_files_decrypted(&self) -> &'static str;
  fn err_decrypt_failed(&self) -> &'static str;
  fn err_no_decryptable_file(&self) -> &'static str;
}

pub fn strings_for(lang: Language) -> &'static dyn Strings {
  match lang {
    Language::English => &english::EnglishStrings,
    Language::French => &french::FrenchStrings,
  }
}
