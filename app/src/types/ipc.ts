export type TrustLevel = 'undefined' | 'marginal' | 'full' | 'ultimate';

export type CheckStatus = 'ok' | 'info' | 'warning' | 'error';

export interface HealthCheck {
  category: string;
  name: string;
  status: CheckStatus;
  current_value: string | null;
  explanation: string;
  fix: string | null;
}

export type DecryptStatus = 'can_decrypt' | 'no_key' | 'checking' | 'unknown';

export type VerifyOutcome =
  | 'valid'
  | 'bad_sig'
  | 'unknown_key'
  | 'expired_key'
  | 'revoked_key'
  | { error: string };

export interface SubkeyInfo {
  fingerprint: string;
  key_id: string;
  algo: string;
  usage: string;
  expires: string | null;
}

export interface KeyInfo {
  fingerprint: string;
  key_id: string;
  name: string;
  email: string;
  algo: string;
  created: string;
  expires: string | null;
  has_secret: boolean;
  on_card: boolean;
  card_serial: string | null;
  subkeys: SubkeyInfo[];
  trust: TrustLevel;
}

export interface VerifyResult {
  outcome: VerifyOutcome;
  signer_name: string | null;
  signer_fp: string | null;
  signed_at: string | null;
  detail: string;
  signer_trust: TrustLevel;
}

export interface CardInfo {
  serial: string;
  sig_fp: string | null;
  enc_fp: string | null;
  auth_fp: string | null;
}

// ── Chat ──────────────────────────────────────────────────────────

export interface RoomParticipant {
  fp: string;
  joined_at: string;
}

export interface Room {
  id: string;
  name: string;
  relay: string;
  my_fp: string;
  created_at: string;
  participants: RoomParticipant[];
}

export interface ChatMessage {
  msg_id: string;
  sender_fp: string;
  content: string;
  ts: number;
  room_id: string;
}
