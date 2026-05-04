# FAQ

Frequently asked questions about pgpilot and OpenPGP.

## General

### What's the difference between a public and private key?

- **Public key**: Safe to share. Used to encrypt messages *to* you. Anyone can have it.
- **Private key**: Secret. Used to decrypt messages and sign. **Never share.**

Think: public key = mailbox (anyone can drop letters); private key = key to your mailbox (only you).

### Why don't my keys expire?

pgpilot creates keys that **never expire by design**. Subkeys expire (after 1, 2, or 5 years), but the master key is permanent.

**Why?** Expiring master keys is more complex and rare. Subkey rotation (Remplacer) is the modern best practice.

If you need expiring keys, use `gpg` directly: `gpg --quick-gen-key "Name" ed25519 cert 1y`.

### What's a fingerprint? What's a Key ID?

- **Fingerprint**: 40 hexadecimal characters. Unique identifier for your key.
  - Example: `ABCD1234567890ABCD1234567890ABCD1234567890`
  - Always verify this in person before trusting someone's key

- **Key ID**: Last 16 characters of fingerprint (long ID) or last 8 characters (short ID).
  - Long ID: `1234567890ABCDEF`
  - Short ID: `90ABCDEF` (⚠️ **Not recommended** — trivially collisible)

pgpilot always uses full 40-character fingerprints.

### Why separate subkeys?

Three subkeys (Sign, Encryption, Auth) allow:
- **Rotation**: Refresh subkeys without changing your identity
- **Delegation**: Share subkeys with services without sharing master key
- **Hardware**: Move individual subkeys to YubiKey
- **Specialization**: Each key optimized for its purpose

This is modern best practice.

### Can I revoke a key?

**Revocation** marks a key as no longer valid (e.g., it's compromised or lost).

**In pgpilot**: You cannot revoke the master key (pgpilot has no UI for this). For emergencies, use:
```bash
gpg --gen-revoke <fingerprint> > revocation.asc
gpg --import revocation.asc
gpg --keyserver keys.openpgp.org --send-keys <fingerprint>
```

**Subkeys**: Use **Remplacer** (Rotate) in pgpilot to revoke old subkeys and create new ones.

---

## Keys and Keyrings

### Where does pgpilot store my keys?

In the standard GnuPG home directory:
- Linux: `~/.gnupg/`
- macOS: `~/.gnupg/` or `/Users/<username>/.gnupg/`
- Windows: `%APPDATA%\gnupg\`

pgpilot doesn't create its own keyring — it delegates to `gpg`.

### Can I use pgpilot with keys created by `gpg`?

Yes! pgpilot reads and manages any GPG keyring. You can:
- Create keys with `gpg` and manage them in pgpilot
- Create keys in pgpilot and use them with `gpg`

They're fully compatible.

### How do I backup my private key?

1. In pgpilot: Select your key → Click **Sauvegarder** → Choose a folder
2. pgpilot exports:
   - `<KeyID>_secret.asc` — your private key (encrypted)
   - `<KeyID>_revocation.rev` — revocation certificate

Store the backup somewhere secure (offline, encrypted external drive, safe).

### How do I restore a backup?

1. Copy the `_secret.asc` file to a safe location
2. In pgpilot: Click **Importer** → **Fichier** → choose the `.asc` file
3. pgpilot imports your key back

Your backup passphrase is required to re-import.

---

## Trust and Verification

### What's "trust level"?

Trust tells pgpilot how much you believe in someone's identity:

- **Undefined**: You haven't verified (default)
- **Marginal**: You've partially verified (e.g., verified identity but not fingerprint)
- **Full**: You've fully verified (met in person, compared fingerprints)
- **Ultimate**: You own this key (your own keys auto-set to Ultimate)

pgpilot warns before encrypting to Undefined keys.

### When should I set trust?

After you:
1. Meet someone in person
2. Ask them to say their fingerprint
3. Verify it matches their key in pgpilot
4. Set trust to Full

**Never** set trust to Full based on email alone.

### How do I verify a signature?

1. Get the file (e.g., `document.pdf`)
2. Get the signature (e.g., `document.pdf.sig`)
3. In pgpilot: Click **Vérifier** → choose file and signature
4. pgpilot shows result (Valid / Bad Sig / Unknown Key / Expired / Revoked)

If Valid and signer trust is Full/Ultimate, the document is authentic and from who you think.

---

## Sharing and Publishing

### Should I publish my key to a keyserver?

**Yes**, if you want people to find you by email. Once published:
- Others search for `your@email.com`
- They find and download your public key
- They can send you encrypted messages

Publishing is safe — it's public information.

### Which keyserver should I use?

- **keys.openpgp.org** (recommended): Privacy-respecting, requires email verification
- **keyserver.ubuntu.com**: Traditional, lists emails publicly

Most people use keys.openpgp.org now.

### What happens when I publish?

pgpilot calls `gpg --send-keys <fingerprint>` to:
1. Upload your **public key** (not private!)
2. Keyserver indexes it by fingerprint + email
3. Anyone can now download your public key

Your private key **never** leaves your computer.

### Can I remove my key from a keyserver?

Once published, keys persist (they can't be truly deleted from distributed keyservers). You can:
1. Revoke the key (marks it as invalid)
2. Set privacy options (keys.openpgp.org supports hiding your email)

For complete removal, contact the keyserver admins.

---

## Subkeys

### Why do subkeys expire?

Expiring subkeys force you to rotate them periodically. If a subkey leaks, you only need to rotate that subkey, not your identity.

pgpilot defaults to 2 years, but you can choose 1 or 5 years.

### What's the difference between "Renew" and "Replace"?

- **Renew** (Renouveler): Extend the expiry date of the same subkey
  - Use if the subkey is still good, just old
  - Quick operation

- **Replace** (Remplacer): Create a new subkey and revoke the old one
  - Use if the subkey is compromised or you want to refresh
  - Creates a new key with fresh parameters
  - Old key is marked revoked

### Can I have multiple Sign keys?

Yes. Use **Ajouter sous-clef** to add extra Signature subkeys. You can have:
- Multiple Sign keys (e.g., one for work, one for personal)
- Multiple Encryption keys (for future algorithm migration)
- Multiple Auth keys (for different SSH identities)

But you need at least one of each for most operations.

---

## Encryption

### Who can decrypt my encrypted files?

Only the recipients you specified. Each recipient uses their **private key** to decrypt.

You cannot decrypt files encrypted for others (even if you created them).

### What if I lose my private key?

Without the private key:
- You cannot decrypt files encrypted to you
- You cannot sign documents as you
- You cannot use SSH with your Auth key

This is why backup is critical. If lost:
1. Revoke the key (publish revocation)
2. Create a new key
3. Tell everyone your new key

**Prevention**: Back up now (see "How do I backup my private key?" above).

### Should I encrypt files to myself?

Often, yes! Before sending to someone:
1. Encrypt a test file to yourself
2. Decrypt it to verify it works
3. Then encrypt to the real recipient

This catches mistakes before sending.

---

## Security

### Is pgpilot secure?

pgpilot is a **GUI wrapper** around `gpg`. Security depends on:
- **GnuPG**: Mature, battle-tested (used by billions)
- **Your system**: Protect your computer, keep OS updated
- **Your keys**: Use strong passphrases, back up, rotate compromised keys
- **Your choices**: Verify fingerprints before trusting

pgpilot is no worse than using `gpg` directly.

### What if my computer is hacked?

Hacker gains access to:
- Your private keys (on disk, if not on YubiKey)
- Your passphrases (if typed after infection)
- Decrypted files (if stolen or read)

**Mitigation**:
- Use a YubiKey (keys never leave hardware)
- Use strong passphrases (slows brute-force)
- Keep OS patched (fewer zero-days)
- Revoke compromised keys immediately

### What's the master key for?

The master key certifies (signs) your subkeys. Only the master key can:
- Create new subkeys
- Revoke subkeys
- Add user IDs (email addresses)
- Sign other keys (for key signing parties)

The master key is long-term and should be kept secure (ideally offline or on hardware).

### Should I keep my master key on disk?

Best practice:
- **On disk**: Keep a backup offline (USB in safe)
- **On hardware**: Migrate subkeys to YubiKey, leave master key on disk (you rarely use it)
- **Offline**: Air-gapped machine (most paranoid, rarely needed)

For most people: on disk with a strong passphrase is fine.

---

## Troubleshooting

### pgpilot won't start

See [Installation](1-installation.md) — verify GnuPG and `pinentry` are installed.

### Password prompt doesn't appear

See [Troubleshooting](8-troubleshooting.md) — pinentry section.

### Keys won't import

Check [Troubleshooting](8-troubleshooting.md) — import section.

### YubiKey not detected

See [Troubleshooting](8-troubleshooting.md) — YubiKey section.

---

## More questions?

- See [Troubleshooting](8-troubleshooting.md) for common issues
- See [Security](9-security.md) for threat models
- Email: guillaume+code@friloux.me (not monitored 24/7; use GitHub issues for bugs)
