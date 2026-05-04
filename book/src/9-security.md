# Security

Threat model, best practices, and limitations.

## Threat Model

pgpilot is designed to protect against these threats:

### Data Eavesdropping

**Threat**: Attacker intercepts your files in transit (email, cloud, USB).

**Protection**: File encryption.
- **How**: Use pgpilot to encrypt files before sending.
- **Result**: Only recipients with private keys can decrypt.

### Forged Messages

**Threat**: Attacker sends a message claiming to be you (impersonation).

**Protection**: Digital signatures.
- **How**: Use pgpilot to sign files. Recipients verify with your public key.
- **Result**: Only you could have signed (proves authenticity).

### Key Theft (Disk)

**Threat**: Attacker steals your computer or disk.

**Protection**: Hardware keys (YubiKey) or passphrases.
- **Hardware**: Private keys never leave the card. Attacker gets nothing.
- **Passphrase**: Private key is encrypted on disk. Attacker must crack passphrase.

### Subkey Compromise

**Threat**: One subkey (e.g., Encryption) is compromised, but you don't know.

**Protection**: Subkey rotation and revocation.
- **How**: Use pgpilot **Replace** to create new subkey and revoke old.
- **Result**: Old subkey marked revoked; new one is active.

### Key Loss

**Threat**: You lose access to your private key (hardware failure, deletion).

**Protection**: Backups.
- **How**: Use pgpilot **Backup** to export secret key and revocation cert.
- **Result**: If you lose key, you can restore from backup. If you can't restore, you can revoke and create new key.

---

## What pgpilot Does NOT Protect Against

### Compromised Computer

**Threat**: Malware installs on your computer and reads private keys, passphrases, or plaintext files.

**Limitations**:
- pgpilot uses `gpg` on your computer. If OS is compromised, attacker can intercept.
- If using a YubiKey, keys themselves are protected, but attacker could intercept commands to sign/decrypt.
- Decrypted files on disk are readable to malware.

**Mitigation**:
- Keep OS patched (fewer vulnerabilities)
- Use antivirus / malware scanner
- Don't run untrusted binaries
- Use a hardware key (harder to extract secrets)

### Weak Passphrases

**Threat**: Attacker brute-forces your private key password.

**Limitations**:
- If passphrase is weak (e.g., "password123"), attacker can try all possibilities.
- 2024 GPUs can test billions of passphrases per second.

**Mitigation**:
- Use strong passphrases: 15+ characters, mixed case, numbers, symbols
- Use a passphrase manager (Bitwarden, 1Password, KeePass)
- Use a hardware key (no passphrase needed, can't be brute-forced)

### Trust Mistakes

**Threat**: You trust the wrong person's key (you don't actually verify their identity).

**Limitations**:
- pgpilot relies on you to verify fingerprints in person.
- If you skip verification, you might encrypt to an attacker's key.

**Mitigation**:
- **Always** verify fingerprints by comparing in person or via a secure channel.
- Use pgpilot's trust levels (Marginal / Full) to mark who you've verified.
- Encrypt test files to yourself first; only encrypt to strangers after verification.

### Network Eavesdropping (Keyserver Lookups)

**Threat**: Attacker monitors your network and sees which keys you're searching for (privacy leak).

**Limitations**:
- When you search a keyserver, your search query is visible (unless VPN/Tor).
- Attacker can infer who you're communicating with.

**Mitigation**:
- Use VPN or Tor for keyserver lookups
- Use keys.openpgp.org (doesn't return email addresses in searches, more private)
- Avoid searching by email; search by fingerprint instead

### GPG Bugs

**Threat**: Undiscovered vulnerability in GnuPG could leak keys or signatures.

**Limitations**:
- pgpilot delegates all crypto to `gpg`. If `gpg` is broken, pgpilot is broken.

**Mitigation**:
- Keep GnuPG updated: `gpg --version` should show latest version
- Monitor security advisories: https://gnupg.org/security.html
- Use a hardware key (reduces attack surface; much of crypto happens on-chip)

---

## Best Practices

### Key Creation

1. **Create via pgpilot or `gpg --batch`** (not interactive; avoids mistakes)
2. **No expiry on master key** (pgpilot default; correct design)
3. **2-year expiry on subkeys** (pgpilot default; balance freshness + usability)
4. **Use ed25519 / cv25519** (pgpilot default; modern, secure)

### Passphrases

1. **Strong**: 15+ characters, mix of case + numbers + symbols
2. **Unique**: Don't reuse passwords from other accounts
3. **Backed up**: Store securely in a password manager
4. **Reviewed regularly**: Ensure you still remember it

### Backups

1. **Backup immediately after key creation** (use pgpilot **Backup**)
2. **Store offline**: USB in a safe, not on your computer
3. **Encrypt the backup** (use VeraCrypt, LUKS, BitLocker)
4. **Test annual recovery**: Restore to temp machine, verify it works
5. **Document passphrases**: Store separately from key backup (in password manager)

### Trust & Verification

1. **Verify fingerprints in person** (meet face-to-face if possible)
2. **Or via trusted channel** (video call, phone call from known number)
3. **Never trust email/text** (attacker can intercept)
4. **Set trust explicitly** (don't encrypt to Undefined-trust keys)
5. **Review trust annually** (revoke if person is no longer trustworthy)

### Key Rotation

1. **Renew subkeys before expiry** (1-2 months before expiration date)
2. **Rotate compromised subkeys immediately** (use pgpilot **Replace**)
3. **Publish rotations to keyserver** (use pgpilot **Publish**)
4. **Announce to contacts** (email with new key details)

### Hardware Keys (YubiKey)

1. **Change default PIN** (from `12345678` to strong PIN)
2. **Keep with you** (don't leave unattended)
3. **Backup master key offline** (YubiKey only holds subkeys)
4. **Test recovery**: If card lost, can you restore from backup?
5. **Document serial number** (in case of loss/theft)

### File Encryption

1. **Test encryption/decryption** (encrypt to yourself first)
2. **Use armor format** (`.asc`) for plain text / email
3. **Use binary format** (`.gpg`) for files, archives
4. **Keep signatures with files** (store `.sig` next to encrypted file)
5. **Verify signatures before trusting** (always check signer trust)

---

## Limitations of pgpilot

### No Master Key Expiry

pgpilot doesn't let you set master key expiry. This is intentional:
- Master key expiry is complex and rarely needed
- Modern practice: keep master key forever; rotate subkeys instead
- If you need expiring master keys, use `gpg --quick-gen-key "Name" ed25519 cert 1y`

### No Key Server Pulling (Search Only)

pgpilot can import keys from keyservers (pull), but cannot pull **updates** to your existing keys.

**Workaround**: Periodically re-import your contacts' keys to get latest versions:
```bash
gpg --keyserver keys.openpgp.org --recv-keys <their-fingerprint>
```

Or use external tools:
```bash
gpg --auto-key-retrieve  # Fetches unknown signing keys automatically
```

### No Expiry Modification (Master Key)

You cannot change master key expiry in pgpilot. Use `gpg` directly:
```bash
gpg --quick-set-expire <master-fp> <duration>
```

### No User ID Management

pgpilot doesn't add/remove user IDs (email addresses) on keys.

**Workaround**: Use `gpg --edit-key <fp>` → `adduid` / `deluid`.

### No Direct SSH Key Export (Auth Subkey)

pgpilot doesn't export Auth subkeys as SSH public keys. Use:
```bash
gpg --export-ssh-key <fingerprint> > ~/.ssh/id_pgp.pub
```

pgpilot may add this in a future version.

### No Key Signing (Web of Trust)

pgpilot doesn't sign other keys (key signing parties). Use:
```bash
gpg --default-key <your-fp> --sign-key <their-fp>
```

### No Revocation Generation (Master Key)

pgpilot cannot generate revocation certificates for master keys. Use:
```bash
gpg --gen-revoke <master-fp> > revocation.asc
```

Keep this file offline in case of emergency.

---

## Reporting Security Issues

If you discover a security vulnerability in pgpilot:

1. **Do NOT post on GitHub issues** (public disclosure could harm users)
2. **Email**: guillaume+code@friloux.me
3. **Include**:
   - Detailed description of vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)
4. **Allow time** for pgpilot maintainers to respond and patch

Responsible disclosure is appreciated. Security researchers will be credited.

---

## OpenPGP Standards

pgpilot uses:
- **OpenPGP RFC 4880** (IETF standard)
- **Modern algorithms**: ed25519 (sign), cv25519 (encrypt), SHA-256+
- **Deprecated algorithms avoided**: RSA 1024, SHA-1, MD5, IDEA, 3DES, etc.

GnuPG enforces these standards, preventing weak keys.

---

## Further Reading

- **GnuPG Manual**: https://gnupg.org/documentation/manuals/gnupg/
- **RFC 4880** (OpenPGP spec): https://tools.ietf.org/html/rfc4880
- **keys.openpgp.org privacy policy**: https://keys.openpgp.org/about
- **YubiKey documentation**: https://support.yubico.com/
- **NIST Cybersecurity Framework**: https://www.nist.gov/cyberframework

---

## Questions?

See [FAQ](7-faq.md) or [Troubleshooting](8-troubleshooting.md) for answers to common questions.
