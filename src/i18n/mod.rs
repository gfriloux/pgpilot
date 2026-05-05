pub mod english;
pub mod french;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum Language {
  #[default]
  English,
  French,
}

// Methods are called from UI views; some appear unused to rustc because views use dynamic dispatch.
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

  // key_list.rs
  fn key_list_error(&self, err: &str) -> String;
  fn key_list_header_name(&self) -> &'static str;
  fn key_list_header_expires(&self) -> &'static str;
  fn key_list_header_status(&self) -> &'static str;
  fn key_list_select_hint(&self) -> &'static str;

  // key_detail.rs
  fn key_type_on_card(&self) -> &'static str;
  fn key_type_public_private(&self) -> &'static str;
  fn key_type_public_only(&self) -> &'static str;
  fn subkey_type_signature(&self) -> &'static str;
  fn subkey_type_encryption(&self) -> &'static str;
  fn subkey_type_ssh_auth(&self) -> &'static str;
  fn export_menu_save_disk(&self) -> &'static str;
  fn export_menu_copy_clipboard(&self) -> &'static str;
  fn export_menu_paste_link(&self) -> &'static str;
  fn subkey_expiry_1_year(&self) -> &'static str;
  fn subkey_expiry_2_years(&self) -> &'static str;
  fn subkey_expiry_5_years(&self) -> &'static str;

  // create_key.rs
  fn create_key_generating(&self) -> &'static str;
  fn create_key_title(&self) -> &'static str;
  fn create_key_subtitle(&self) -> &'static str;
  fn create_key_section_identity(&self) -> &'static str;
  fn create_key_field_name(&self) -> &'static str;
  fn create_key_field_email(&self) -> &'static str;
  fn create_key_section_subkeys(&self) -> &'static str;
  fn create_key_section_expiration(&self) -> &'static str;
  fn create_key_include_ssh(&self) -> &'static str;
  fn create_key_about_master(&self) -> &'static str;
  fn create_key_hint_expiry(&self) -> &'static str;
  fn create_key_hint_ssh(&self) -> &'static str;
  fn create_key_hint_master(&self) -> &'static str;

  // encrypt.rs
  fn encrypt_tab_my_keys(&self) -> &'static str;
  fn encrypt_tab_public_keys(&self) -> &'static str;
  fn encrypt_no_keys(&self) -> &'static str;
  fn encrypt_choose_files(&self) -> &'static str;
  fn encrypt_drop_hint(&self) -> &'static str;
  fn encrypt_format_ascii_desc(&self) -> &'static str;
  fn encrypt_format_binary_desc(&self) -> &'static str;
  fn encrypt_multi_recipient_hint(&self) -> &'static str;
  fn encrypt_select_hint(&self) -> &'static str;

  // sign.rs
  fn sign_no_keys(&self) -> &'static str;
  fn sign_about(&self) -> &'static str;

  // verify.rs
  fn verify_sig_file_placeholder(&self) -> &'static str;
  fn verify_trust_warning(&self) -> &'static str;
  fn verify_fingerprint_label(&self) -> &'static str;
  fn verify_bad_sig_desc(&self) -> &'static str;
  fn verify_unknown_key_desc(&self) -> &'static str;
  fn verify_expired_key_desc(&self) -> &'static str;
  fn verify_revoked_key_desc(&self) -> &'static str;
  fn verify_about(&self) -> &'static str;
  fn verify_sig_auto_hint_with_name(&self, auto_name: &str) -> String;

  // import.rs
  fn import_source_from_file(&self) -> &'static str;
  fn import_select_source(&self) -> &'static str;
  fn import_url_hint(&self) -> &'static str;
  fn import_url_button(&self) -> &'static str;
  fn import_keyserver_hint(&self) -> &'static str;
  fn import_keyserver_button(&self) -> &'static str;
  fn import_paste_hint(&self) -> &'static str;
  fn import_paste_button(&self) -> &'static str;

  // health.rs
  fn health_category_installation(&self) -> &'static str;
  fn health_category_agent(&self) -> &'static str;
  fn health_category_security(&self) -> &'static str;

  // decrypt.rs
  fn decrypt_auto_key_hint(&self) -> &'static str;
  fn decrypt_drop_hint(&self) -> &'static str;
  fn decrypt_key_available(&self) -> &'static str;
  fn decrypt_key_missing(&self) -> &'static str;
  fn decrypt_key_checking(&self) -> &'static str;
  fn decrypt_no_key_warning(&self) -> &'static str;
  fn decrypt_about(&self) -> &'static str;

  // File dialog titles
  fn dialog_choose_files_encrypt(&self) -> &'static str;
  fn dialog_choose_files_decrypt(&self) -> &'static str;
  fn dialog_filter_gpg_files(&self) -> &'static str;
  fn dialog_choose_file_sign(&self) -> &'static str;
  fn dialog_choose_file_verify(&self) -> &'static str;
  fn dialog_choose_sig_file(&self) -> &'static str;
  fn dialog_choose_backup_folder(&self) -> &'static str;
}

pub fn strings_for(lang: Language) -> &'static dyn Strings {
  match lang {
    Language::English => &english::EnglishStrings,
    Language::French => &french::FrenchStrings,
  }
}
