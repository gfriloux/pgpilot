// Integration tests for the i18n layer.
// The crate exposes `pub mod i18n` via src/lib.rs, so these tests can
// reference all public i18n items directly.

use pgpilot::i18n::{strings_for, Language};

fn en() -> &'static dyn pgpilot::i18n::Strings {
  strings_for(Language::English)
}

fn fr() -> &'static dyn pgpilot::i18n::Strings {
  strings_for(Language::French)
}

// ---------------------------------------------------------------------------
// Test 1 — all English strings are non-empty
// ---------------------------------------------------------------------------
#[test]
fn english_strings_all_non_empty() {
  let s = en();

  // Navigation
  assert!(!s.nav_my_keys().is_empty(), "nav_my_keys");
  assert!(!s.nav_public_keys().is_empty(), "nav_public_keys");
  assert!(!s.nav_import().is_empty(), "nav_import");
  assert!(!s.nav_create_key().is_empty(), "nav_create_key");
  assert!(!s.nav_encrypt().is_empty(), "nav_encrypt");
  assert!(!s.nav_decrypt().is_empty(), "nav_decrypt");
  assert!(!s.nav_sign().is_empty(), "nav_sign");
  assert!(!s.nav_verify().is_empty(), "nav_verify");
  assert!(!s.nav_health().is_empty(), "nav_health");
  assert!(!s.nav_settings().is_empty(), "nav_settings");
  assert!(!s.sidebar_section_keys().is_empty(), "sidebar_section_keys");
  assert!(
    !s.sidebar_section_operations().is_empty(),
    "sidebar_section_operations"
  );
  assert!(
    !s.sidebar_section_tools().is_empty(),
    "sidebar_section_tools"
  );

  // Common buttons
  assert!(!s.btn_ok().is_empty(), "btn_ok");
  assert!(!s.btn_cancel().is_empty(), "btn_cancel");
  assert!(!s.btn_confirm().is_empty(), "btn_confirm");
  assert!(!s.btn_back().is_empty(), "btn_back");
  assert!(!s.btn_create().is_empty(), "btn_create");
  assert!(!s.btn_delete().is_empty(), "btn_delete");
  assert!(!s.btn_export().is_empty(), "btn_export");
  assert!(!s.btn_import().is_empty(), "btn_import");
  assert!(!s.btn_copy().is_empty(), "btn_copy");
  assert!(!s.btn_publish().is_empty(), "btn_publish");
  assert!(!s.btn_backup().is_empty(), "btn_backup");
  assert!(!s.btn_migrate().is_empty(), "btn_migrate");
  assert!(!s.btn_renew().is_empty(), "btn_renew");
  assert!(!s.btn_rotate().is_empty(), "btn_rotate");
  assert!(!s.btn_add_subkey().is_empty(), "btn_add_subkey");
  assert!(!s.btn_export_public().is_empty(), "btn_export_public");
  assert!(!s.btn_backup_key().is_empty(), "btn_backup_key");
  assert!(!s.btn_migrate_yubikey().is_empty(), "btn_migrate_yubikey");
  assert!(!s.btn_decrypt().is_empty(), "btn_decrypt");
  assert!(!s.btn_verify().is_empty(), "btn_verify");
  assert!(!s.btn_sign().is_empty(), "btn_sign");
  assert!(!s.btn_encrypt().is_empty(), "btn_encrypt");

  // Key / detail labels
  assert!(!s.key_fingerprint().is_empty(), "key_fingerprint");
  assert!(!s.key_created().is_empty(), "key_created");
  assert!(!s.key_expires().is_empty(), "key_expires");
  assert!(!s.key_never_expires().is_empty(), "key_never_expires");
  assert!(!s.key_trust().is_empty(), "key_trust");
  assert!(!s.key_subkeys().is_empty(), "key_subkeys");
  assert!(!s.key_no_subkeys().is_empty(), "key_no_subkeys");

  // Trust levels
  assert!(!s.trust_undefined().is_empty(), "trust_undefined");
  assert!(!s.trust_marginal().is_empty(), "trust_marginal");
  assert!(!s.trust_full().is_empty(), "trust_full");
  assert!(!s.trust_ultimate().is_empty(), "trust_ultimate");

  // Status messages
  assert!(!s.status_key_created().is_empty(), "status_key_created");
  assert!(!s.status_key_deleted().is_empty(), "status_key_deleted");
  assert!(!s.status_key_exported().is_empty(), "status_key_exported");
  assert!(!s.status_key_imported().is_empty(), "status_key_imported");
  assert!(!s.status_published().is_empty(), "status_published");
  assert!(
    !s.status_publish_failed().is_empty(),
    "status_publish_failed"
  );
  assert!(!s.status_backup_done().is_empty(), "status_backup_done");
  assert!(
    !s.status_preferences_saved().is_empty(),
    "status_preferences_saved"
  );
  assert!(!s.status_key_copied().is_empty(), "status_key_copied");
  assert!(!s.status_link_copied().is_empty(), "status_link_copied");
  assert!(!s.status_card_migrated().is_empty(), "status_card_migrated");
  assert!(
    !s.status_subkey_renewed().is_empty(),
    "status_subkey_renewed"
  );
  assert!(
    !s.status_subkey_rotated().is_empty(),
    "status_subkey_rotated"
  );
  assert!(!s.status_file_signed().is_empty(), "status_file_signed");
  assert!(
    !s.status_files_encrypted().is_empty(),
    "status_files_encrypted"
  );
  assert!(!s.status_trust_updated().is_empty(), "status_trust_updated");
  assert!(
    !s.status_subkey_created().is_empty(),
    "status_subkey_created"
  );
  assert!(
    !s.status_published_openpgp_email().is_empty(),
    "status_published_openpgp_email"
  );
  assert!(
    !s.status_files_decrypted().is_empty(),
    "status_files_decrypted"
  );

  // Error messages
  assert!(!s.err_gpg_not_found().is_empty(), "err_gpg_not_found");
  assert!(!s.err_invalid_key().is_empty(), "err_invalid_key");
  assert!(!s.err_import_not_pgp().is_empty(), "err_import_not_pgp");
  assert!(!s.err_export_failed().is_empty(), "err_export_failed");
  assert!(!s.err_delete_failed().is_empty(), "err_delete_failed");
  assert!(!s.err_create_failed().is_empty(), "err_create_failed");
  assert!(!s.err_import_failed().is_empty(), "err_import_failed");
  assert!(
    !s.err_subkey_renew_failed().is_empty(),
    "err_subkey_renew_failed"
  );
  assert!(!s.err_sign_failed().is_empty(), "err_sign_failed");
  assert!(!s.err_encrypt_failed().is_empty(), "err_encrypt_failed");
  assert!(!s.err_backup_failed().is_empty(), "err_backup_failed");
  assert!(!s.err_upload_failed().is_empty(), "err_upload_failed");
  assert!(
    !s.err_save_config_failed().is_empty(),
    "err_save_config_failed"
  );
  assert!(!s.err_trust_failed().is_empty(), "err_trust_failed");
  assert!(
    !s.err_diagnostic_failed().is_empty(),
    "err_diagnostic_failed"
  );
  assert!(
    !s.err_subkey_add_failed().is_empty(),
    "err_subkey_add_failed"
  );
  assert!(!s.err_republish_failed().is_empty(), "err_republish_failed");
  assert!(!s.err_decrypt_failed().is_empty(), "err_decrypt_failed");
  assert!(
    !s.err_no_decryptable_file().is_empty(),
    "err_no_decryptable_file"
  );

  // Encrypt
  assert!(!s.encrypt_title().is_empty(), "encrypt_title");
  assert!(!s.encrypt_add_files().is_empty(), "encrypt_add_files");
  assert!(!s.encrypt_recipients().is_empty(), "encrypt_recipients");
  assert!(
    !s.encrypt_no_recipients().is_empty(),
    "encrypt_no_recipients"
  );
  assert!(
    !s.encrypt_trust_warning_title().is_empty(),
    "encrypt_trust_warning_title"
  );
  assert!(
    !s.encrypt_trust_warning_body().is_empty(),
    "encrypt_trust_warning_body"
  );
  assert!(
    !s.encrypt_format_binary().is_empty(),
    "encrypt_format_binary"
  );
  assert!(!s.encrypt_format_armor().is_empty(), "encrypt_format_armor");
  assert!(!s.encrypt_tab_my_keys().is_empty(), "encrypt_tab_my_keys");
  assert!(
    !s.encrypt_tab_public_keys().is_empty(),
    "encrypt_tab_public_keys"
  );
  assert!(!s.encrypt_no_keys().is_empty(), "encrypt_no_keys");
  assert!(!s.encrypt_choose_files().is_empty(), "encrypt_choose_files");
  assert!(!s.encrypt_drop_hint().is_empty(), "encrypt_drop_hint");
  assert!(
    !s.encrypt_format_ascii_desc().is_empty(),
    "encrypt_format_ascii_desc"
  );
  assert!(
    !s.encrypt_format_binary_desc().is_empty(),
    "encrypt_format_binary_desc"
  );
  assert!(
    !s.encrypt_multi_recipient_hint().is_empty(),
    "encrypt_multi_recipient_hint"
  );
  assert!(!s.encrypt_select_hint().is_empty(), "encrypt_select_hint");
  assert!(!s.encrypt_in_progress().is_empty(), "encrypt_in_progress");

  // Sign / Verify
  assert!(!s.sign_title().is_empty(), "sign_title");
  assert!(!s.sign_select_file().is_empty(), "sign_select_file");
  assert!(!s.sign_select_key().is_empty(), "sign_select_key");
  assert!(!s.sign_no_keys().is_empty(), "sign_no_keys");
  assert!(!s.sign_about().is_empty(), "sign_about");
  assert!(!s.verify_title().is_empty(), "verify_title");
  assert!(!s.verify_select_file().is_empty(), "verify_select_file");
  assert!(!s.verify_outcome_valid().is_empty(), "verify_outcome_valid");
  assert!(
    !s.verify_outcome_bad_sig().is_empty(),
    "verify_outcome_bad_sig"
  );
  assert!(
    !s.verify_outcome_unknown_key().is_empty(),
    "verify_outcome_unknown_key"
  );
  assert!(
    !s.verify_outcome_expired_key().is_empty(),
    "verify_outcome_expired_key"
  );
  assert!(
    !s.verify_outcome_revoked_key().is_empty(),
    "verify_outcome_revoked_key"
  );
  assert!(!s.verify_no_file().is_empty(), "verify_no_file");
  assert!(!s.verify_sig_auto_hint().is_empty(), "verify_sig_auto_hint");
  assert!(!s.verify_signed_by().is_empty(), "verify_signed_by");
  assert!(!s.verify_signed_on().is_empty(), "verify_signed_on");
  assert!(!s.verify_in_progress().is_empty(), "verify_in_progress");
  assert!(!s.verify_error_prefix().is_empty(), "verify_error_prefix");
  assert!(
    !s.verify_valid_full_trust().is_empty(),
    "verify_valid_full_trust"
  );
  assert!(
    !s.verify_valid_marginal_trust().is_empty(),
    "verify_valid_marginal_trust"
  );
  assert!(
    !s.verify_valid_no_trust().is_empty(),
    "verify_valid_no_trust"
  );
  assert!(
    !s.verify_sig_file_placeholder().is_empty(),
    "verify_sig_file_placeholder"
  );
  assert!(!s.verify_trust_warning().is_empty(), "verify_trust_warning");
  assert!(
    !s.verify_fingerprint_label().is_empty(),
    "verify_fingerprint_label"
  );
  assert!(!s.verify_bad_sig_desc().is_empty(), "verify_bad_sig_desc");
  assert!(
    !s.verify_unknown_key_desc().is_empty(),
    "verify_unknown_key_desc"
  );
  assert!(
    !s.verify_expired_key_desc().is_empty(),
    "verify_expired_key_desc"
  );
  assert!(
    !s.verify_revoked_key_desc().is_empty(),
    "verify_revoked_key_desc"
  );
  assert!(!s.verify_about().is_empty(), "verify_about");
  assert!(
    !s.verify_sig_auto_hint_with_name("test.sig").is_empty(),
    "verify_sig_auto_hint_with_name"
  );

  // Health / Diagnostic
  assert!(!s.health_title().is_empty(), "health_title");
  assert!(!s.health_ok().is_empty(), "health_ok");
  assert!(!s.health_warning().is_empty(), "health_warning");
  assert!(!s.health_error().is_empty(), "health_error");
  assert!(!s.health_info().is_empty(), "health_info");
  assert!(
    !s.health_diagnostics_title().is_empty(),
    "health_diagnostics_title"
  );
  assert!(
    !s.health_diagnostics_desc().is_empty(),
    "health_diagnostics_desc"
  );
  assert!(!s.health_checking().is_empty(), "health_checking");
  assert!(
    !s.health_category_installation().is_empty(),
    "health_category_installation"
  );
  assert!(
    !s.health_category_agent().is_empty(),
    "health_category_agent"
  );
  assert!(
    !s.health_category_security().is_empty(),
    "health_category_security"
  );

  // Import
  assert!(!s.import_title().is_empty(), "import_title");
  assert!(!s.import_tab_file().is_empty(), "import_tab_file");
  assert!(!s.import_tab_url().is_empty(), "import_tab_url");
  assert!(!s.import_tab_keyserver().is_empty(), "import_tab_keyserver");
  assert!(!s.import_tab_paste().is_empty(), "import_tab_paste");
  assert!(
    !s.import_source_from_file().is_empty(),
    "import_source_from_file"
  );
  assert!(!s.import_select_source().is_empty(), "import_select_source");
  assert!(!s.import_url_hint().is_empty(), "import_url_hint");
  assert!(!s.import_url_button().is_empty(), "import_url_button");
  assert!(
    !s.import_keyserver_hint().is_empty(),
    "import_keyserver_hint"
  );
  assert!(
    !s.import_keyserver_button().is_empty(),
    "import_keyserver_button"
  );
  assert!(!s.import_paste_hint().is_empty(), "import_paste_hint");
  assert!(!s.import_paste_button().is_empty(), "import_paste_button");

  // Keyserver
  assert!(!s.keyserver_openpgp().is_empty(), "keyserver_openpgp");
  assert!(!s.keyserver_ubuntu().is_empty(), "keyserver_ubuntu");
  assert!(
    !s.keyserver_status_unknown().is_empty(),
    "keyserver_status_unknown"
  );
  assert!(
    !s.keyserver_status_published().is_empty(),
    "keyserver_status_published"
  );
  assert!(
    !s.keyserver_status_not_published().is_empty(),
    "keyserver_status_not_published"
  );
  assert!(
    !s.keyserver_badge_published().is_empty(),
    "keyserver_badge_published"
  );
  assert!(
    !s.keyserver_badge_not_published().is_empty(),
    "keyserver_badge_not_published"
  );
  assert!(
    !s.keyserver_badge_checking().is_empty(),
    "keyserver_badge_checking"
  );
  assert!(
    !s.keyserver_badge_link_btn().is_empty(),
    "keyserver_badge_link_btn"
  );

  // Settings
  assert!(!s.settings_title().is_empty(), "settings_title");
  assert!(!s.settings_language().is_empty(), "settings_language");
  assert!(
    !s.settings_language_english().is_empty(),
    "settings_language_english"
  );
  assert!(
    !s.settings_language_french().is_empty(),
    "settings_language_french"
  );
  assert!(
    !s.settings_scale_factor().is_empty(),
    "settings_scale_factor"
  );
  assert!(
    !s.settings_scale_factor_hint().is_empty(),
    "settings_scale_factor_hint"
  );
  assert!(!s.settings_theme().is_empty(), "settings_theme");
  assert!(
    !s.settings_theme_catppuccin().is_empty(),
    "settings_theme_catppuccin"
  );
  assert!(!s.settings_theme_ussr().is_empty(), "settings_theme_ussr");

  // Modals
  assert!(!s.modal_delete_title().is_empty(), "modal_delete_title");
  assert!(
    !s.modal_delete_stub_only().is_empty(),
    "modal_delete_stub_only"
  );
  assert!(
    !s.modal_delete_stub_body().is_empty(),
    "modal_delete_stub_body"
  );
  assert!(!s.modal_delete_secret().is_empty(), "modal_delete_secret");
  assert!(
    !s.modal_delete_secret_body().is_empty(),
    "modal_delete_secret_body"
  );
  assert!(!s.modal_delete_public().is_empty(), "modal_delete_public");
  assert!(
    !s.modal_delete_public_body().is_empty(),
    "modal_delete_public_body"
  );
  assert!(
    !s.modal_migration_irreversible().is_empty(),
    "modal_migration_irreversible"
  );
  assert!(
    !s.modal_migration_backup_warning().is_empty(),
    "modal_migration_backup_warning"
  );
  assert!(
    !s.modal_migration_backup_btn().is_empty(),
    "modal_migration_backup_btn"
  );
  assert!(
    !s.modal_migration_confirm_btn().is_empty(),
    "modal_migration_confirm_btn"
  );
  assert!(
    !s.modal_migration_cancel_btn().is_empty(),
    "modal_migration_cancel_btn"
  );
  assert!(
    !s.modal_delete_export_first_btn().is_empty(),
    "modal_delete_export_first_btn"
  );
  assert!(
    !s.modal_delete_confirm_btn().is_empty(),
    "modal_delete_confirm_btn"
  );
  assert!(
    !s.modal_delete_cancel_btn().is_empty(),
    "modal_delete_cancel_btn"
  );
  assert!(
    !s.modal_publish_recommended().is_empty(),
    "modal_publish_recommended"
  );
  assert!(
    !s.modal_publish_openpgp_desc().is_empty(),
    "modal_publish_openpgp_desc"
  );
  assert!(
    !s.modal_publish_ubuntu_desc().is_empty(),
    "modal_publish_ubuntu_desc"
  );
  assert!(
    !s.modal_publish_privacy().is_empty(),
    "modal_publish_privacy"
  );
  assert!(
    !s.modal_publish_confirm_btn().is_empty(),
    "modal_publish_confirm_btn"
  );
  assert!(
    !s.modal_publish_select_keyserver().is_empty(),
    "modal_publish_select_keyserver"
  );

  // Key list
  assert!(!s.key_list_error("test").is_empty(), "key_list_error");
  assert!(!s.key_list_header_name().is_empty(), "key_list_header_name");
  assert!(
    !s.key_list_header_expires().is_empty(),
    "key_list_header_expires"
  );
  assert!(
    !s.key_list_header_status().is_empty(),
    "key_list_header_status"
  );
  assert!(!s.key_list_select_hint().is_empty(), "key_list_select_hint");

  // Key detail
  assert!(!s.key_type_on_card().is_empty(), "key_type_on_card");
  assert!(
    !s.key_type_public_private().is_empty(),
    "key_type_public_private"
  );
  assert!(!s.key_type_public_only().is_empty(), "key_type_public_only");
  assert!(
    !s.subkey_type_signature().is_empty(),
    "subkey_type_signature"
  );
  assert!(
    !s.subkey_type_encryption().is_empty(),
    "subkey_type_encryption"
  );
  assert!(!s.subkey_type_ssh_auth().is_empty(), "subkey_type_ssh_auth");
  assert!(
    !s.export_menu_save_disk().is_empty(),
    "export_menu_save_disk"
  );
  assert!(
    !s.export_menu_copy_clipboard().is_empty(),
    "export_menu_copy_clipboard"
  );
  assert!(
    !s.export_menu_paste_link().is_empty(),
    "export_menu_paste_link"
  );
  assert!(!s.subkey_expiry_1_year().is_empty(), "subkey_expiry_1_year");
  assert!(
    !s.subkey_expiry_2_years().is_empty(),
    "subkey_expiry_2_years"
  );
  assert!(
    !s.subkey_expiry_5_years().is_empty(),
    "subkey_expiry_5_years"
  );

  // Create key
  assert!(
    !s.create_key_generating().is_empty(),
    "create_key_generating"
  );
  assert!(!s.create_key_title().is_empty(), "create_key_title");
  assert!(!s.create_key_subtitle().is_empty(), "create_key_subtitle");
  assert!(
    !s.create_key_section_identity().is_empty(),
    "create_key_section_identity"
  );
  assert!(
    !s.create_key_field_name().is_empty(),
    "create_key_field_name"
  );
  assert!(
    !s.create_key_field_email().is_empty(),
    "create_key_field_email"
  );
  assert!(
    !s.create_key_section_subkeys().is_empty(),
    "create_key_section_subkeys"
  );
  assert!(
    !s.create_key_section_expiration().is_empty(),
    "create_key_section_expiration"
  );
  assert!(
    !s.create_key_include_ssh().is_empty(),
    "create_key_include_ssh"
  );
  assert!(
    !s.create_key_about_master().is_empty(),
    "create_key_about_master"
  );
  assert!(
    !s.create_key_hint_expiry().is_empty(),
    "create_key_hint_expiry"
  );
  assert!(!s.create_key_hint_ssh().is_empty(), "create_key_hint_ssh");
  assert!(
    !s.create_key_hint_master().is_empty(),
    "create_key_hint_master"
  );

  // File dialogs
  assert!(
    !s.dialog_choose_files_encrypt().is_empty(),
    "dialog_choose_files_encrypt"
  );
  assert!(
    !s.dialog_choose_files_decrypt().is_empty(),
    "dialog_choose_files_decrypt"
  );
  assert!(
    !s.dialog_filter_gpg_files().is_empty(),
    "dialog_filter_gpg_files"
  );
  assert!(
    !s.dialog_choose_file_sign().is_empty(),
    "dialog_choose_file_sign"
  );
  assert!(
    !s.dialog_choose_file_verify().is_empty(),
    "dialog_choose_file_verify"
  );
  assert!(
    !s.dialog_choose_sig_file().is_empty(),
    "dialog_choose_sig_file"
  );
  assert!(
    !s.dialog_choose_backup_folder().is_empty(),
    "dialog_choose_backup_folder"
  );

  // Misc
  assert!(!s.no_file_selected().is_empty(), "no_file_selected");
  assert!(!s.loading().is_empty(), "loading");
  assert!(!s.no_keys().is_empty(), "no_keys");

  // Decrypt
  assert!(!s.decrypt_title().is_empty(), "decrypt_title");
  assert!(!s.decrypt_add_files().is_empty(), "decrypt_add_files");
  assert!(!s.decrypt_in_progress().is_empty(), "decrypt_in_progress");
  assert!(
    !s.decrypt_auto_key_hint().is_empty(),
    "decrypt_auto_key_hint"
  );
  assert!(!s.decrypt_drop_hint().is_empty(), "decrypt_drop_hint");
  assert!(
    !s.decrypt_key_available().is_empty(),
    "decrypt_key_available"
  );
  assert!(!s.decrypt_key_missing().is_empty(), "decrypt_key_missing");
  assert!(!s.decrypt_key_checking().is_empty(), "decrypt_key_checking");
  assert!(
    !s.decrypt_no_key_warning().is_empty(),
    "decrypt_no_key_warning"
  );
  assert!(!s.decrypt_about().is_empty(), "decrypt_about");
}

// ---------------------------------------------------------------------------
// Test 2 — all French strings are non-empty
// ---------------------------------------------------------------------------
#[test]
fn french_strings_all_non_empty() {
  let s = fr();

  // Navigation
  assert!(!s.nav_my_keys().is_empty(), "nav_my_keys");
  assert!(!s.nav_public_keys().is_empty(), "nav_public_keys");
  assert!(!s.nav_import().is_empty(), "nav_import");
  assert!(!s.nav_create_key().is_empty(), "nav_create_key");
  assert!(!s.nav_encrypt().is_empty(), "nav_encrypt");
  assert!(!s.nav_decrypt().is_empty(), "nav_decrypt");
  assert!(!s.nav_sign().is_empty(), "nav_sign");
  assert!(!s.nav_verify().is_empty(), "nav_verify");
  assert!(!s.nav_health().is_empty(), "nav_health");
  assert!(!s.nav_settings().is_empty(), "nav_settings");
  assert!(!s.sidebar_section_keys().is_empty(), "sidebar_section_keys");
  assert!(
    !s.sidebar_section_operations().is_empty(),
    "sidebar_section_operations"
  );
  assert!(
    !s.sidebar_section_tools().is_empty(),
    "sidebar_section_tools"
  );

  // Common buttons
  assert!(!s.btn_ok().is_empty(), "btn_ok");
  assert!(!s.btn_cancel().is_empty(), "btn_cancel");
  assert!(!s.btn_confirm().is_empty(), "btn_confirm");
  assert!(!s.btn_back().is_empty(), "btn_back");
  assert!(!s.btn_create().is_empty(), "btn_create");
  assert!(!s.btn_delete().is_empty(), "btn_delete");
  assert!(!s.btn_export().is_empty(), "btn_export");
  assert!(!s.btn_import().is_empty(), "btn_import");
  assert!(!s.btn_copy().is_empty(), "btn_copy");
  assert!(!s.btn_publish().is_empty(), "btn_publish");
  assert!(!s.btn_backup().is_empty(), "btn_backup");
  assert!(!s.btn_migrate().is_empty(), "btn_migrate");
  assert!(!s.btn_renew().is_empty(), "btn_renew");
  assert!(!s.btn_rotate().is_empty(), "btn_rotate");
  assert!(!s.btn_add_subkey().is_empty(), "btn_add_subkey");
  assert!(!s.btn_export_public().is_empty(), "btn_export_public");
  assert!(!s.btn_backup_key().is_empty(), "btn_backup_key");
  assert!(!s.btn_migrate_yubikey().is_empty(), "btn_migrate_yubikey");
  assert!(!s.btn_decrypt().is_empty(), "btn_decrypt");
  assert!(!s.btn_verify().is_empty(), "btn_verify");
  assert!(!s.btn_sign().is_empty(), "btn_sign");
  assert!(!s.btn_encrypt().is_empty(), "btn_encrypt");

  // Key / detail labels
  assert!(!s.key_fingerprint().is_empty(), "key_fingerprint");
  assert!(!s.key_created().is_empty(), "key_created");
  assert!(!s.key_expires().is_empty(), "key_expires");
  assert!(!s.key_never_expires().is_empty(), "key_never_expires");
  assert!(!s.key_trust().is_empty(), "key_trust");
  assert!(!s.key_subkeys().is_empty(), "key_subkeys");
  assert!(!s.key_no_subkeys().is_empty(), "key_no_subkeys");

  // Trust levels
  assert!(!s.trust_undefined().is_empty(), "trust_undefined");
  assert!(!s.trust_marginal().is_empty(), "trust_marginal");
  assert!(!s.trust_full().is_empty(), "trust_full");
  assert!(!s.trust_ultimate().is_empty(), "trust_ultimate");

  // Status messages
  assert!(!s.status_key_created().is_empty(), "status_key_created");
  assert!(!s.status_key_deleted().is_empty(), "status_key_deleted");
  assert!(!s.status_key_exported().is_empty(), "status_key_exported");
  assert!(!s.status_key_imported().is_empty(), "status_key_imported");
  assert!(!s.status_published().is_empty(), "status_published");
  assert!(
    !s.status_publish_failed().is_empty(),
    "status_publish_failed"
  );
  assert!(!s.status_backup_done().is_empty(), "status_backup_done");
  assert!(
    !s.status_preferences_saved().is_empty(),
    "status_preferences_saved"
  );
  assert!(!s.status_key_copied().is_empty(), "status_key_copied");
  assert!(!s.status_link_copied().is_empty(), "status_link_copied");
  assert!(!s.status_card_migrated().is_empty(), "status_card_migrated");
  assert!(
    !s.status_subkey_renewed().is_empty(),
    "status_subkey_renewed"
  );
  assert!(
    !s.status_subkey_rotated().is_empty(),
    "status_subkey_rotated"
  );
  assert!(!s.status_file_signed().is_empty(), "status_file_signed");
  assert!(
    !s.status_files_encrypted().is_empty(),
    "status_files_encrypted"
  );
  assert!(!s.status_trust_updated().is_empty(), "status_trust_updated");
  assert!(
    !s.status_subkey_created().is_empty(),
    "status_subkey_created"
  );
  assert!(
    !s.status_published_openpgp_email().is_empty(),
    "status_published_openpgp_email"
  );
  assert!(
    !s.status_files_decrypted().is_empty(),
    "status_files_decrypted"
  );

  // Error messages
  assert!(!s.err_gpg_not_found().is_empty(), "err_gpg_not_found");
  assert!(!s.err_invalid_key().is_empty(), "err_invalid_key");
  assert!(!s.err_import_not_pgp().is_empty(), "err_import_not_pgp");
  assert!(!s.err_export_failed().is_empty(), "err_export_failed");
  assert!(!s.err_delete_failed().is_empty(), "err_delete_failed");
  assert!(!s.err_create_failed().is_empty(), "err_create_failed");
  assert!(!s.err_import_failed().is_empty(), "err_import_failed");
  assert!(
    !s.err_subkey_renew_failed().is_empty(),
    "err_subkey_renew_failed"
  );
  assert!(!s.err_sign_failed().is_empty(), "err_sign_failed");
  assert!(!s.err_encrypt_failed().is_empty(), "err_encrypt_failed");
  assert!(!s.err_backup_failed().is_empty(), "err_backup_failed");
  assert!(!s.err_upload_failed().is_empty(), "err_upload_failed");
  assert!(
    !s.err_save_config_failed().is_empty(),
    "err_save_config_failed"
  );
  assert!(!s.err_trust_failed().is_empty(), "err_trust_failed");
  assert!(
    !s.err_diagnostic_failed().is_empty(),
    "err_diagnostic_failed"
  );
  assert!(
    !s.err_subkey_add_failed().is_empty(),
    "err_subkey_add_failed"
  );
  assert!(!s.err_republish_failed().is_empty(), "err_republish_failed");
  assert!(!s.err_decrypt_failed().is_empty(), "err_decrypt_failed");
  assert!(
    !s.err_no_decryptable_file().is_empty(),
    "err_no_decryptable_file"
  );

  // Encrypt
  assert!(!s.encrypt_title().is_empty(), "encrypt_title");
  assert!(!s.encrypt_add_files().is_empty(), "encrypt_add_files");
  assert!(!s.encrypt_recipients().is_empty(), "encrypt_recipients");
  assert!(
    !s.encrypt_no_recipients().is_empty(),
    "encrypt_no_recipients"
  );
  assert!(
    !s.encrypt_trust_warning_title().is_empty(),
    "encrypt_trust_warning_title"
  );
  assert!(
    !s.encrypt_trust_warning_body().is_empty(),
    "encrypt_trust_warning_body"
  );
  assert!(
    !s.encrypt_format_binary().is_empty(),
    "encrypt_format_binary"
  );
  assert!(!s.encrypt_format_armor().is_empty(), "encrypt_format_armor");
  assert!(!s.encrypt_tab_my_keys().is_empty(), "encrypt_tab_my_keys");
  assert!(
    !s.encrypt_tab_public_keys().is_empty(),
    "encrypt_tab_public_keys"
  );
  assert!(!s.encrypt_no_keys().is_empty(), "encrypt_no_keys");
  assert!(!s.encrypt_choose_files().is_empty(), "encrypt_choose_files");
  assert!(!s.encrypt_drop_hint().is_empty(), "encrypt_drop_hint");
  assert!(
    !s.encrypt_format_ascii_desc().is_empty(),
    "encrypt_format_ascii_desc"
  );
  assert!(
    !s.encrypt_format_binary_desc().is_empty(),
    "encrypt_format_binary_desc"
  );
  assert!(
    !s.encrypt_multi_recipient_hint().is_empty(),
    "encrypt_multi_recipient_hint"
  );
  assert!(!s.encrypt_select_hint().is_empty(), "encrypt_select_hint");
  assert!(!s.encrypt_in_progress().is_empty(), "encrypt_in_progress");

  // Sign / Verify
  assert!(!s.sign_title().is_empty(), "sign_title");
  assert!(!s.sign_select_file().is_empty(), "sign_select_file");
  assert!(!s.sign_select_key().is_empty(), "sign_select_key");
  assert!(!s.sign_no_keys().is_empty(), "sign_no_keys");
  assert!(!s.sign_about().is_empty(), "sign_about");
  assert!(!s.verify_title().is_empty(), "verify_title");
  assert!(!s.verify_select_file().is_empty(), "verify_select_file");
  assert!(!s.verify_outcome_valid().is_empty(), "verify_outcome_valid");
  assert!(
    !s.verify_outcome_bad_sig().is_empty(),
    "verify_outcome_bad_sig"
  );
  assert!(
    !s.verify_outcome_unknown_key().is_empty(),
    "verify_outcome_unknown_key"
  );
  assert!(
    !s.verify_outcome_expired_key().is_empty(),
    "verify_outcome_expired_key"
  );
  assert!(
    !s.verify_outcome_revoked_key().is_empty(),
    "verify_outcome_revoked_key"
  );
  assert!(!s.verify_no_file().is_empty(), "verify_no_file");
  assert!(!s.verify_sig_auto_hint().is_empty(), "verify_sig_auto_hint");
  assert!(!s.verify_signed_by().is_empty(), "verify_signed_by");
  assert!(!s.verify_signed_on().is_empty(), "verify_signed_on");
  assert!(!s.verify_in_progress().is_empty(), "verify_in_progress");
  assert!(!s.verify_error_prefix().is_empty(), "verify_error_prefix");
  assert!(
    !s.verify_valid_full_trust().is_empty(),
    "verify_valid_full_trust"
  );
  assert!(
    !s.verify_valid_marginal_trust().is_empty(),
    "verify_valid_marginal_trust"
  );
  assert!(
    !s.verify_valid_no_trust().is_empty(),
    "verify_valid_no_trust"
  );
  assert!(
    !s.verify_sig_file_placeholder().is_empty(),
    "verify_sig_file_placeholder"
  );
  assert!(!s.verify_trust_warning().is_empty(), "verify_trust_warning");
  assert!(
    !s.verify_fingerprint_label().is_empty(),
    "verify_fingerprint_label"
  );
  assert!(!s.verify_bad_sig_desc().is_empty(), "verify_bad_sig_desc");
  assert!(
    !s.verify_unknown_key_desc().is_empty(),
    "verify_unknown_key_desc"
  );
  assert!(
    !s.verify_expired_key_desc().is_empty(),
    "verify_expired_key_desc"
  );
  assert!(
    !s.verify_revoked_key_desc().is_empty(),
    "verify_revoked_key_desc"
  );
  assert!(!s.verify_about().is_empty(), "verify_about");
  assert!(
    !s.verify_sig_auto_hint_with_name("test.sig").is_empty(),
    "verify_sig_auto_hint_with_name"
  );

  // Health / Diagnostic
  assert!(!s.health_title().is_empty(), "health_title");
  assert!(!s.health_ok().is_empty(), "health_ok");
  assert!(!s.health_warning().is_empty(), "health_warning");
  assert!(!s.health_error().is_empty(), "health_error");
  assert!(!s.health_info().is_empty(), "health_info");
  assert!(
    !s.health_diagnostics_title().is_empty(),
    "health_diagnostics_title"
  );
  assert!(
    !s.health_diagnostics_desc().is_empty(),
    "health_diagnostics_desc"
  );
  assert!(!s.health_checking().is_empty(), "health_checking");
  assert!(
    !s.health_category_installation().is_empty(),
    "health_category_installation"
  );
  assert!(
    !s.health_category_agent().is_empty(),
    "health_category_agent"
  );
  assert!(
    !s.health_category_security().is_empty(),
    "health_category_security"
  );

  // Import
  assert!(!s.import_title().is_empty(), "import_title");
  assert!(!s.import_tab_file().is_empty(), "import_tab_file");
  assert!(!s.import_tab_url().is_empty(), "import_tab_url");
  assert!(!s.import_tab_keyserver().is_empty(), "import_tab_keyserver");
  assert!(!s.import_tab_paste().is_empty(), "import_tab_paste");
  assert!(
    !s.import_source_from_file().is_empty(),
    "import_source_from_file"
  );
  assert!(!s.import_select_source().is_empty(), "import_select_source");
  assert!(!s.import_url_hint().is_empty(), "import_url_hint");
  assert!(!s.import_url_button().is_empty(), "import_url_button");
  assert!(
    !s.import_keyserver_hint().is_empty(),
    "import_keyserver_hint"
  );
  assert!(
    !s.import_keyserver_button().is_empty(),
    "import_keyserver_button"
  );
  assert!(!s.import_paste_hint().is_empty(), "import_paste_hint");
  assert!(!s.import_paste_button().is_empty(), "import_paste_button");

  // Keyserver
  assert!(!s.keyserver_openpgp().is_empty(), "keyserver_openpgp");
  assert!(!s.keyserver_ubuntu().is_empty(), "keyserver_ubuntu");
  assert!(
    !s.keyserver_status_unknown().is_empty(),
    "keyserver_status_unknown"
  );
  assert!(
    !s.keyserver_status_published().is_empty(),
    "keyserver_status_published"
  );
  assert!(
    !s.keyserver_status_not_published().is_empty(),
    "keyserver_status_not_published"
  );
  assert!(
    !s.keyserver_badge_published().is_empty(),
    "keyserver_badge_published"
  );
  assert!(
    !s.keyserver_badge_not_published().is_empty(),
    "keyserver_badge_not_published"
  );
  assert!(
    !s.keyserver_badge_checking().is_empty(),
    "keyserver_badge_checking"
  );
  assert!(
    !s.keyserver_badge_link_btn().is_empty(),
    "keyserver_badge_link_btn"
  );

  // Settings
  assert!(!s.settings_title().is_empty(), "settings_title");
  assert!(!s.settings_language().is_empty(), "settings_language");
  assert!(
    !s.settings_language_english().is_empty(),
    "settings_language_english"
  );
  assert!(
    !s.settings_language_french().is_empty(),
    "settings_language_french"
  );
  assert!(
    !s.settings_scale_factor().is_empty(),
    "settings_scale_factor"
  );
  assert!(
    !s.settings_scale_factor_hint().is_empty(),
    "settings_scale_factor_hint"
  );
  assert!(!s.settings_theme().is_empty(), "settings_theme");
  assert!(
    !s.settings_theme_catppuccin().is_empty(),
    "settings_theme_catppuccin"
  );
  assert!(!s.settings_theme_ussr().is_empty(), "settings_theme_ussr");

  // Modals
  assert!(!s.modal_delete_title().is_empty(), "modal_delete_title");
  assert!(
    !s.modal_delete_stub_only().is_empty(),
    "modal_delete_stub_only"
  );
  assert!(
    !s.modal_delete_stub_body().is_empty(),
    "modal_delete_stub_body"
  );
  assert!(!s.modal_delete_secret().is_empty(), "modal_delete_secret");
  assert!(
    !s.modal_delete_secret_body().is_empty(),
    "modal_delete_secret_body"
  );
  assert!(!s.modal_delete_public().is_empty(), "modal_delete_public");
  assert!(
    !s.modal_delete_public_body().is_empty(),
    "modal_delete_public_body"
  );
  assert!(
    !s.modal_migration_irreversible().is_empty(),
    "modal_migration_irreversible"
  );
  assert!(
    !s.modal_migration_backup_warning().is_empty(),
    "modal_migration_backup_warning"
  );
  assert!(
    !s.modal_migration_backup_btn().is_empty(),
    "modal_migration_backup_btn"
  );
  assert!(
    !s.modal_migration_confirm_btn().is_empty(),
    "modal_migration_confirm_btn"
  );
  assert!(
    !s.modal_migration_cancel_btn().is_empty(),
    "modal_migration_cancel_btn"
  );
  assert!(
    !s.modal_delete_export_first_btn().is_empty(),
    "modal_delete_export_first_btn"
  );
  assert!(
    !s.modal_delete_confirm_btn().is_empty(),
    "modal_delete_confirm_btn"
  );
  assert!(
    !s.modal_delete_cancel_btn().is_empty(),
    "modal_delete_cancel_btn"
  );
  assert!(
    !s.modal_publish_recommended().is_empty(),
    "modal_publish_recommended"
  );
  assert!(
    !s.modal_publish_openpgp_desc().is_empty(),
    "modal_publish_openpgp_desc"
  );
  assert!(
    !s.modal_publish_ubuntu_desc().is_empty(),
    "modal_publish_ubuntu_desc"
  );
  assert!(
    !s.modal_publish_privacy().is_empty(),
    "modal_publish_privacy"
  );
  assert!(
    !s.modal_publish_confirm_btn().is_empty(),
    "modal_publish_confirm_btn"
  );
  assert!(
    !s.modal_publish_select_keyserver().is_empty(),
    "modal_publish_select_keyserver"
  );

  // Key list
  assert!(!s.key_list_error("test").is_empty(), "key_list_error");
  assert!(!s.key_list_header_name().is_empty(), "key_list_header_name");
  assert!(
    !s.key_list_header_expires().is_empty(),
    "key_list_header_expires"
  );
  assert!(
    !s.key_list_header_status().is_empty(),
    "key_list_header_status"
  );
  assert!(!s.key_list_select_hint().is_empty(), "key_list_select_hint");

  // Key detail
  assert!(!s.key_type_on_card().is_empty(), "key_type_on_card");
  assert!(
    !s.key_type_public_private().is_empty(),
    "key_type_public_private"
  );
  assert!(!s.key_type_public_only().is_empty(), "key_type_public_only");
  assert!(
    !s.subkey_type_signature().is_empty(),
    "subkey_type_signature"
  );
  assert!(
    !s.subkey_type_encryption().is_empty(),
    "subkey_type_encryption"
  );
  assert!(!s.subkey_type_ssh_auth().is_empty(), "subkey_type_ssh_auth");
  assert!(
    !s.export_menu_save_disk().is_empty(),
    "export_menu_save_disk"
  );
  assert!(
    !s.export_menu_copy_clipboard().is_empty(),
    "export_menu_copy_clipboard"
  );
  assert!(
    !s.export_menu_paste_link().is_empty(),
    "export_menu_paste_link"
  );
  assert!(!s.subkey_expiry_1_year().is_empty(), "subkey_expiry_1_year");
  assert!(
    !s.subkey_expiry_2_years().is_empty(),
    "subkey_expiry_2_years"
  );
  assert!(
    !s.subkey_expiry_5_years().is_empty(),
    "subkey_expiry_5_years"
  );

  // Create key
  assert!(
    !s.create_key_generating().is_empty(),
    "create_key_generating"
  );
  assert!(!s.create_key_title().is_empty(), "create_key_title");
  assert!(!s.create_key_subtitle().is_empty(), "create_key_subtitle");
  assert!(
    !s.create_key_section_identity().is_empty(),
    "create_key_section_identity"
  );
  assert!(
    !s.create_key_field_name().is_empty(),
    "create_key_field_name"
  );
  assert!(
    !s.create_key_field_email().is_empty(),
    "create_key_field_email"
  );
  assert!(
    !s.create_key_section_subkeys().is_empty(),
    "create_key_section_subkeys"
  );
  assert!(
    !s.create_key_section_expiration().is_empty(),
    "create_key_section_expiration"
  );
  assert!(
    !s.create_key_include_ssh().is_empty(),
    "create_key_include_ssh"
  );
  assert!(
    !s.create_key_about_master().is_empty(),
    "create_key_about_master"
  );
  assert!(
    !s.create_key_hint_expiry().is_empty(),
    "create_key_hint_expiry"
  );
  assert!(!s.create_key_hint_ssh().is_empty(), "create_key_hint_ssh");
  assert!(
    !s.create_key_hint_master().is_empty(),
    "create_key_hint_master"
  );

  // File dialogs
  assert!(
    !s.dialog_choose_files_encrypt().is_empty(),
    "dialog_choose_files_encrypt"
  );
  assert!(
    !s.dialog_choose_files_decrypt().is_empty(),
    "dialog_choose_files_decrypt"
  );
  assert!(
    !s.dialog_filter_gpg_files().is_empty(),
    "dialog_filter_gpg_files"
  );
  assert!(
    !s.dialog_choose_file_sign().is_empty(),
    "dialog_choose_file_sign"
  );
  assert!(
    !s.dialog_choose_file_verify().is_empty(),
    "dialog_choose_file_verify"
  );
  assert!(
    !s.dialog_choose_sig_file().is_empty(),
    "dialog_choose_sig_file"
  );
  assert!(
    !s.dialog_choose_backup_folder().is_empty(),
    "dialog_choose_backup_folder"
  );

  // Misc
  assert!(!s.no_file_selected().is_empty(), "no_file_selected");
  assert!(!s.loading().is_empty(), "loading");
  assert!(!s.no_keys().is_empty(), "no_keys");

  // Decrypt
  assert!(!s.decrypt_title().is_empty(), "decrypt_title");
  assert!(!s.decrypt_add_files().is_empty(), "decrypt_add_files");
  assert!(!s.decrypt_in_progress().is_empty(), "decrypt_in_progress");
  assert!(
    !s.decrypt_auto_key_hint().is_empty(),
    "decrypt_auto_key_hint"
  );
  assert!(!s.decrypt_drop_hint().is_empty(), "decrypt_drop_hint");
  assert!(
    !s.decrypt_key_available().is_empty(),
    "decrypt_key_available"
  );
  assert!(!s.decrypt_key_missing().is_empty(), "decrypt_key_missing");
  assert!(!s.decrypt_key_checking().is_empty(), "decrypt_key_checking");
  assert!(
    !s.decrypt_no_key_warning().is_empty(),
    "decrypt_no_key_warning"
  );
  assert!(!s.decrypt_about().is_empty(), "decrypt_about");
}

// ---------------------------------------------------------------------------
// Test 3 — translated strings differ between EN and FR
//
// Excluded from this test (EN == FR by design):
//   btn_ok                  — "OK" in both
//   health_ok               — "OK" in both
//   health_info             — "Info" in both
//   keyserver_openpgp       — technical hostname, not translated
//   keyserver_ubuntu        — technical hostname, not translated
//   settings_language_english — "English" in both (the name of the language)
//   settings_theme_catppuccin — proper name, not translated
//   settings_theme_ussr     — acronym, not translated
//   subkey_type_signature   — "Signature" in both
//   subkey_type_ssh_auth    — "Auth SSH" in both
//   health_category_installation — "Installation" in both
//   create_key_field_email  — "Email" in both
//   encrypt_format_armor    — ".asc (armored)" in both
// ---------------------------------------------------------------------------
#[test]
fn english_differs_from_french() {
  let e = en();
  let f = fr();

  // Navigation
  assert_ne!(e.nav_my_keys(), f.nav_my_keys(), "nav_my_keys");
  assert_ne!(e.nav_public_keys(), f.nav_public_keys(), "nav_public_keys");
  assert_ne!(e.nav_import(), f.nav_import(), "nav_import");
  assert_ne!(e.nav_create_key(), f.nav_create_key(), "nav_create_key");
  assert_ne!(e.nav_encrypt(), f.nav_encrypt(), "nav_encrypt");
  assert_ne!(e.nav_decrypt(), f.nav_decrypt(), "nav_decrypt");
  assert_ne!(e.nav_sign(), f.nav_sign(), "nav_sign");
  assert_ne!(e.nav_verify(), f.nav_verify(), "nav_verify");
  assert_ne!(e.nav_health(), f.nav_health(), "nav_health");
  assert_ne!(e.nav_settings(), f.nav_settings(), "nav_settings");
  assert_ne!(
    e.sidebar_section_keys(),
    f.sidebar_section_keys(),
    "sidebar_section_keys"
  );
  assert_ne!(
    e.sidebar_section_tools(),
    f.sidebar_section_tools(),
    "sidebar_section_tools"
  );

  // Buttons that differ
  assert_ne!(e.btn_cancel(), f.btn_cancel(), "btn_cancel");
  assert_ne!(e.btn_confirm(), f.btn_confirm(), "btn_confirm");
  assert_ne!(e.btn_back(), f.btn_back(), "btn_back");
  assert_ne!(e.btn_create(), f.btn_create(), "btn_create");
  assert_ne!(e.btn_delete(), f.btn_delete(), "btn_delete");
  assert_ne!(e.btn_export(), f.btn_export(), "btn_export");
  assert_ne!(e.btn_import(), f.btn_import(), "btn_import");
  assert_ne!(e.btn_copy(), f.btn_copy(), "btn_copy");
  assert_ne!(e.btn_publish(), f.btn_publish(), "btn_publish");
  assert_ne!(e.btn_backup(), f.btn_backup(), "btn_backup");
  assert_ne!(e.btn_migrate(), f.btn_migrate(), "btn_migrate");
  assert_ne!(e.btn_renew(), f.btn_renew(), "btn_renew");
  assert_ne!(e.btn_rotate(), f.btn_rotate(), "btn_rotate");
  assert_ne!(e.btn_add_subkey(), f.btn_add_subkey(), "btn_add_subkey");
  assert_ne!(
    e.btn_export_public(),
    f.btn_export_public(),
    "btn_export_public"
  );
  assert_ne!(e.btn_backup_key(), f.btn_backup_key(), "btn_backup_key");
  assert_ne!(
    e.btn_migrate_yubikey(),
    f.btn_migrate_yubikey(),
    "btn_migrate_yubikey"
  );
  assert_ne!(e.btn_decrypt(), f.btn_decrypt(), "btn_decrypt");
  assert_ne!(e.btn_verify(), f.btn_verify(), "btn_verify");
  assert_ne!(e.btn_sign(), f.btn_sign(), "btn_sign");
  assert_ne!(e.btn_encrypt(), f.btn_encrypt(), "btn_encrypt");

  // Key labels
  assert_ne!(e.key_fingerprint(), f.key_fingerprint(), "key_fingerprint");
  assert_ne!(e.key_created(), f.key_created(), "key_created");
  assert_ne!(e.key_expires(), f.key_expires(), "key_expires");
  assert_ne!(
    e.key_never_expires(),
    f.key_never_expires(),
    "key_never_expires"
  );
  assert_ne!(e.key_trust(), f.key_trust(), "key_trust");
  assert_ne!(e.key_subkeys(), f.key_subkeys(), "key_subkeys");
  assert_ne!(e.key_no_subkeys(), f.key_no_subkeys(), "key_no_subkeys");

  // Trust levels
  assert_ne!(e.trust_undefined(), f.trust_undefined(), "trust_undefined");
  assert_ne!(e.trust_marginal(), f.trust_marginal(), "trust_marginal");
  assert_ne!(e.trust_full(), f.trust_full(), "trust_full");
  assert_ne!(e.trust_ultimate(), f.trust_ultimate(), "trust_ultimate");

  // Status messages
  assert_ne!(
    e.status_key_created(),
    f.status_key_created(),
    "status_key_created"
  );
  assert_ne!(
    e.status_key_deleted(),
    f.status_key_deleted(),
    "status_key_deleted"
  );
  assert_ne!(
    e.status_key_exported(),
    f.status_key_exported(),
    "status_key_exported"
  );
  assert_ne!(
    e.status_key_imported(),
    f.status_key_imported(),
    "status_key_imported"
  );
  assert_ne!(
    e.status_published(),
    f.status_published(),
    "status_published"
  );
  assert_ne!(
    e.status_publish_failed(),
    f.status_publish_failed(),
    "status_publish_failed"
  );
  assert_ne!(
    e.status_backup_done(),
    f.status_backup_done(),
    "status_backup_done"
  );
  assert_ne!(
    e.status_preferences_saved(),
    f.status_preferences_saved(),
    "status_preferences_saved"
  );
  assert_ne!(
    e.status_key_copied(),
    f.status_key_copied(),
    "status_key_copied"
  );
  assert_ne!(
    e.status_link_copied(),
    f.status_link_copied(),
    "status_link_copied"
  );
  assert_ne!(
    e.status_card_migrated(),
    f.status_card_migrated(),
    "status_card_migrated"
  );
  assert_ne!(
    e.status_subkey_renewed(),
    f.status_subkey_renewed(),
    "status_subkey_renewed"
  );
  assert_ne!(
    e.status_subkey_rotated(),
    f.status_subkey_rotated(),
    "status_subkey_rotated"
  );
  assert_ne!(
    e.status_file_signed(),
    f.status_file_signed(),
    "status_file_signed"
  );
  assert_ne!(
    e.status_files_encrypted(),
    f.status_files_encrypted(),
    "status_files_encrypted"
  );
  assert_ne!(
    e.status_trust_updated(),
    f.status_trust_updated(),
    "status_trust_updated"
  );
  assert_ne!(
    e.status_subkey_created(),
    f.status_subkey_created(),
    "status_subkey_created"
  );
  assert_ne!(
    e.status_published_openpgp_email(),
    f.status_published_openpgp_email(),
    "status_published_openpgp_email"
  );
  assert_ne!(
    e.status_files_decrypted(),
    f.status_files_decrypted(),
    "status_files_decrypted"
  );

  // Error messages
  assert_ne!(
    e.err_gpg_not_found(),
    f.err_gpg_not_found(),
    "err_gpg_not_found"
  );
  assert_ne!(e.err_invalid_key(), f.err_invalid_key(), "err_invalid_key");
  assert_ne!(
    e.err_import_not_pgp(),
    f.err_import_not_pgp(),
    "err_import_not_pgp"
  );
  assert_ne!(
    e.err_export_failed(),
    f.err_export_failed(),
    "err_export_failed"
  );
  assert_ne!(
    e.err_delete_failed(),
    f.err_delete_failed(),
    "err_delete_failed"
  );
  assert_ne!(
    e.err_create_failed(),
    f.err_create_failed(),
    "err_create_failed"
  );
  assert_ne!(
    e.err_import_failed(),
    f.err_import_failed(),
    "err_import_failed"
  );
  assert_ne!(
    e.err_subkey_renew_failed(),
    f.err_subkey_renew_failed(),
    "err_subkey_renew_failed"
  );
  assert_ne!(e.err_sign_failed(), f.err_sign_failed(), "err_sign_failed");
  assert_ne!(
    e.err_encrypt_failed(),
    f.err_encrypt_failed(),
    "err_encrypt_failed"
  );
  assert_ne!(
    e.err_backup_failed(),
    f.err_backup_failed(),
    "err_backup_failed"
  );
  assert_ne!(
    e.err_upload_failed(),
    f.err_upload_failed(),
    "err_upload_failed"
  );
  assert_ne!(
    e.err_save_config_failed(),
    f.err_save_config_failed(),
    "err_save_config_failed"
  );
  assert_ne!(
    e.err_trust_failed(),
    f.err_trust_failed(),
    "err_trust_failed"
  );
  assert_ne!(
    e.err_diagnostic_failed(),
    f.err_diagnostic_failed(),
    "err_diagnostic_failed"
  );
  assert_ne!(
    e.err_subkey_add_failed(),
    f.err_subkey_add_failed(),
    "err_subkey_add_failed"
  );
  assert_ne!(
    e.err_republish_failed(),
    f.err_republish_failed(),
    "err_republish_failed"
  );
  assert_ne!(
    e.err_decrypt_failed(),
    f.err_decrypt_failed(),
    "err_decrypt_failed"
  );
  assert_ne!(
    e.err_no_decryptable_file(),
    f.err_no_decryptable_file(),
    "err_no_decryptable_file"
  );

  // Encrypt
  assert_ne!(e.encrypt_title(), f.encrypt_title(), "encrypt_title");
  assert_ne!(
    e.encrypt_add_files(),
    f.encrypt_add_files(),
    "encrypt_add_files"
  );
  assert_ne!(
    e.encrypt_recipients(),
    f.encrypt_recipients(),
    "encrypt_recipients"
  );
  assert_ne!(
    e.encrypt_no_recipients(),
    f.encrypt_no_recipients(),
    "encrypt_no_recipients"
  );
  assert_ne!(
    e.encrypt_trust_warning_title(),
    f.encrypt_trust_warning_title(),
    "encrypt_trust_warning_title"
  );
  assert_ne!(
    e.encrypt_trust_warning_body(),
    f.encrypt_trust_warning_body(),
    "encrypt_trust_warning_body"
  );
  assert_ne!(
    e.encrypt_format_binary(),
    f.encrypt_format_binary(),
    "encrypt_format_binary"
  );
  assert_ne!(
    e.encrypt_tab_my_keys(),
    f.encrypt_tab_my_keys(),
    "encrypt_tab_my_keys"
  );
  assert_ne!(
    e.encrypt_tab_public_keys(),
    f.encrypt_tab_public_keys(),
    "encrypt_tab_public_keys"
  );
  assert_ne!(e.encrypt_no_keys(), f.encrypt_no_keys(), "encrypt_no_keys");
  assert_ne!(
    e.encrypt_choose_files(),
    f.encrypt_choose_files(),
    "encrypt_choose_files"
  );
  assert_ne!(
    e.encrypt_drop_hint(),
    f.encrypt_drop_hint(),
    "encrypt_drop_hint"
  );
  assert_ne!(
    e.encrypt_format_ascii_desc(),
    f.encrypt_format_ascii_desc(),
    "encrypt_format_ascii_desc"
  );
  assert_ne!(
    e.encrypt_format_binary_desc(),
    f.encrypt_format_binary_desc(),
    "encrypt_format_binary_desc"
  );
  assert_ne!(
    e.encrypt_multi_recipient_hint(),
    f.encrypt_multi_recipient_hint(),
    "encrypt_multi_recipient_hint"
  );
  assert_ne!(
    e.encrypt_select_hint(),
    f.encrypt_select_hint(),
    "encrypt_select_hint"
  );
  assert_ne!(
    e.encrypt_in_progress(),
    f.encrypt_in_progress(),
    "encrypt_in_progress"
  );

  // Sign / Verify
  assert_ne!(e.sign_title(), f.sign_title(), "sign_title");
  assert_ne!(
    e.sign_select_file(),
    f.sign_select_file(),
    "sign_select_file"
  );
  assert_ne!(e.sign_select_key(), f.sign_select_key(), "sign_select_key");
  assert_ne!(e.sign_no_keys(), f.sign_no_keys(), "sign_no_keys");
  assert_ne!(e.sign_about(), f.sign_about(), "sign_about");
  assert_ne!(e.verify_title(), f.verify_title(), "verify_title");
  assert_ne!(
    e.verify_select_file(),
    f.verify_select_file(),
    "verify_select_file"
  );
  assert_ne!(
    e.verify_outcome_valid(),
    f.verify_outcome_valid(),
    "verify_outcome_valid"
  );
  assert_ne!(
    e.verify_outcome_bad_sig(),
    f.verify_outcome_bad_sig(),
    "verify_outcome_bad_sig"
  );
  assert_ne!(
    e.verify_outcome_unknown_key(),
    f.verify_outcome_unknown_key(),
    "verify_outcome_unknown_key"
  );
  assert_ne!(
    e.verify_outcome_expired_key(),
    f.verify_outcome_expired_key(),
    "verify_outcome_expired_key"
  );
  assert_ne!(
    e.verify_outcome_revoked_key(),
    f.verify_outcome_revoked_key(),
    "verify_outcome_revoked_key"
  );
  assert_ne!(e.verify_no_file(), f.verify_no_file(), "verify_no_file");
  assert_ne!(
    e.verify_sig_auto_hint(),
    f.verify_sig_auto_hint(),
    "verify_sig_auto_hint"
  );
  assert_ne!(
    e.verify_signed_by(),
    f.verify_signed_by(),
    "verify_signed_by"
  );
  assert_ne!(
    e.verify_signed_on(),
    f.verify_signed_on(),
    "verify_signed_on"
  );
  assert_ne!(
    e.verify_in_progress(),
    f.verify_in_progress(),
    "verify_in_progress"
  );
  assert_ne!(
    e.verify_error_prefix(),
    f.verify_error_prefix(),
    "verify_error_prefix"
  );
  assert_ne!(
    e.verify_valid_full_trust(),
    f.verify_valid_full_trust(),
    "verify_valid_full_trust"
  );
  assert_ne!(
    e.verify_valid_marginal_trust(),
    f.verify_valid_marginal_trust(),
    "verify_valid_marginal_trust"
  );
  assert_ne!(
    e.verify_valid_no_trust(),
    f.verify_valid_no_trust(),
    "verify_valid_no_trust"
  );
  assert_ne!(
    e.verify_sig_file_placeholder(),
    f.verify_sig_file_placeholder(),
    "verify_sig_file_placeholder"
  );
  assert_ne!(
    e.verify_trust_warning(),
    f.verify_trust_warning(),
    "verify_trust_warning"
  );
  assert_ne!(
    e.verify_fingerprint_label(),
    f.verify_fingerprint_label(),
    "verify_fingerprint_label"
  );
  assert_ne!(
    e.verify_bad_sig_desc(),
    f.verify_bad_sig_desc(),
    "verify_bad_sig_desc"
  );
  assert_ne!(
    e.verify_unknown_key_desc(),
    f.verify_unknown_key_desc(),
    "verify_unknown_key_desc"
  );
  assert_ne!(
    e.verify_expired_key_desc(),
    f.verify_expired_key_desc(),
    "verify_expired_key_desc"
  );
  assert_ne!(
    e.verify_revoked_key_desc(),
    f.verify_revoked_key_desc(),
    "verify_revoked_key_desc"
  );
  assert_ne!(e.verify_about(), f.verify_about(), "verify_about");

  // Health
  assert_ne!(e.health_title(), f.health_title(), "health_title");
  assert_ne!(e.health_warning(), f.health_warning(), "health_warning");
  assert_ne!(e.health_error(), f.health_error(), "health_error");
  assert_ne!(
    e.health_diagnostics_title(),
    f.health_diagnostics_title(),
    "health_diagnostics_title"
  );
  assert_ne!(
    e.health_diagnostics_desc(),
    f.health_diagnostics_desc(),
    "health_diagnostics_desc"
  );
  assert_ne!(e.health_checking(), f.health_checking(), "health_checking");
  assert_ne!(
    e.health_category_agent(),
    f.health_category_agent(),
    "health_category_agent"
  );
  assert_ne!(
    e.health_category_security(),
    f.health_category_security(),
    "health_category_security"
  );

  // Import
  assert_ne!(e.import_title(), f.import_title(), "import_title");
  assert_ne!(e.import_tab_file(), f.import_tab_file(), "import_tab_file");
  assert_ne!(e.import_tab_url(), f.import_tab_url(), "import_tab_url");
  assert_ne!(
    e.import_tab_keyserver(),
    f.import_tab_keyserver(),
    "import_tab_keyserver"
  );
  assert_ne!(
    e.import_tab_paste(),
    f.import_tab_paste(),
    "import_tab_paste"
  );
  assert_ne!(
    e.import_source_from_file(),
    f.import_source_from_file(),
    "import_source_from_file"
  );
  assert_ne!(
    e.import_select_source(),
    f.import_select_source(),
    "import_select_source"
  );
  assert_ne!(e.import_url_hint(), f.import_url_hint(), "import_url_hint");
  assert_ne!(
    e.import_url_button(),
    f.import_url_button(),
    "import_url_button"
  );
  assert_ne!(
    e.import_keyserver_hint(),
    f.import_keyserver_hint(),
    "import_keyserver_hint"
  );
  assert_ne!(
    e.import_keyserver_button(),
    f.import_keyserver_button(),
    "import_keyserver_button"
  );
  assert_ne!(
    e.import_paste_hint(),
    f.import_paste_hint(),
    "import_paste_hint"
  );
  assert_ne!(
    e.import_paste_button(),
    f.import_paste_button(),
    "import_paste_button"
  );

  // Keyserver status / badges
  assert_ne!(
    e.keyserver_status_unknown(),
    f.keyserver_status_unknown(),
    "keyserver_status_unknown"
  );
  assert_ne!(
    e.keyserver_status_published(),
    f.keyserver_status_published(),
    "keyserver_status_published"
  );
  assert_ne!(
    e.keyserver_status_not_published(),
    f.keyserver_status_not_published(),
    "keyserver_status_not_published"
  );
  assert_ne!(
    e.keyserver_badge_published(),
    f.keyserver_badge_published(),
    "keyserver_badge_published"
  );
  assert_ne!(
    e.keyserver_badge_not_published(),
    f.keyserver_badge_not_published(),
    "keyserver_badge_not_published"
  );
  assert_ne!(
    e.keyserver_badge_checking(),
    f.keyserver_badge_checking(),
    "keyserver_badge_checking"
  );
  assert_ne!(
    e.keyserver_badge_link_btn(),
    f.keyserver_badge_link_btn(),
    "keyserver_badge_link_btn"
  );

  // Settings
  assert_ne!(e.settings_title(), f.settings_title(), "settings_title");
  assert_ne!(
    e.settings_language(),
    f.settings_language(),
    "settings_language"
  );
  assert_ne!(
    e.settings_language_french(),
    f.settings_language_french(),
    "settings_language_french"
  );
  assert_ne!(
    e.settings_scale_factor(),
    f.settings_scale_factor(),
    "settings_scale_factor"
  );
  assert_ne!(
    e.settings_scale_factor_hint(),
    f.settings_scale_factor_hint(),
    "settings_scale_factor_hint"
  );
  // settings_theme is "Theme" in both EN and FR — intentionally excluded

  // Modals
  assert_ne!(
    e.modal_delete_title(),
    f.modal_delete_title(),
    "modal_delete_title"
  );
  assert_ne!(
    e.modal_delete_stub_only(),
    f.modal_delete_stub_only(),
    "modal_delete_stub_only"
  );
  assert_ne!(
    e.modal_delete_stub_body(),
    f.modal_delete_stub_body(),
    "modal_delete_stub_body"
  );
  assert_ne!(
    e.modal_delete_secret(),
    f.modal_delete_secret(),
    "modal_delete_secret"
  );
  assert_ne!(
    e.modal_delete_secret_body(),
    f.modal_delete_secret_body(),
    "modal_delete_secret_body"
  );
  assert_ne!(
    e.modal_delete_public(),
    f.modal_delete_public(),
    "modal_delete_public"
  );
  assert_ne!(
    e.modal_delete_public_body(),
    f.modal_delete_public_body(),
    "modal_delete_public_body"
  );
  assert_ne!(
    e.modal_migration_irreversible(),
    f.modal_migration_irreversible(),
    "modal_migration_irreversible"
  );
  assert_ne!(
    e.modal_migration_backup_warning(),
    f.modal_migration_backup_warning(),
    "modal_migration_backup_warning"
  );
  assert_ne!(
    e.modal_migration_backup_btn(),
    f.modal_migration_backup_btn(),
    "modal_migration_backup_btn"
  );
  assert_ne!(
    e.modal_migration_confirm_btn(),
    f.modal_migration_confirm_btn(),
    "modal_migration_confirm_btn"
  );
  assert_ne!(
    e.modal_migration_cancel_btn(),
    f.modal_migration_cancel_btn(),
    "modal_migration_cancel_btn"
  );
  assert_ne!(
    e.modal_delete_export_first_btn(),
    f.modal_delete_export_first_btn(),
    "modal_delete_export_first_btn"
  );
  assert_ne!(
    e.modal_delete_confirm_btn(),
    f.modal_delete_confirm_btn(),
    "modal_delete_confirm_btn"
  );
  assert_ne!(
    e.modal_delete_cancel_btn(),
    f.modal_delete_cancel_btn(),
    "modal_delete_cancel_btn"
  );
  assert_ne!(
    e.modal_publish_recommended(),
    f.modal_publish_recommended(),
    "modal_publish_recommended"
  );
  assert_ne!(
    e.modal_publish_openpgp_desc(),
    f.modal_publish_openpgp_desc(),
    "modal_publish_openpgp_desc"
  );
  assert_ne!(
    e.modal_publish_ubuntu_desc(),
    f.modal_publish_ubuntu_desc(),
    "modal_publish_ubuntu_desc"
  );
  assert_ne!(
    e.modal_publish_privacy(),
    f.modal_publish_privacy(),
    "modal_publish_privacy"
  );
  assert_ne!(
    e.modal_publish_confirm_btn(),
    f.modal_publish_confirm_btn(),
    "modal_publish_confirm_btn"
  );
  assert_ne!(
    e.modal_publish_select_keyserver(),
    f.modal_publish_select_keyserver(),
    "modal_publish_select_keyserver"
  );

  // Key list
  assert_ne!(
    e.key_list_header_name(),
    f.key_list_header_name(),
    "key_list_header_name"
  );
  assert_ne!(
    e.key_list_header_expires(),
    f.key_list_header_expires(),
    "key_list_header_expires"
  );
  assert_ne!(
    e.key_list_header_status(),
    f.key_list_header_status(),
    "key_list_header_status"
  );
  assert_ne!(
    e.key_list_select_hint(),
    f.key_list_select_hint(),
    "key_list_select_hint"
  );

  // Key detail
  assert_ne!(
    e.key_type_on_card(),
    f.key_type_on_card(),
    "key_type_on_card"
  );
  assert_ne!(
    e.key_type_public_private(),
    f.key_type_public_private(),
    "key_type_public_private"
  );
  assert_ne!(
    e.key_type_public_only(),
    f.key_type_public_only(),
    "key_type_public_only"
  );
  assert_ne!(
    e.subkey_type_encryption(),
    f.subkey_type_encryption(),
    "subkey_type_encryption"
  );
  assert_ne!(
    e.export_menu_save_disk(),
    f.export_menu_save_disk(),
    "export_menu_save_disk"
  );
  assert_ne!(
    e.export_menu_copy_clipboard(),
    f.export_menu_copy_clipboard(),
    "export_menu_copy_clipboard"
  );
  assert_ne!(
    e.export_menu_paste_link(),
    f.export_menu_paste_link(),
    "export_menu_paste_link"
  );
  assert_ne!(
    e.subkey_expiry_1_year(),
    f.subkey_expiry_1_year(),
    "subkey_expiry_1_year"
  );
  assert_ne!(
    e.subkey_expiry_2_years(),
    f.subkey_expiry_2_years(),
    "subkey_expiry_2_years"
  );
  assert_ne!(
    e.subkey_expiry_5_years(),
    f.subkey_expiry_5_years(),
    "subkey_expiry_5_years"
  );

  // Create key
  assert_ne!(
    e.create_key_generating(),
    f.create_key_generating(),
    "create_key_generating"
  );
  assert_ne!(
    e.create_key_title(),
    f.create_key_title(),
    "create_key_title"
  );
  assert_ne!(
    e.create_key_subtitle(),
    f.create_key_subtitle(),
    "create_key_subtitle"
  );
  assert_ne!(
    e.create_key_section_identity(),
    f.create_key_section_identity(),
    "create_key_section_identity"
  );
  assert_ne!(
    e.create_key_field_name(),
    f.create_key_field_name(),
    "create_key_field_name"
  );
  assert_ne!(
    e.create_key_section_subkeys(),
    f.create_key_section_subkeys(),
    "create_key_section_subkeys"
  );
  // create_key_section_expiration is "Expiration" in both — excluded like "Installation"
  assert_ne!(
    e.create_key_include_ssh(),
    f.create_key_include_ssh(),
    "create_key_include_ssh"
  );
  assert_ne!(
    e.create_key_about_master(),
    f.create_key_about_master(),
    "create_key_about_master"
  );
  assert_ne!(
    e.create_key_hint_expiry(),
    f.create_key_hint_expiry(),
    "create_key_hint_expiry"
  );
  assert_ne!(
    e.create_key_hint_ssh(),
    f.create_key_hint_ssh(),
    "create_key_hint_ssh"
  );
  assert_ne!(
    e.create_key_hint_master(),
    f.create_key_hint_master(),
    "create_key_hint_master"
  );

  // File dialogs
  assert_ne!(
    e.dialog_choose_files_encrypt(),
    f.dialog_choose_files_encrypt(),
    "dialog_choose_files_encrypt"
  );
  assert_ne!(
    e.dialog_choose_files_decrypt(),
    f.dialog_choose_files_decrypt(),
    "dialog_choose_files_decrypt"
  );
  assert_ne!(
    e.dialog_filter_gpg_files(),
    f.dialog_filter_gpg_files(),
    "dialog_filter_gpg_files"
  );
  assert_ne!(
    e.dialog_choose_file_sign(),
    f.dialog_choose_file_sign(),
    "dialog_choose_file_sign"
  );
  assert_ne!(
    e.dialog_choose_file_verify(),
    f.dialog_choose_file_verify(),
    "dialog_choose_file_verify"
  );
  assert_ne!(
    e.dialog_choose_sig_file(),
    f.dialog_choose_sig_file(),
    "dialog_choose_sig_file"
  );
  assert_ne!(
    e.dialog_choose_backup_folder(),
    f.dialog_choose_backup_folder(),
    "dialog_choose_backup_folder"
  );

  // Misc
  assert_ne!(
    e.no_file_selected(),
    f.no_file_selected(),
    "no_file_selected"
  );
  assert_ne!(e.loading(), f.loading(), "loading");
  assert_ne!(e.no_keys(), f.no_keys(), "no_keys");

  // Decrypt
  assert_ne!(e.decrypt_title(), f.decrypt_title(), "decrypt_title");
  assert_ne!(
    e.decrypt_add_files(),
    f.decrypt_add_files(),
    "decrypt_add_files"
  );
  assert_ne!(
    e.decrypt_in_progress(),
    f.decrypt_in_progress(),
    "decrypt_in_progress"
  );
  assert_ne!(
    e.decrypt_auto_key_hint(),
    f.decrypt_auto_key_hint(),
    "decrypt_auto_key_hint"
  );
  assert_ne!(
    e.decrypt_drop_hint(),
    f.decrypt_drop_hint(),
    "decrypt_drop_hint"
  );
  assert_ne!(
    e.decrypt_key_available(),
    f.decrypt_key_available(),
    "decrypt_key_available"
  );
  assert_ne!(
    e.decrypt_key_missing(),
    f.decrypt_key_missing(),
    "decrypt_key_missing"
  );
  assert_ne!(
    e.decrypt_key_checking(),
    f.decrypt_key_checking(),
    "decrypt_key_checking"
  );
  assert_ne!(
    e.decrypt_no_key_warning(),
    f.decrypt_no_key_warning(),
    "decrypt_no_key_warning"
  );
  assert_ne!(e.decrypt_about(), f.decrypt_about(), "decrypt_about");
}

// ---------------------------------------------------------------------------
// Test 4 — English strings contain no French characteristic words
// ---------------------------------------------------------------------------
#[test]
fn english_strings_contain_no_french_words() {
  let s = en();

  // These lowercase substrings are characteristic of French UI text.
  // They would never appear legitimately in English UI strings.
  let markers: &[&str] = &[
    "sur ",
    "chiffr",
    "publique",
    "privée",
    "sélection",
    "état",
    "clef",
    "signer un",
    "obten",
    "copier",
    "enregist",
    "depuis",
    "choisir",
    "glisser",
    "aucune",
    "vérif",
    "fingerprint :",
    "nom /",
    "expiree",
    "expire le",
    "génér",
    "identite",
    "sous-clef",
    "inclure",
    "nouvelle",
    "propos",
    "destinat",
  ];

  // Collect all static English strings into a single list for uniform checking.
  let static_strings: &[(&str, &str)] = &[
    ("nav_my_keys", s.nav_my_keys()),
    ("nav_public_keys", s.nav_public_keys()),
    ("nav_import", s.nav_import()),
    ("nav_create_key", s.nav_create_key()),
    ("nav_encrypt", s.nav_encrypt()),
    ("nav_decrypt", s.nav_decrypt()),
    ("nav_sign", s.nav_sign()),
    ("nav_verify", s.nav_verify()),
    ("nav_health", s.nav_health()),
    ("nav_settings", s.nav_settings()),
    ("sidebar_section_keys", s.sidebar_section_keys()),
    ("sidebar_section_operations", s.sidebar_section_operations()),
    ("sidebar_section_tools", s.sidebar_section_tools()),
    ("btn_ok", s.btn_ok()),
    ("btn_cancel", s.btn_cancel()),
    ("btn_confirm", s.btn_confirm()),
    ("btn_back", s.btn_back()),
    ("btn_create", s.btn_create()),
    ("btn_delete", s.btn_delete()),
    ("btn_export", s.btn_export()),
    ("btn_import", s.btn_import()),
    ("btn_copy", s.btn_copy()),
    ("btn_publish", s.btn_publish()),
    ("btn_backup", s.btn_backup()),
    ("btn_migrate", s.btn_migrate()),
    ("btn_renew", s.btn_renew()),
    ("btn_rotate", s.btn_rotate()),
    ("btn_add_subkey", s.btn_add_subkey()),
    ("btn_export_public", s.btn_export_public()),
    ("btn_backup_key", s.btn_backup_key()),
    ("btn_migrate_yubikey", s.btn_migrate_yubikey()),
    ("btn_decrypt", s.btn_decrypt()),
    ("btn_verify", s.btn_verify()),
    ("btn_sign", s.btn_sign()),
    ("btn_encrypt", s.btn_encrypt()),
    ("key_fingerprint", s.key_fingerprint()),
    ("key_created", s.key_created()),
    ("key_expires", s.key_expires()),
    ("key_never_expires", s.key_never_expires()),
    ("key_trust", s.key_trust()),
    ("key_subkeys", s.key_subkeys()),
    ("key_no_subkeys", s.key_no_subkeys()),
    ("trust_undefined", s.trust_undefined()),
    ("trust_marginal", s.trust_marginal()),
    ("trust_full", s.trust_full()),
    ("trust_ultimate", s.trust_ultimate()),
    ("status_key_created", s.status_key_created()),
    ("status_key_deleted", s.status_key_deleted()),
    ("status_key_exported", s.status_key_exported()),
    ("status_key_imported", s.status_key_imported()),
    ("status_published", s.status_published()),
    ("status_publish_failed", s.status_publish_failed()),
    ("status_backup_done", s.status_backup_done()),
    ("status_preferences_saved", s.status_preferences_saved()),
    ("status_key_copied", s.status_key_copied()),
    ("status_link_copied", s.status_link_copied()),
    ("status_card_migrated", s.status_card_migrated()),
    ("status_subkey_renewed", s.status_subkey_renewed()),
    ("status_subkey_rotated", s.status_subkey_rotated()),
    ("status_file_signed", s.status_file_signed()),
    ("status_files_encrypted", s.status_files_encrypted()),
    ("status_trust_updated", s.status_trust_updated()),
    ("status_subkey_created", s.status_subkey_created()),
    (
      "status_published_openpgp_email",
      s.status_published_openpgp_email(),
    ),
    ("status_files_decrypted", s.status_files_decrypted()),
    ("err_gpg_not_found", s.err_gpg_not_found()),
    ("err_invalid_key", s.err_invalid_key()),
    ("err_import_not_pgp", s.err_import_not_pgp()),
    ("err_export_failed", s.err_export_failed()),
    ("err_delete_failed", s.err_delete_failed()),
    ("err_create_failed", s.err_create_failed()),
    ("err_import_failed", s.err_import_failed()),
    ("err_subkey_renew_failed", s.err_subkey_renew_failed()),
    ("err_sign_failed", s.err_sign_failed()),
    ("err_encrypt_failed", s.err_encrypt_failed()),
    ("err_backup_failed", s.err_backup_failed()),
    ("err_upload_failed", s.err_upload_failed()),
    ("err_save_config_failed", s.err_save_config_failed()),
    ("err_trust_failed", s.err_trust_failed()),
    ("err_diagnostic_failed", s.err_diagnostic_failed()),
    ("err_subkey_add_failed", s.err_subkey_add_failed()),
    ("err_republish_failed", s.err_republish_failed()),
    ("err_decrypt_failed", s.err_decrypt_failed()),
    ("err_no_decryptable_file", s.err_no_decryptable_file()),
    ("encrypt_title", s.encrypt_title()),
    ("encrypt_add_files", s.encrypt_add_files()),
    ("encrypt_recipients", s.encrypt_recipients()),
    ("encrypt_no_recipients", s.encrypt_no_recipients()),
    (
      "encrypt_trust_warning_title",
      s.encrypt_trust_warning_title(),
    ),
    ("encrypt_trust_warning_body", s.encrypt_trust_warning_body()),
    ("encrypt_format_binary", s.encrypt_format_binary()),
    ("encrypt_format_armor", s.encrypt_format_armor()),
    ("encrypt_tab_my_keys", s.encrypt_tab_my_keys()),
    ("encrypt_tab_public_keys", s.encrypt_tab_public_keys()),
    ("encrypt_no_keys", s.encrypt_no_keys()),
    ("encrypt_choose_files", s.encrypt_choose_files()),
    ("encrypt_drop_hint", s.encrypt_drop_hint()),
    ("encrypt_format_ascii_desc", s.encrypt_format_ascii_desc()),
    ("encrypt_format_binary_desc", s.encrypt_format_binary_desc()),
    (
      "encrypt_multi_recipient_hint",
      s.encrypt_multi_recipient_hint(),
    ),
    ("encrypt_select_hint", s.encrypt_select_hint()),
    ("encrypt_in_progress", s.encrypt_in_progress()),
    ("sign_title", s.sign_title()),
    ("sign_select_file", s.sign_select_file()),
    ("sign_select_key", s.sign_select_key()),
    ("sign_no_keys", s.sign_no_keys()),
    ("sign_about", s.sign_about()),
    ("verify_title", s.verify_title()),
    ("verify_select_file", s.verify_select_file()),
    ("verify_outcome_valid", s.verify_outcome_valid()),
    ("verify_outcome_bad_sig", s.verify_outcome_bad_sig()),
    ("verify_outcome_unknown_key", s.verify_outcome_unknown_key()),
    ("verify_outcome_expired_key", s.verify_outcome_expired_key()),
    ("verify_outcome_revoked_key", s.verify_outcome_revoked_key()),
    ("verify_no_file", s.verify_no_file()),
    ("verify_sig_auto_hint", s.verify_sig_auto_hint()),
    ("verify_signed_by", s.verify_signed_by()),
    ("verify_signed_on", s.verify_signed_on()),
    ("verify_in_progress", s.verify_in_progress()),
    ("verify_error_prefix", s.verify_error_prefix()),
    ("verify_valid_full_trust", s.verify_valid_full_trust()),
    (
      "verify_valid_marginal_trust",
      s.verify_valid_marginal_trust(),
    ),
    ("verify_valid_no_trust", s.verify_valid_no_trust()),
    (
      "verify_sig_file_placeholder",
      s.verify_sig_file_placeholder(),
    ),
    ("verify_trust_warning", s.verify_trust_warning()),
    ("verify_fingerprint_label", s.verify_fingerprint_label()),
    ("verify_bad_sig_desc", s.verify_bad_sig_desc()),
    ("verify_unknown_key_desc", s.verify_unknown_key_desc()),
    ("verify_expired_key_desc", s.verify_expired_key_desc()),
    ("verify_revoked_key_desc", s.verify_revoked_key_desc()),
    ("verify_about", s.verify_about()),
    ("health_title", s.health_title()),
    ("health_ok", s.health_ok()),
    ("health_warning", s.health_warning()),
    ("health_error", s.health_error()),
    ("health_info", s.health_info()),
    ("health_diagnostics_title", s.health_diagnostics_title()),
    ("health_diagnostics_desc", s.health_diagnostics_desc()),
    ("health_checking", s.health_checking()),
    (
      "health_category_installation",
      s.health_category_installation(),
    ),
    ("health_category_agent", s.health_category_agent()),
    ("health_category_security", s.health_category_security()),
    ("import_title", s.import_title()),
    ("import_tab_file", s.import_tab_file()),
    ("import_tab_url", s.import_tab_url()),
    ("import_tab_keyserver", s.import_tab_keyserver()),
    ("import_tab_paste", s.import_tab_paste()),
    ("import_source_from_file", s.import_source_from_file()),
    ("import_select_source", s.import_select_source()),
    ("import_url_hint", s.import_url_hint()),
    ("import_url_button", s.import_url_button()),
    ("import_keyserver_hint", s.import_keyserver_hint()),
    ("import_keyserver_button", s.import_keyserver_button()),
    ("import_paste_hint", s.import_paste_hint()),
    ("import_paste_button", s.import_paste_button()),
    ("keyserver_openpgp", s.keyserver_openpgp()),
    ("keyserver_ubuntu", s.keyserver_ubuntu()),
    ("keyserver_status_unknown", s.keyserver_status_unknown()),
    ("keyserver_status_published", s.keyserver_status_published()),
    (
      "keyserver_status_not_published",
      s.keyserver_status_not_published(),
    ),
    ("keyserver_badge_published", s.keyserver_badge_published()),
    (
      "keyserver_badge_not_published",
      s.keyserver_badge_not_published(),
    ),
    ("keyserver_badge_checking", s.keyserver_badge_checking()),
    ("keyserver_badge_link_btn", s.keyserver_badge_link_btn()),
    ("settings_title", s.settings_title()),
    ("settings_language", s.settings_language()),
    ("settings_language_english", s.settings_language_english()),
    ("settings_language_french", s.settings_language_french()),
    ("settings_scale_factor", s.settings_scale_factor()),
    ("settings_scale_factor_hint", s.settings_scale_factor_hint()),
    ("settings_theme", s.settings_theme()),
    ("settings_theme_catppuccin", s.settings_theme_catppuccin()),
    ("settings_theme_ussr", s.settings_theme_ussr()),
    ("modal_delete_title", s.modal_delete_title()),
    ("modal_delete_stub_only", s.modal_delete_stub_only()),
    ("modal_delete_stub_body", s.modal_delete_stub_body()),
    ("modal_delete_secret", s.modal_delete_secret()),
    ("modal_delete_secret_body", s.modal_delete_secret_body()),
    ("modal_delete_public", s.modal_delete_public()),
    ("modal_delete_public_body", s.modal_delete_public_body()),
    (
      "modal_migration_irreversible",
      s.modal_migration_irreversible(),
    ),
    (
      "modal_migration_backup_warning",
      s.modal_migration_backup_warning(),
    ),
    ("modal_migration_backup_btn", s.modal_migration_backup_btn()),
    (
      "modal_migration_confirm_btn",
      s.modal_migration_confirm_btn(),
    ),
    ("modal_migration_cancel_btn", s.modal_migration_cancel_btn()),
    (
      "modal_delete_export_first_btn",
      s.modal_delete_export_first_btn(),
    ),
    ("modal_delete_confirm_btn", s.modal_delete_confirm_btn()),
    ("modal_delete_cancel_btn", s.modal_delete_cancel_btn()),
    ("modal_publish_recommended", s.modal_publish_recommended()),
    ("modal_publish_openpgp_desc", s.modal_publish_openpgp_desc()),
    ("modal_publish_ubuntu_desc", s.modal_publish_ubuntu_desc()),
    ("modal_publish_privacy", s.modal_publish_privacy()),
    ("modal_publish_confirm_btn", s.modal_publish_confirm_btn()),
    (
      "modal_publish_select_keyserver",
      s.modal_publish_select_keyserver(),
    ),
    ("key_list_header_name", s.key_list_header_name()),
    ("key_list_header_expires", s.key_list_header_expires()),
    ("key_list_header_status", s.key_list_header_status()),
    ("key_list_select_hint", s.key_list_select_hint()),
    ("key_type_on_card", s.key_type_on_card()),
    ("key_type_public_private", s.key_type_public_private()),
    ("key_type_public_only", s.key_type_public_only()),
    ("subkey_type_signature", s.subkey_type_signature()),
    ("subkey_type_encryption", s.subkey_type_encryption()),
    ("subkey_type_ssh_auth", s.subkey_type_ssh_auth()),
    ("export_menu_save_disk", s.export_menu_save_disk()),
    ("export_menu_copy_clipboard", s.export_menu_copy_clipboard()),
    ("export_menu_paste_link", s.export_menu_paste_link()),
    ("subkey_expiry_1_year", s.subkey_expiry_1_year()),
    ("subkey_expiry_2_years", s.subkey_expiry_2_years()),
    ("subkey_expiry_5_years", s.subkey_expiry_5_years()),
    ("create_key_generating", s.create_key_generating()),
    ("create_key_title", s.create_key_title()),
    ("create_key_subtitle", s.create_key_subtitle()),
    (
      "create_key_section_identity",
      s.create_key_section_identity(),
    ),
    ("create_key_field_name", s.create_key_field_name()),
    ("create_key_field_email", s.create_key_field_email()),
    ("create_key_section_subkeys", s.create_key_section_subkeys()),
    (
      "create_key_section_expiration",
      s.create_key_section_expiration(),
    ),
    ("create_key_include_ssh", s.create_key_include_ssh()),
    ("create_key_about_master", s.create_key_about_master()),
    ("create_key_hint_expiry", s.create_key_hint_expiry()),
    ("create_key_hint_ssh", s.create_key_hint_ssh()),
    ("create_key_hint_master", s.create_key_hint_master()),
    (
      "dialog_choose_files_encrypt",
      s.dialog_choose_files_encrypt(),
    ),
    (
      "dialog_choose_files_decrypt",
      s.dialog_choose_files_decrypt(),
    ),
    ("dialog_filter_gpg_files", s.dialog_filter_gpg_files()),
    ("dialog_choose_file_sign", s.dialog_choose_file_sign()),
    ("dialog_choose_file_verify", s.dialog_choose_file_verify()),
    ("dialog_choose_sig_file", s.dialog_choose_sig_file()),
    (
      "dialog_choose_backup_folder",
      s.dialog_choose_backup_folder(),
    ),
    ("no_file_selected", s.no_file_selected()),
    ("loading", s.loading()),
    ("no_keys", s.no_keys()),
    ("decrypt_title", s.decrypt_title()),
    ("decrypt_add_files", s.decrypt_add_files()),
    ("decrypt_in_progress", s.decrypt_in_progress()),
    ("decrypt_auto_key_hint", s.decrypt_auto_key_hint()),
    ("decrypt_drop_hint", s.decrypt_drop_hint()),
    ("decrypt_key_available", s.decrypt_key_available()),
    ("decrypt_key_missing", s.decrypt_key_missing()),
    ("decrypt_key_checking", s.decrypt_key_checking()),
    ("decrypt_no_key_warning", s.decrypt_no_key_warning()),
    ("decrypt_about", s.decrypt_about()),
  ];

  // Also check the two String-returning methods with a neutral argument.
  let dynamic_strings: &[(&str, String)] = &[
    ("key_list_error", s.key_list_error("test")),
    (
      "verify_sig_auto_hint_with_name",
      s.verify_sig_auto_hint_with_name("test.sig"),
    ),
  ];

  for (name, value) in static_strings {
    let lower = value.to_lowercase();
    for marker in markers {
      assert!(
        !lower.contains(marker),
        "English string '{name}' contains French marker '{marker}': {value:?}"
      );
    }
  }

  for (name, value) in dynamic_strings {
    let lower = value.to_lowercase();
    for marker in markers {
      assert!(
        !lower.contains(marker),
        "English string '{name}' contains French marker '{marker}': {value:?}"
      );
    }
  }
}
