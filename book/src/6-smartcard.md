# YubiKey / Smartcard

Store your subkeys on hardware for enhanced security.

## Why use a hardware key?

A **hardware security key** (e.g., **YubiKey**) keeps your private keys isolated from your computer:

- Private keys never leave the hardware
- Compromised computer ≠ compromised keys
- Requires physical key present to sign/decrypt
- You still use `gpg` — pgpilot is your GUI

**Trade-off**: Slower (requires physical interaction); more secure.

---

## Supported Hardware

pgpilot supports **OpenPGP-compatible** smartcards via `gpg`:

- **YubiKey 5 / 5C / 5 Nano** (most popular)
- **YubiKey 4** (older, still supported)
- **Nitrokey** (open-source alternative)
- **Gemalto IDBridge** (enterprise)
- Any **OpenPGP Card 3.x** compatible device

### Requirements

- Hardware key plugged in (USB-A, USB-C, or NFC depending on model)
- GnuPG ≥ 2.2 (with smart card support)
- `scdaemon` running (usually automatic; see [Troubleshooting](8-troubleshooting.md))

---

## Checking Card Status

pgpilot auto-detects connected hardware:

1. Select a key in **My Keys**
2. In the detail panel, look for:
   - **Card badge**: Shows card serial number if a key is migrated to this card
   - **Card icon**: Appears next to key if stored on hardware

If no card is detected:
- Check that the key is plugged in
- See [Troubleshooting](8-troubleshooting.md) for scdaemon issues

---

## Migrating a Subkey to Hardware

Move an individual subkey to the card (e.g., move only the Encryption subkey).

### Before you start

1. **Back up your key**: Click **Backup** first (see [Key Management](3-key-management.md))
   - If migration fails, you can restore from backup
2. **Initialize the card** (first-time only):
   ```bash
   gpg --card-edit
   admin
   passwd  # set admin PIN (default: 12345678)
   quit
   ```

### Migrate in pgpilot

1. Select your key
2. Click **Migrate to YubiKey**
3. A modal asks: "Which subkey?"
   - Choose **Sign**, **Encrypt**, or **Auth**
4. Confirm: "This action cannot be undone"
5. pgpilot calls `gpg --edit-key --command-fd 0 --status-fd 2 <fp>`:
   - Sends `key N` (select subkey)
   - Sends `keytocard`
   - Prompts for card admin PIN (default: `12345678`)
6. After successful migration:
   - Card icon appears in the key detail
   - Subkey now lives on the card
   - Private key removed from your computer

### What happens to the original key?

After migration, your computer has:
- **Secret key stub** — pointer to the card (gpg recognizes which card)
- **Public key** — unchanged, still in `~/.gnupg/pubring.gpg`

When you sign/decrypt, gpg sends commands to the card via USB.

---

## Using a Card-Based Key

Once a subkey is on the card, signing and decryption work the same in pgpilot:

1. **Signing**: Click **Sign**, choose signer key, enter PIN (via `pinentry`)
2. **Decryption**: Click **Decrypt**, decryption happens on card, enter PIN
3. **Publishing**: **Publish** still works (sends stub + public key to keyserver)

From pgpilot's perspective, it's transparent — you don't need to know if a key is on card or on disk.

**Difference you'll notice**: Operations require physical presence (key must be plugged in) and PIN entry.

---

## SSH Authentication with Auth Subkey

If you created your key with SSH Auth subkey enabled, you can use it for SSH:

### Setup (one-time)

1. Export your public key in SSH format (currently requires command-line):
   ```bash
   gpg --export-ssh-key <fingerprint> > ~/.ssh/id_pgp.pub
   ```

2. Add to authorized_keys on your servers:
   ```bash
   cat ~/.ssh/id_pgp.pub >> ~/.ssh/authorized_keys  # on remote server
   ```

3. Configure SSH to use the GPG key:
   ```bash
   export SSH_AUTH_SOCK="/run/user/$(id -u)/gnupg/S.gpg-agent.ssh"
   ssh -i ~/.ssh/id_pgp.pub username@server.com
   ```

### Usage

- SSH now uses your GPG key instead of separate SSH keys
- If Auth subkey is on a YubiKey, SSH also requires the card + PIN
- Centralize key management: one master key, subkeys for different purposes

**Note**: This requires `gpg-agent` SSH socket, not pgpilot UI. pgpilot may add SSH key export in a future release.

---

## Rotating a Card-Based Subkey

If a card-based subkey is compromised or expired:

1. Rotate as normal: Select key, click subkey **Replace**
2. pgpilot creates a new subkey on **disk**
3. Old subkey on card is revoked
4. You can now:
   - Keep the new subkey on disk, or
   - Migrate it to card (see "Migrating a Subkey to Hardware" above)

---

## Multiple Keys / Multiple Cards

You can have multiple hardware keys:

1. Plug in a different card
2. In pgpilot, select a different key
3. Migrate different subkeys to different cards
4. Cards are identified by serial number (shown in detail panel)

pgpilot keeps track of which card holds which key.

---

## Resetting a Card

To erase and reuse a card:

```bash
gpg --card-edit
admin
factoryreset  # WARNING: erases all keys on card
quit
```

Then reinitialize:
```bash
gpg --card-edit
admin
passwd
quit
```

**⚠️ WARNING**: `factoryreset` is irreversible. Ensure you have backups before doing this.

---

## Troubleshooting

**"Card not detected"**
- Plug in the key (USB-A, USB-C, or NFC)
- Check: `gpg --card-status`
- If error, see [Troubleshooting](8-troubleshooting.md) — scdaemon section

**"Migration failed"**
- Wrong admin PIN entered (default: `12345678`)
- Card full (max 3 subkeys per OpenPGP Card 3.x)
- Card not detected mid-operation

If migration partially fails, restore from backup:
```bash
gpg --import <backup-secret.asc>
```

**"Sign/decrypt hangs on card"**
- Card unplugged
- Card unresponsive (try unplugging and re-plugging)
- See [Troubleshooting](8-troubleshooting.md) — pinentry/scdaemon issues

**"Two keys on same card"**
- OpenPGP Card 3.x holds **3 subkeys max**: Signature, Encryption, Authentication
- You cannot put two master keys' subkeys on one card
- Use separate cards for separate identities

---

## Best practices

1. **Back up before migration**
   - Always export secret key backup (see [Key Management](3-key-management.md))
   - Keep backups offline (USB in a safe)

2. **Use strong card PIN**
   - Default: `12345678` (extremely weak)
   - Change immediately: `gpg --card-edit → admin → passwd`
   - Recommended: 8+ mixed characters

3. **Keep card with you**
   - Can't sign/decrypt without it
   - Misplaced card = need to rotate and revoke old keys

4. **Test recovery**
   - Periodically restore from backup and verify it works
   - Don't discover backup corruption during an emergency

5. **Document your setup**
   - Note which card holds which key's subkeys
   - Store card serial numbers with your backups

---

## Next steps

- Learn about encryption & signing — see [File Operations](5-file-operations.md)
- Publish keys with migrated subkeys — see [Keyserver & Sharing](4-keyserver.md)
- Diagnose issues — see [Troubleshooting](8-troubleshooting.md)
