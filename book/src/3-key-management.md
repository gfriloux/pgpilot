# Key Management

Detailed workflows for managing GPG keys with pgpilot.

## Understanding Key Structure

pgpilot creates keys with a specific structure optimized for modern cryptography:

- **Primary key** (master cert): ed25519, certification-only (doesn't expire)
- **Sign subkey** (S): ed25519, for signing files and commits
- **Encryption subkey** (E): cv25519, for decryption of messages/files
- **Auth subkey** (A): ed25519 (optional), for SSH authentication

This structure follows best practices: subkeys can be rotated without changing your master key identity.

---

## Navigating the Key List

The **My Keys** view displays your secret keys in a 2-column layout:

**Left panel (320px fixed)**:
- Master key list with name, email, creation date, and expiry badge
- Trust level indicator (colored badge: Undefined / Marginal / Full)
- Key icons (S E A) for subkey types present

**Right panel (fills remaining space)**:
- Detailed key information
- Subkey list (read-only)
- Action buttons (Export, Backup, Migrate, Publish, Delete)
- Trust level picker

Click a key in the list to select it and view details on the right.

**Theme note**: In the USSR theme, status indicators (keyserver published, YubiKey, trust level) appear as circular SVG badges instead of text icons, providing a distinctive Soviet aesthetic while maintaining clarity.

---

## Managing Trust Levels

**Trust levels** tell pgpilot how much you trust a key. When encrypting, only keys at sufficient trust (Full or Ultimate) are recommended without warning.

### View trust level

1. Select a key in the list (right panel shows details)
2. Find the **Trust** badge — shows current level
   - **Undefined** (gray) — you haven't verified this key
   - **Marginal** (amber) — you've partially verified the owner
   - **Full** (green) — you've fully verified the owner
   - **Ultimate** (green, rare) — you own this key

### Change trust level

1. Select a key in the list
2. In the detail panel, click the **Trust** badge
3. A picker appears with three options: Undefined, Marginal, Full
4. Click your choice
5. Trust is saved immediately (calls `gpg --import-ownertrust`)

**Best practice**: Set trust only after meeting someone in person and verifying their fingerprint by hand.

---

## Working with Subkeys

Subkeys are the workhorses of your key. They handle encryption and signing, while the primary key stays secure.

### View subkeys

Select a key in the list. The right panel shows a **Subkeys** section:

```
[S] Sign             Created: 2024-01-15   Expires: Never
    Algo: ed25519  ID: ABC123...

[E] Encrypt          Created: 2024-01-15   Expires: Never
    Algo: cv25519  ID: DEF456...

[A] Auth SSH         Created: 2024-01-20   Expires: 2026-01-20
    Algo: ed25519  ID: GHI789...
```

Legend: **[S]** = Sign, **[E]** = Encrypt, **[A]** = Authentication.

### Add a subkey

If you're missing a subkey (e.g., you created a key without SSH Auth and want to add it now):

1. Select your key
2. Click **+ Add Subkey**
3. Choose the type: Sign / Encrypt / Auth
4. Confirm
5. pgpilot calls `gpg --quick-add-key` with the new subkey

This is useful when you realize you need a subkey you didn't include at creation.

### Renew subkey expiry

Subkeys expire by default (after 1, 2, or 5 years — see [Quickstart](2-quickstart.md)). Renewal extends the expiry date.

1. Select a key, then click a subkey card's **Renew** button
2. A modal appears asking for new expiry duration (1/2/5 years)
3. Click **Renew**
4. pgpilot calls `gpg --quick-set-expire`
5. The subkey's expiry updates

You can renew a subkey as many times as needed.

### Rotate a subkey (Replace)

Rotation creates a **new** subkey with fresh parameters and **revokes** the old one. Use this when:
- A subkey is compromised or suspected compromised
- You want to keep the primary key but refresh all subkeys

Process:

1. Select a key, then click a subkey card's **Replace** button
2. A modal appears asking for new expiry duration
3. Click **Replace**
4. pgpilot:
   - Snapshots your secret key (backup)
   - Creates a new subkey with `gpg --quick-add-key`
   - Revokes the old subkey via `gpg --edit-key` (dialogue-driven)
   - If revocation fails, restores the snapshot
5. When done, the old subkey is marked revoked; new one is active

**Note**: The primary key identity is unchanged — fingerprint stays the same.

---

## Exporting Keys

Share your public key with contacts, or backup your secret key.

### Export public key (file)

1. Select your key
2. Click **Export** or the three-dot menu
3. Choose **File**
4. pgpilot opens a file-save dialog
5. Choose a location (default: `YourName.pub.asc`)
6. Click **Save**

The `.asc` file contains only your **public key**. It's safe to share widely.

### Export public key (clipboard)

1. Select your key
2. Click **Export** or the three-dot menu
3. Choose **Clipboard**
4. A confirmation modal appears (to avoid accidental copy)
5. Click **Copy**
6. pgpilot copies the public key to clipboard
7. A green status message confirms: "Key copied"

Now paste it into an email, chat, or website.

### Export public key (upload to paste.rs)

1. Select your key
2. Click **Export** or the three-dot menu
3. Choose **Paste**
4. A confirmation modal appears
5. Click **Paste**
6. pgpilot uploads to paste.rs and returns a shareable link (e.g., `https://paste.rs/abc123`)
7. Share that link — anyone can fetch your public key without needing email or GitHub

This is handy for temporary sharing or embedding in a bio.

### Backup secret key

**IMPORTANT**: Your secret key is what allows you to decrypt messages and sign files. Back it up in a secure location!

1. Select your key
2. Click **Backup**
3. pgpilot opens a folder-picker dialog
4. Choose a secure location (USB drive, external SSD, safe, etc.)
5. Click **Select Folder**
6. pgpilot exports two files:
   - `<KeyID>_secret.asc` — your **secret key** (encrypted, requires password to use)
   - `<KeyID>_revocation.rev` — **revocation certificate** (if available)
7. A status message confirms: "Backup complete"

**Security practices**:
- Store backup offline (no network)
- Encrypt the USB drive itself
- Keep a copy in a physical safe if critical
- Test recovery annually (decrypt, check fingerprint)

---

## Importing Keys

Add someone else's public key to your keyring so you can send them encrypted messages.

### Import from file

1. Click **Import** in the sidebar
2. The **Import** view opens with tabs
3. Select **File**
4. Click **Choose file**
5. Select an `.asc` file containing a PGP key
6. Click **Import**
7. pgpilot validates and imports the key
8. If successful, you're returned to **My Keys** with the new key listed

### Import from URL

1. Click **Import**
2. Select **URL**
3. Paste an HTTPS link to a `.asc` file (e.g., `https://example.com/keys/alice.asc`)
4. Click **Load from URL**
5. pgpilot fetches the file and imports it

Must be HTTPS; HTTP is rejected for security.

### Import from keyserver

1. Click **Import**
2. Select **Keyserver**
3. Enter a search query:
   - **Fingerprint** (40 hex chars): `ABCD1234...`
   - **Key ID** (16 hex chars): `1234567890ABCDEF`
   - **Email**: `alice@example.com`
4. Choose keyserver:
   - **keys.openpgp.org** (default, recommended — privacy-respecting)
   - **keyserver.ubuntu.com** (traditional, returns all matching keys)
5. Click **Search**
6. pgpilot queries the keyserver and shows matching key(s)
7. Click **Import** for the key you want

For email queries, pgpilot URL-encodes the address and uses the `/pks/lookup?op=get` API endpoint.

### Import from pasted text

1. Click **Import**
2. Select **Paste**
3. Paste the entire armored key text (begins with `-----BEGIN PGP PUBLIC KEY BLOCK-----`)
4. The preview updates live
5. Click **Import**
6. pgpilot validates and adds the key

**Security note**: pgpilot validates that input starts with `-----BEGIN PGP`, rejecting HTML pages, error messages, etc.

---

## Deleting Keys

Remove a key from your keyring.

### Delete public key

If you have only the **public key** (no secret):

1. Select the key in **My Keys**
2. Click **Delete**
3. A confirmation modal appears: "Are you sure?"
4. Click **Delete**
5. pgpilot calls `gpg --delete-keys <fingerprint>`
6. Key is removed; status shows: "Key deleted"

### Delete secret key

If you have the **secret key**:

1. Select the key
2. Click **Delete**
3. A warning modal appears: "This action cannot be undone..."
4. Click **Delete secret key**
5. pgpilot calls `gpg --delete-secret-and-public-keys <fingerprint>`
6. Both secret and public key are erased
7. Status: "Key deleted"

**CRITICAL**: Before deleting your secret key, export a backup! Once deleted, you cannot decrypt old messages or revoke the key if it's compromised.

### Delete YubiKey stub

If a key is stored on a **YubiKey** (smart card):

1. Select the key
2. Click **Supprimer**
3. Confirm
4. pgpilot removes the key from your local keyring

The stub remains on the card; you can re-import it later. The private key is **not** erased from the card (only a `gpg` delete on the keyring stub).

---

## Publishing to Keyservers

Make your public key discoverable so others can find you by email.

### Publish a key

1. Select your key
2. Click **Publish**
3. A modal appears asking which keyserver:
   - **keys.openpgp.org** (recommended — verified keys only, privacy-preserving)
   - **keyserver.ubuntu.com** (traditional, lists all keys)
4. Click **Publish**
5. pgpilot calls `gpg --keyserver <url> --send-keys <fingerprint>`
6. If successful, status shows: "Key published to `<serveur>`"

### Check publication status

1. Select your key
2. In the detail panel, find the **Keyserver** badge:
   - **Unknown** (gray) — not yet checked
   - **Checking** (spinner) — checking now
   - **Published** (green) — found on keyserver
   - **Not Published** (red) — not found

pgpilot automatically checks status every time you view a key (calls `safe_get(https://keys.openpgp.org/...)` to verify).

### Auto-republish

pgpilot automatically re-publishes **already-published** keys every 4 weeks:
- When you rotate a subkey, pgpilot remembers which keyserver you used
- Every 28 days, it re-sends the key to keep your certificate fresh on the keyserver
- You can trigger manually anytime by clicking **Publish** again

This ensures your key stays visible and your latest subkeys are always discoverable.

### Share a keyserver link

Once published, you can share a direct link to your key:

```
https://keys.openpgp.org/search?q=alice@example.com
```

Others can fetch your key without your email or any central authority. 

---

## Next steps

- Encrypt files for contacts — see [File Operations](5-file-operations.md)
- Use SSH with your Auth subkey — see [YubiKey / Smartcard](6-smartcard.md) (SSH section)
- Rotate keys on hardware — see [YubiKey / Smartcard](6-smartcard.md)
- Troubleshoot issues — see [Troubleshooting](8-troubleshooting.md)
