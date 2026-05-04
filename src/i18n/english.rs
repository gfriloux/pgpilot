use super::Strings;

pub struct EnglishStrings;

impl Strings for EnglishStrings {
  fn nav_my_keys(&self) -> &'static str {
    "My Keys"
  }
  fn nav_public_keys(&self) -> &'static str {
    "Public Keys"
  }
  fn nav_import(&self) -> &'static str {
    "Import"
  }
  fn nav_create_key(&self) -> &'static str {
    "Create Key"
  }
  fn nav_encrypt(&self) -> &'static str {
    "Encrypt"
  }
  fn nav_decrypt(&self) -> &'static str {
    "Decrypt"
  }
  fn nav_sign(&self) -> &'static str {
    "Sign"
  }
  fn nav_verify(&self) -> &'static str {
    "Verify"
  }
  fn nav_health(&self) -> &'static str {
    "Diagnostics"
  }
  fn nav_settings(&self) -> &'static str {
    "Settings"
  }
  fn sidebar_section_keys(&self) -> &'static str {
    "KEYS"
  }
  fn sidebar_section_operations(&self) -> &'static str {
    "OPERATIONS"
  }
  fn sidebar_section_tools(&self) -> &'static str {
    "TOOLS"
  }

  fn btn_ok(&self) -> &'static str {
    "OK"
  }
  fn btn_cancel(&self) -> &'static str {
    "Cancel"
  }
  fn btn_confirm(&self) -> &'static str {
    "Confirm"
  }
  fn btn_back(&self) -> &'static str {
    "Back"
  }
  fn btn_create(&self) -> &'static str {
    "Create"
  }
  fn btn_delete(&self) -> &'static str {
    "Delete"
  }
  fn btn_export(&self) -> &'static str {
    "Export"
  }
  fn btn_import(&self) -> &'static str {
    "Import"
  }
  fn btn_copy(&self) -> &'static str {
    "Copy"
  }
  fn btn_publish(&self) -> &'static str {
    "Publish"
  }
  fn btn_backup(&self) -> &'static str {
    "Backup"
  }
  fn btn_migrate(&self) -> &'static str {
    "Migrate to card"
  }
  fn btn_renew(&self) -> &'static str {
    "Renew"
  }
  fn btn_rotate(&self) -> &'static str {
    "Rotate"
  }
  fn btn_add_subkey(&self) -> &'static str {
    "Add subkey"
  }

  fn key_fingerprint(&self) -> &'static str {
    "Fingerprint"
  }
  fn key_created(&self) -> &'static str {
    "Created"
  }
  fn key_expires(&self) -> &'static str {
    "Expires"
  }
  fn key_never_expires(&self) -> &'static str {
    "Never expires"
  }
  fn key_trust(&self) -> &'static str {
    "Trust"
  }
  fn key_subkeys(&self) -> &'static str {
    "Subkeys"
  }
  fn key_no_subkeys(&self) -> &'static str {
    "No subkeys"
  }

  fn trust_undefined(&self) -> &'static str {
    "Undefined"
  }
  fn trust_marginal(&self) -> &'static str {
    "Marginal"
  }
  fn trust_full(&self) -> &'static str {
    "Full"
  }
  fn trust_ultimate(&self) -> &'static str {
    "Ultimate"
  }

  fn status_key_created(&self) -> &'static str {
    "Key created"
  }
  fn status_key_deleted(&self) -> &'static str {
    "Key deleted"
  }
  fn status_key_exported(&self) -> &'static str {
    "Key exported"
  }
  fn status_key_imported(&self) -> &'static str {
    "Key imported"
  }
  fn status_published(&self) -> &'static str {
    "Published to keyserver"
  }
  fn status_publish_failed(&self) -> &'static str {
    "Publish failed"
  }
  fn status_backup_done(&self) -> &'static str {
    "Backup complete"
  }
  fn status_preferences_saved(&self) -> &'static str {
    "Preferences saved"
  }

  fn err_gpg_not_found(&self) -> &'static str {
    "gpg not found"
  }
  fn err_invalid_key(&self) -> &'static str {
    "Invalid key"
  }
  fn err_import_not_pgp(&self) -> &'static str {
    "Content does not contain a PGP key"
  }
  fn err_export_failed(&self) -> &'static str {
    "Export failed"
  }

  fn encrypt_title(&self) -> &'static str {
    "Encrypt files"
  }
  fn encrypt_add_files(&self) -> &'static str {
    "Add files"
  }
  fn encrypt_recipients(&self) -> &'static str {
    "Recipients"
  }
  fn encrypt_no_recipients(&self) -> &'static str {
    "No recipients selected"
  }
  fn encrypt_trust_warning_title(&self) -> &'static str {
    "Untrusted recipients"
  }
  fn encrypt_trust_warning_body(&self) -> &'static str {
    "Some recipients have insufficient trust level. Encrypt anyway?"
  }
  fn encrypt_format_binary(&self) -> &'static str {
    ".gpg (binary)"
  }
  fn encrypt_format_armor(&self) -> &'static str {
    ".asc (armored)"
  }

  fn sign_title(&self) -> &'static str {
    "Sign file"
  }
  fn sign_select_file(&self) -> &'static str {
    "Select file to sign"
  }
  fn sign_select_key(&self) -> &'static str {
    "Signing key"
  }

  fn verify_title(&self) -> &'static str {
    "Verify signature"
  }
  fn verify_select_file(&self) -> &'static str {
    "Select file to verify"
  }
  fn verify_outcome_valid(&self) -> &'static str {
    "Valid signature"
  }
  fn verify_outcome_bad_sig(&self) -> &'static str {
    "Bad signature"
  }
  fn verify_outcome_unknown_key(&self) -> &'static str {
    "Unknown key"
  }
  fn verify_outcome_expired_key(&self) -> &'static str {
    "Expired key"
  }
  fn verify_outcome_revoked_key(&self) -> &'static str {
    "Revoked key"
  }

  fn health_title(&self) -> &'static str {
    "Diagnostics"
  }
  fn health_ok(&self) -> &'static str {
    "OK"
  }
  fn health_warning(&self) -> &'static str {
    "Warning"
  }
  fn health_error(&self) -> &'static str {
    "Error"
  }
  fn health_info(&self) -> &'static str {
    "Info"
  }

  fn import_title(&self) -> &'static str {
    "Import key"
  }
  fn import_tab_file(&self) -> &'static str {
    "From file"
  }
  fn import_tab_url(&self) -> &'static str {
    "From URL"
  }
  fn import_tab_keyserver(&self) -> &'static str {
    "Keyserver"
  }
  fn import_tab_paste(&self) -> &'static str {
    "Paste"
  }

  fn keyserver_openpgp(&self) -> &'static str {
    "keys.openpgp.org"
  }
  fn keyserver_ubuntu(&self) -> &'static str {
    "keyserver.ubuntu.com"
  }
  fn keyserver_status_unknown(&self) -> &'static str {
    "Unknown"
  }
  fn keyserver_status_published(&self) -> &'static str {
    "Published"
  }
  fn keyserver_status_not_published(&self) -> &'static str {
    "Not published"
  }

  fn settings_title(&self) -> &'static str {
    "Settings"
  }
  fn settings_language(&self) -> &'static str {
    "Language"
  }
  fn settings_language_english(&self) -> &'static str {
    "English"
  }
  fn settings_language_french(&self) -> &'static str {
    "Francais"
  }

  fn health_diagnostics_title(&self) -> &'static str {
    "GPG Diagnostics"
  }
  fn health_diagnostics_desc(&self) -> &'static str {
    "Status of your GnuPG installation and configuration."
  }
  fn health_checking(&self) -> &'static str {
    "Checking…"
  }

  fn status_key_copied(&self) -> &'static str {
    "Key copied to clipboard"
  }
  fn status_link_copied(&self) -> &'static str {
    "Link copied to clipboard"
  }
  fn status_card_migrated(&self) -> &'static str {
    "Migration complete"
  }
  fn status_subkey_renewed(&self) -> &'static str {
    "Subkey renewed"
  }
  fn status_subkey_rotated(&self) -> &'static str {
    "Subkey rotated"
  }
  fn status_file_signed(&self) -> &'static str {
    "File signed"
  }
  fn status_files_encrypted(&self) -> &'static str {
    "Files encrypted"
  }

  fn err_delete_failed(&self) -> &'static str {
    "Delete failed"
  }
  fn err_create_failed(&self) -> &'static str {
    "Creation failed"
  }
  fn err_import_failed(&self) -> &'static str {
    "Import failed"
  }
  fn err_subkey_renew_failed(&self) -> &'static str {
    "Renewal failed"
  }
  fn err_sign_failed(&self) -> &'static str {
    "Signing failed"
  }
  fn err_encrypt_failed(&self) -> &'static str {
    "Encryption failed"
  }
  fn err_backup_failed(&self) -> &'static str {
    "Backup failed"
  }
  fn err_upload_failed(&self) -> &'static str {
    "Upload failed"
  }
  fn err_save_config_failed(&self) -> &'static str {
    "Failed to save preferences"
  }

  fn modal_delete_title(&self) -> &'static str {
    "Delete key?"
  }
  fn modal_delete_stub_only(&self) -> &'static str {
    "Only the local stub will be deleted."
  }
  fn modal_delete_stub_body(&self) -> &'static str {
    "The physical key on the YubiKey will not be affected."
  }
  fn modal_delete_secret(&self) -> &'static str {
    "Irreversible: the secret key will be destroyed."
  }
  fn modal_delete_secret_body(&self) -> &'static str {
    "Without a backup, your encrypted data will be permanently lost."
  }
  fn modal_delete_public(&self) -> &'static str {
    "The public key will be removed from your keyring."
  }
  fn modal_delete_public_body(&self) -> &'static str {
    "This can be undone by re-importing the key."
  }
  fn modal_migration_irreversible(&self) -> &'static str {
    "Irreversible operation: the secret key will be moved to the YubiKey."
  }
  fn modal_migration_backup_warning(&self) -> &'static str {
    "Without a backup, if the YubiKey is lost or destroyed, encrypted data will be unrecoverable."
  }
  fn modal_migration_backup_btn(&self) -> &'static str {
    "Backup first"
  }
  fn modal_migration_confirm_btn(&self) -> &'static str {
    "I have a backup → Continue"
  }
  fn modal_migration_cancel_btn(&self) -> &'static str {
    "Cancel"
  }
  fn modal_delete_export_first_btn(&self) -> &'static str {
    "Export first"
  }
  fn modal_delete_confirm_btn(&self) -> &'static str {
    "Confirm delete"
  }
  fn modal_delete_cancel_btn(&self) -> &'static str {
    "Cancel"
  }

  fn keyserver_badge_published(&self) -> &'static str {
    "Published on keys.openpgp.org"
  }
  fn keyserver_badge_not_published(&self) -> &'static str {
    "Not yet published"
  }
  fn keyserver_badge_checking(&self) -> &'static str {
    "Checking keys.openpgp.org…"
  }
  fn keyserver_badge_link_btn(&self) -> &'static str {
    "Link"
  }

  fn btn_export_public(&self) -> &'static str {
    "Export public"
  }
  fn btn_backup_key(&self) -> &'static str {
    "Backup"
  }
  fn btn_migrate_yubikey(&self) -> &'static str {
    "Migrate to YubiKey"
  }

  fn decrypt_title(&self) -> &'static str {
    "Decrypt files"
  }
  fn decrypt_add_files(&self) -> &'static str {
    "Add files"
  }
  fn decrypt_in_progress(&self) -> &'static str {
    "Decryption in progress..."
  }
  fn btn_decrypt(&self) -> &'static str {
    "Decrypt"
  }

  fn verify_no_file(&self) -> &'static str {
    "No file selected"
  }
  fn verify_sig_auto_hint(&self) -> &'static str {
    "Optional - automatically searches for <file>.sig"
  }
  fn verify_signed_by(&self) -> &'static str {
    "Signed by"
  }
  fn verify_signed_on(&self) -> &'static str {
    "on"
  }
  fn verify_in_progress(&self) -> &'static str {
    "Verifying..."
  }
  fn verify_error_prefix(&self) -> &'static str {
    "Error"
  }
  fn btn_verify(&self) -> &'static str {
    "Verify"
  }
  fn btn_sign(&self) -> &'static str {
    "Sign"
  }
  fn no_file_selected(&self) -> &'static str {
    "No file selected"
  }
  fn loading(&self) -> &'static str {
    "Loading..."
  }
  fn no_keys(&self) -> &'static str {
    "No keys"
  }

  fn modal_publish_recommended(&self) -> &'static str {
    "Recommended - respects GDPR."
  }
  fn modal_publish_openpgp_desc(&self) -> &'static str {
    "A verification email will be sent to your identity email to make it visible in searches."
  }
  fn modal_publish_ubuntu_desc(&self) -> &'static str {
    "An open keyserver. Your key and identity will be publicly visible."
  }
  fn modal_publish_privacy(&self) -> &'static str {
    "Privacy notice"
  }
  fn modal_publish_confirm_btn(&self) -> &'static str {
    "Publish"
  }
  fn modal_publish_select_keyserver(&self) -> &'static str {
    "Select keyserver"
  }

  fn verify_valid_full_trust(&self) -> &'static str {
    "Valid signature"
  }
  fn verify_valid_marginal_trust(&self) -> &'static str {
    "Valid (marginal trust)"
  }
  fn verify_valid_no_trust(&self) -> &'static str {
    "Correct signature - identity not verified"
  }

  fn status_trust_updated(&self) -> &'static str {
    "Trust level updated"
  }
  fn err_trust_failed(&self) -> &'static str {
    "Failed to update trust"
  }

  fn err_diagnostic_failed(&self) -> &'static str {
    "Diagnostic error"
  }

  fn status_subkey_created(&self) -> &'static str {
    "Subkey created"
  }
  fn err_subkey_add_failed(&self) -> &'static str {
    "Failed to add subkey"
  }

  fn status_published_openpgp_email(&self) -> &'static str {
    "Key published. Check your email to validate publication on keys.openpgp.org."
  }
  fn err_republish_failed(&self) -> &'static str {
    "Republish failed"
  }

  fn btn_encrypt(&self) -> &'static str {
    "Encrypt"
  }
  fn encrypt_in_progress(&self) -> &'static str {
    "Encrypting..."
  }

  fn status_files_decrypted(&self) -> &'static str {
    "Files decrypted"
  }
  fn err_decrypt_failed(&self) -> &'static str {
    "Decryption failed"
  }
  fn err_no_decryptable_file(&self) -> &'static str {
    "No decryptable file selected."
  }
}
