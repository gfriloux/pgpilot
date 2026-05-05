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
    "French"
  }
  fn settings_scale_factor(&self) -> &'static str {
    "UI Scale"
  }
  fn settings_scale_factor_hint(&self) -> &'static str {
    "Adjust the interface scale (useful on HiDPI or 1080p screens)"
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

  // key_list.rs
  fn key_list_error(&self, err: &str) -> String {
    format!("Error: {err}")
  }
  fn key_list_header_name(&self) -> &'static str {
    "Name / Email"
  }
  fn key_list_header_expires(&self) -> &'static str {
    "Expires"
  }
  fn key_list_header_status(&self) -> &'static str {
    "Status"
  }
  fn key_list_select_hint(&self) -> &'static str {
    "Select a key to view details."
  }

  // key_detail.rs
  fn key_type_on_card(&self) -> &'static str {
    "On YubiKey"
  }
  fn key_type_public_private(&self) -> &'static str {
    "Public + Private"
  }
  fn key_type_public_only(&self) -> &'static str {
    "Public"
  }
  fn subkey_type_signature(&self) -> &'static str {
    "Signature"
  }
  fn subkey_type_encryption(&self) -> &'static str {
    "Encryption"
  }
  fn subkey_type_ssh_auth(&self) -> &'static str {
    "SSH Auth"
  }
  fn export_menu_save_disk(&self) -> &'static str {
    "Save to disk"
  }
  fn export_menu_copy_clipboard(&self) -> &'static str {
    "Copy to clipboard"
  }
  fn export_menu_paste_link(&self) -> &'static str {
    "Get a public link (paste.rs)"
  }
  fn subkey_expiry_1_year(&self) -> &'static str {
    "1 year"
  }
  fn subkey_expiry_2_years(&self) -> &'static str {
    "2 years"
  }
  fn subkey_expiry_5_years(&self) -> &'static str {
    "5 years"
  }

  // create_key.rs
  fn create_key_generating(&self) -> &'static str {
    "Generating..."
  }
  fn create_key_title(&self) -> &'static str {
    "New PGP Key"
  }
  fn create_key_subtitle(&self) -> &'static str {
    "Generates a master key and its dedicated subkeys."
  }
  fn create_key_section_identity(&self) -> &'static str {
    "Identity"
  }
  fn create_key_field_name(&self) -> &'static str {
    "Name"
  }
  fn create_key_field_email(&self) -> &'static str {
    "Email"
  }
  fn create_key_section_subkeys(&self) -> &'static str {
    "Subkeys"
  }
  fn create_key_section_expiration(&self) -> &'static str {
    "Expiration"
  }
  fn create_key_include_ssh(&self) -> &'static str {
    "Include SSH authentication key"
  }
  fn create_key_about_master(&self) -> &'static str {
    "About the master key"
  }
  fn create_key_hint_expiry(&self) -> &'static str {
    "Subkeys expire automatically. A short duration limits damage in case of compromise — you can renew them before they expire."
  }
  fn create_key_hint_ssh(&self) -> &'static str {
    "Allows you to authenticate on remote servers without a password, using your PGP key as an SSH key."
  }
  fn create_key_hint_master(&self) -> &'static str {
    "The master key defines your long-term PGP identity — it is only used to certify your subkeys. It never expires. Keep it offline with its revocation certificate."
  }

  // encrypt.rs
  fn encrypt_tab_my_keys(&self) -> &'static str {
    "My keys"
  }
  fn encrypt_tab_public_keys(&self) -> &'static str {
    "Public keys"
  }
  fn encrypt_no_keys(&self) -> &'static str {
    "No keys with encryption capability."
  }
  fn encrypt_choose_files(&self) -> &'static str {
    "Choose files..."
  }
  fn encrypt_drop_hint(&self) -> &'static str {
    "Drag files here"
  }
  fn encrypt_format_ascii_desc(&self) -> &'static str {
    "ASCII text — for pasting in an email or message."
  }
  fn encrypt_format_binary_desc(&self) -> &'static str {
    "Compact binary — for attachments and storage."
  }
  fn encrypt_multi_recipient_hint(&self) -> &'static str {
    "Each recipient can decrypt the file independently with their own key. \
     Remember to add yourself to retain access to the encrypted file."
  }
  fn encrypt_select_hint(&self) -> &'static str {
    "Select recipients and files."
  }

  // sign.rs
  fn sign_no_keys(&self) -> &'static str {
    "No private key with signing capability."
  }
  fn sign_about(&self) -> &'static str {
    "Signing a file creates cryptographic proof that you are its author. \
     The original file is not modified — the signature is saved in a separate .sig file."
  }

  // verify.rs
  fn verify_sig_file_placeholder(&self) -> &'static str {
    "Signature file (.sig)..."
  }
  fn verify_trust_warning(&self) -> &'static str {
    "The displayed identity is not verified by your trust network."
  }
  fn verify_fingerprint_label(&self) -> &'static str {
    "Fingerprint:"
  }
  fn verify_bad_sig_desc(&self) -> &'static str {
    "The signature does not match this file. \
     Verify that you have selected the correct file and signature."
  }
  fn verify_unknown_key_desc(&self) -> &'static str {
    "The signer's public key is not in your keyring. \
     Import it to verify the signer's identity."
  }
  fn verify_expired_key_desc(&self) -> &'static str {
    "The signature is mathematically valid, but the signer's key was expired \
     at the time of verification."
  }
  fn verify_revoked_key_desc(&self) -> &'static str {
    "The key that signed this file has been revoked. \
     The signature is no longer considered trustworthy."
  }
  fn verify_about(&self) -> &'static str {
    "Verifying a signature confirms that the file has not been modified and identifies its author."
  }
  fn verify_sig_auto_hint_with_name(&self, auto_name: &str) -> String {
    format!("Optional — will automatically search for {auto_name}")
  }

  // import.rs
  fn import_source_from_file(&self) -> &'static str {
    "From file"
  }
  fn import_select_source(&self) -> &'static str {
    "Choose the source of the key to import."
  }
  fn import_url_hint(&self) -> &'static str {
    "Paste a URL pointing to an armored key (paste.rs, web page, etc.)."
  }
  fn import_url_button(&self) -> &'static str {
    "Import from URL"
  }
  fn import_keyserver_hint(&self) -> &'static str {
    "Full fingerprint (40 hex), long ID (16 hex) or email address."
  }
  fn import_keyserver_button(&self) -> &'static str {
    "Import from keyserver"
  }
  fn import_paste_hint(&self) -> &'static str {
    "Paste the content of an armored PGP key (-----BEGIN PGP...)."
  }
  fn import_paste_button(&self) -> &'static str {
    "Import pasted key"
  }

  // health.rs
  fn health_category_installation(&self) -> &'static str {
    "Installation"
  }
  fn health_category_agent(&self) -> &'static str {
    "GPG Agent"
  }
  fn health_category_security(&self) -> &'static str {
    "Security"
  }

  // decrypt.rs
  fn decrypt_auto_key_hint(&self) -> &'static str {
    "GPG will automatically use your private key. \
     If it is protected by a passphrase, a window will open to ask for it."
  }
  fn decrypt_drop_hint(&self) -> &'static str {
    "Drag .gpg or .asc files here, or use the button below."
  }
  fn decrypt_key_available(&self) -> &'static str {
    "Key available"
  }
  fn decrypt_key_missing(&self) -> &'static str {
    "Key missing"
  }
  fn decrypt_key_checking(&self) -> &'static str {
    "Checking..."
  }
  fn decrypt_no_key_warning(&self) -> &'static str {
    "Some files cannot be decrypted — you do not have the corresponding private key. \
     These files will be skipped."
  }
  fn decrypt_about(&self) -> &'static str {
    "Decrypt files encrypted with GPG."
  }

  // File dialog titles
  fn dialog_choose_files_encrypt(&self) -> &'static str {
    "Choose files to encrypt"
  }
  fn dialog_choose_files_decrypt(&self) -> &'static str {
    "Choose files to decrypt"
  }
  fn dialog_filter_gpg_files(&self) -> &'static str {
    "GPG files"
  }
  fn dialog_choose_file_sign(&self) -> &'static str {
    "Choose a file to sign"
  }
  fn dialog_choose_file_verify(&self) -> &'static str {
    "Choose the file to verify"
  }
  fn dialog_choose_sig_file(&self) -> &'static str {
    "Choose the signature file (.sig)"
  }
  fn dialog_choose_backup_folder(&self) -> &'static str {
    "Choose a backup folder"
  }
}
