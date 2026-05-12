import { invoke } from '@tauri-apps/api/core';
import type { KeyInfo, CardInfo, VerifyResult, HealthCheck, DecryptStatus } from '../types/ipc';

export const listKeys = (): Promise<KeyInfo[]> => invoke('list_keys');

export const getVersion = (): Promise<string> => invoke('get_version');

export const getCardInfo = (): Promise<CardInfo | null> => invoke('get_card_info');

export const verifySignature = (
  file: string,
  sigFile: string | null,
): Promise<VerifyResult> => invoke('verify_signature', { file, sigFile });

// ── Export ────────────────────────────────────────────────────────
export const exportPublicKeyArmored = (fp: string): Promise<string> =>
  invoke('export_public_key_armored', { fp });

export const exportPublicKeyToFile = (fp: string, destPath: string): Promise<void> =>
  invoke('export_public_key_to_file', { fp, destPath });

// ── Backup ────────────────────────────────────────────────────────
export const backupKey = (fp: string, destDir: string): Promise<string[]> =>
  invoke('backup_key', { fp, destDir });

// ── Delete ────────────────────────────────────────────────────────
export const deleteKey = (fp: string, hasSecret: boolean): Promise<void> =>
  invoke('delete_key', { fp, hasSecret });

// ── Keyserver ────────────────────────────────────────────────────
export const publishKey = (fp: string, keyserverUrl: string): Promise<string> =>
  invoke('publish_key', { fp, keyserverUrl });

export const checkKeyserver = (fp: string): Promise<boolean> =>
  invoke('check_keyserver', { fp });

// ── Subkeys ───────────────────────────────────────────────────────
export const renewSubkeyCmd = (
  keyFp: string,
  subkeyFp: string,
  expiryDays: number,
): Promise<void> =>
  invoke('renew_subkey_cmd', { keyFp, subkeyFp, expiryDays });

export const rotateSubkeyCmd = (
  keyFp: string,
  subkeyFp: string,
  usage: string,
  expiryDays: number,
): Promise<void> =>
  invoke('rotate_subkey_cmd', { keyFp, subkeyFp, usage, expiryDays });

export const addSubkeyCmd = (
  masterFp: string,
  usage: string,
  expiryDays: number,
): Promise<void> =>
  invoke('add_subkey_cmd', { masterFp, usage, expiryDays });

// ── Trust ─────────────────────────────────────────────────────────
export const setKeyTrust = (fp: string, trust: string): Promise<void> =>
  invoke('set_key_trust', { fp, trust });

// ── Import ────────────────────────────────────────────────────────
export const importKeyText = (content: string): Promise<void> =>
  invoke('import_key_text', { content });

export const importKeyUrl = (url: string): Promise<void> =>
  invoke('import_key_url', { url });

export const importKeyKeyserver = (
  query: string,
  keyserverUrl: string,
): Promise<void> =>
  invoke('import_key_keyserver', { query, keyserverUrl });

export const importKeyFile = (path: string): Promise<void> =>
  invoke('import_key_file', { path });

// ── YubiKey ───────────────────────────────────────────────────────
export const cardStatus = (): Promise<CardInfo | null> =>
  invoke('card_status');

export const moveToCard = (fp: string): Promise<void> =>
  invoke('move_to_card', { fp });

// ── Create ────────────────────────────────────────────────────────
export const createKey = (name: string, email: string, expiryDays: number): Promise<string> =>
  invoke('create_key', { name, email, expiryDays });

// ── Encrypt ───────────────────────────────────────────────────────
export const encryptFiles = (
  files: string[],
  recipients: string[],
  armor: boolean,
  forceTrust: boolean,
): Promise<string[]> =>
  invoke('encrypt_files_cmd', { files, recipients, armor, forceTrust });

// ── Sign ──────────────────────────────────────────────────────────
export const signFile = (file: string, signerFp: string): Promise<string> =>
  invoke('sign_file_cmd', { file, signerFp });

// ── Verify ────────────────────────────────────────────────────────
export const verifySignatureCmd = (
  file: string,
  sigFile: string | null,
): Promise<VerifyResult> =>
  invoke('verify_signature_cmd', { file, sigFile });

// ── Health ────────────────────────────────────────────────────────
export const runHealthChecks = (): Promise<HealthCheck[]> =>
  invoke('run_health_checks_cmd');

// ── Decrypt ───────────────────────────────────────────────────────
export const decryptFiles = (files: string[]): Promise<string[]> =>
  invoke('decrypt_files_cmd', { files });

export const inspectDecrypt = (file: string): Promise<DecryptStatus> =>
  invoke('inspect_decrypt_cmd', { file });

// ── Revocation certificate ─────────────────────────────────────────
export const checkRevocationCert = (fp: string): Promise<string | null> =>
  invoke('check_revocation_cert', { fp });

export const generateRevocationCert = (fp: string): Promise<string> =>
  invoke('generate_revocation_cert_cmd', { fp });
