import type { KeyInfo } from '../types/ipc';

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

const MOCK_KEYS: KeyInfo[] = [
  {
    fingerprint: 'A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2',
    key_id: 'A1B2C3D4E5F6A1B2',
    name: 'Alice Dupont',
    email: 'alice@example.com',
    algo: 'ed25519',
    created: '2023-01-15',
    expires: null,
    has_secret: true,
    on_card: false,
    card_serial: null,
    trust: 'ultimate',
    subkeys: [
      {
        fingerprint: 'B1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6B1B2',
        key_id: 'B1B2C3D4E5F6B1B2',
        algo: 'ed25519',
        usage: 'S',
        expires: '2025-01-15',
      },
      {
        fingerprint: 'C1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6C1B2',
        key_id: 'C1B2C3D4E5F6C1B2',
        algo: 'cv25519',
        usage: 'E',
        expires: '2025-01-15',
      },
      {
        fingerprint: 'D1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6D1B2',
        key_id: 'D1B2C3D4E5F6D1B2',
        algo: 'ed25519',
        usage: 'A',
        expires: '2025-01-15',
      },
    ],
  },
  {
    fingerprint: '1234567890ABCDEF1234567890ABCDEF12345678',
    key_id: '1234567890ABCDEF',
    name: 'Bob Martin',
    email: 'bob@example.org',
    algo: 'rsa4096',
    created: '2021-06-01',
    expires: '2026-06-01',
    has_secret: false,
    on_card: false,
    card_serial: null,
    trust: 'marginal',
    subkeys: [
      {
        fingerprint: '2234567890ABCDEF1234567890ABCDEF22345678',
        key_id: '2234567890ABCDEF',
        algo: 'rsa4096',
        usage: 'S',
        expires: '2026-06-01',
      },
      {
        fingerprint: '3234567890ABCDEF1234567890ABCDEF32345678',
        key_id: '3234567890ABCDEF',
        algo: 'rsa4096',
        usage: 'E',
        expires: '2026-06-01',
      },
    ],
  },
  {
    fingerprint: 'FEDCBA9876543210FEDCBA9876543210FEDCBA98',
    key_id: 'FEDCBA9876543210',
    name: 'Charlie Moreau',
    email: 'charlie@secure.io',
    algo: 'ed25519',
    created: '2024-03-10',
    expires: null,
    has_secret: true,
    on_card: true,
    card_serial: 'D2760001240100000006123456780000',
    trust: 'full',
    subkeys: [
      {
        fingerprint: 'EEDCBA9876543210FEDCBA9876543210EEDCBA98',
        key_id: 'EEDCBA9876543210',
        algo: 'ed25519',
        usage: 'S',
        expires: null,
      },
      {
        fingerprint: 'FEDCBA9876543210FEDCBA9876543210FEDCBA99',
        key_id: 'FEDCBA9876543211',
        algo: 'cv25519',
        usage: 'E',
        expires: null,
      },
    ],
  },
];

const MOCK_ARMORED = `-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEY2test1BByweqCEEP4test2aBCEtest3DEFtest4GHtest5IJtest6KL
test7MNtest8OPtest9QRtestABtestCDtestEFtestGHtestIJtestKL==
=test
-----END PGP PUBLIC KEY BLOCK-----`;

export async function invoke<T>(
  cmd: string,
  _args: Record<string, unknown> = {},
): Promise<T> {
  await delay(60);
  switch (cmd) {
    case 'get_version':
      return '0.8.0-mock' as T;

    case 'list_keys':
      return MOCK_KEYS as T;

    case 'get_card_info':
    case 'card_status':
      return null as T;

    // Export
    case 'export_public_key_armored':
      return MOCK_ARMORED as T;

    case 'export_public_key_to_file':
      return undefined as T;

    // Backup
    case 'backup_key':
      return ['ABCDEF12_secret.asc', 'ABCDEF12_revocation.rev'] as T;

    // Delete
    case 'delete_key':
      return undefined as T;

    // Keyserver
    case 'publish_key':
      return 'https://keys.openpgp.org/vks/v1/upload' as T;

    case 'check_keyserver':
      return false as T;

    // Subkeys
    case 'renew_subkey_cmd':
    case 'rotate_subkey_cmd':
    case 'add_subkey_cmd':
      return undefined as T;

    // Trust
    case 'set_key_trust':
      return undefined as T;

    // Import
    case 'import_key_text':
    case 'import_key_url':
    case 'import_key_keyserver':
    case 'import_key_file':
      return undefined as T;

    // YubiKey
    case 'move_to_card':
      return undefined as T;

    // Create
    case 'create_key':
      return 'DEADBEEF1234567890ABCDEF1234567890DEADBE' as T;

    // Encrypt
    case 'encrypt_files_cmd':
      return ['/tmp/file.txt.gpg'] as T;

    // Sign
    case 'sign_file_cmd':
      return '/tmp/file.txt.sig' as T;

    // Verify
    case 'verify_signature_cmd':
      return {
        outcome: 'valid',
        signer_name: 'Alice Dupont',
        signer_fp: 'A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2',
        signed_at: '2024-01-15',
        detail: 'Good signature',
        signer_trust: 'ultimate',
      } as T;

    // Health
    case 'run_health_checks_cmd':
      return [
        {
          category: 'Installation',
          name: 'GPG installed',
          status: 'ok',
          current_value: 'gpg 2.4.3',
          explanation: 'GnuPG is installed and accessible.',
          fix: null,
        },
        {
          category: 'Agent GPG',
          name: 'GPG agent running',
          status: 'ok',
          current_value: null,
          explanation: 'The gpg-agent daemon is running.',
          fix: null,
        },
        {
          category: 'Sécurité',
          name: 'Keybox permissions',
          status: 'warning',
          current_value: '0644',
          explanation: 'Keybox should not be world-readable.',
          fix: 'chmod 600 ~/.gnupg/pubring.kbx',
        },
      ] as T;

    // Decrypt
    case 'decrypt_files_cmd':
      return ['Decrypted: /tmp/file.txt'] as T;

    case 'inspect_decrypt_cmd':
      return 'can_decrypt' as T;

    // Chat
    case 'chat_list_rooms':
      return [] as T;

    case 'chat_create_room':
      return {
        id: 'mock-room-1',
        name: (_args as { name: string }).name,
        relay: 'mqtts://broker.hivemq.com:8883',
        my_fp: 'A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2',
        created_at: new Date().toISOString(),
        participants: [
          {
            fp: 'A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2',
            joined_at: new Date().toISOString(),
          },
        ],
      } as T;

    case 'chat_delete_room':
    case 'chat_add_participant':
    case 'chat_start':
    case 'chat_stop':
      return undefined as T;

    case 'chat_send':
      return ('mock-msg-id-' + Math.random().toString(36).slice(2)) as T;

    case 'chat_generate_join_code':
      return 'pgpilot:join:bW9ja19qb2luX2NvZGU' as T;

    case 'chat_join_room':
      return {
        id: 'joined-room-' + Math.random().toString(36).slice(2),
        name: 'Joined Room',
        relay: 'mqtts://broker.hivemq.com:8883',
        my_fp: 'A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2',
        created_at: new Date().toISOString(),
        participants: [],
      } as T;

    case 'check_revocation_cert':
      return '~/.gnupg/openpgp-revocs.d/A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2.rev' as T;

    case 'generate_revocation_cert_cmd':
      return '~/.gnupg/openpgp-revocs.d/A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2.rev' as T;

    default:
      throw new Error(`mock-tauri: unknown command "${cmd}"`);
  }
}
