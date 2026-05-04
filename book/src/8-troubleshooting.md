# Troubleshooting

Diagnose and fix common issues with pgpilot.

## Getting Help

Before trying fixes, collect diagnostic info:

1. Open pgpilot, click **Diagnostic** (sidebar)
2. Take a screenshot of the results
3. Note any red "Error" entries
4. Include this info when reporting bugs

---

## Installation & Startup

### "pgpilot: command not found"

**Cause**: Binary not in PATH.

**Fix**:
- If downloaded: move to `/usr/local/bin/`:
  ```bash
  sudo mv /path/to/pgpilot /usr/local/bin/
  pgpilot  # should work now
  ```
- If built from source: add to PATH:
  ```bash
  export PATH="$PATH:$(pwd)/target/release"
  ```

### "GPG not found" or "gpg: command not found"

**Cause**: GnuPG not installed.

**Fix**: Install GnuPG:
- Ubuntu/Debian: `sudo apt install gnupg`
- Fedora/RHEL: `sudo dnf install gnupg`
- Arch: `sudo pacman -S gnupg`
- macOS: `brew install gnupg`

Then restart pgpilot.

### pgpilot window won't appear on startup

**Cause**: X11/Wayland display issue.

**Fix**:
1. Check display is running:
   ```bash
   echo $DISPLAY     # X11 should show :0 or similar
   echo $WAYLAND_DISPLAY  # Wayland should show wayland-0 or similar
   ```
2. If empty, start a display server or SSH with display forwarding:
   ```bash
   ssh -X user@server  # X11 forward
   ```
3. Try running pgpilot again

**Note**: Wayland + iced can be finicky; if stuck, try X11 or GNOME/KDE Plasma.

---

## GPG and Keys

### "No keys found" / Empty My Keys list

**Cause**: GnuPG keyring is empty or corrupted.

**Fix**:
1. List keys manually:
   ```bash
   gpg --list-keys
   ```
2. If empty, create your first key in pgpilot: **Créer** → fill form → **Créer clef**
3. If `gpg --list-keys` shows keys but pgpilot doesn't:
   - Check GNUPGHOME env var:
     ```bash
     echo $GNUPGHOME  # should be empty (uses ~/.gnupg) or set to custom dir
     ```
   - If custom dir set, make sure it exists and contains keys
   - Restart pgpilot

### "Invalid fingerprint" error during import

**Cause**: File isn't a valid PGP key or is corrupted.

**Fix**:
1. Verify file is a valid key:
   ```bash
   gpg --import < /path/to/file.asc
   ```
2. If GnuPG rejects it, the file is invalid
3. Ask the sender for a fresh export:
   - Have them run: `gpg --export --armor <email>`
   - Send you the output

### "Passphrase required" prompt hangs

**Cause**: `pinentry` (password UI) is not running or misconfigured.

**Fix**:
1. Check pinentry is installed:
   ```bash
   which pinentry
   ```
   If not found:
   - Ubuntu/Debian: `sudo apt install pinentry` (or `pinentry-gtk2` for GUI)
   - Fedora: `sudo dnf install pinentry`
   - Arch: `sudo pacman -S pinentry`

2. If `pinentry` exists but hangs:
   - Kill stuck processes:
     ```bash
     pkill -f pinentry
     pkill -f gpg-agent
     ```
   - Start fresh:
     ```bash
     gpg-agent --daemon
     pgpilot
     ```

3. If using remote SSH, ensure X11/Wayland forwarding:
   ```bash
   ssh -X user@server  # Enable X11 forwarding
   ```

4. Last resort: use pinentry-tty (terminal-based, no GUI):
   ```bash
   echo "pinentry-program $(which pinentry-tty)" >> ~/.gnupg/gpg-agent.conf
   pkill gpg-agent
   pgpilot
   ```

### "Failed to encrypt" or "Trust level too low"

**Cause**: Recipient's key has insufficient trust (Undefined/Marginal).

**Fix**:
1. Verify recipient's key:
   - Select their key in pgpilot
   - Check fingerprint (ask them for it in person)
   - Click **Confiance** badge → **Complète**
2. Try encrypting again

Or override trust for one-off:
- Click **Chiffrer** (Encrypt)
- Check untrusted recipient
- Click **Chiffrer**
- When warning appears, click **Continuer** (Continue)

This uses `--trust-model always` (bypasses trust check without modifying trust database).

---

## Keyserver & Publishing

### "Publication failed" / "Keyserver error"

**Cause**: Network issue, keyserver down, or key already published.

**Fix**:
1. Check network:
   ```bash
   ping keys.openpgp.org
   ```
2. Wait a few minutes, retry
3. Try different keyserver:
   - Click **Publier** (Publish)
   - Choose **keyserver.ubuntu.com** instead of **keys.openpgp.org**

4. If key was published, verify it's there:
   ```bash
   gpg --keyserver keys.openpgp.org --recv-keys <your-fingerprint>
   ```

### "Key not found after publishing"

**Cause**: Publication succeeded, but indexing hasn't completed yet (keys.openpgp.org takes ~5 min).

**Fix**:
1. Wait 5-10 minutes
2. Refresh in pgpilot (select key again or restart)
3. If still not found after 30 min:
   - Check email from keys.openpgp.org (they may require email verification)
   - Click verification link in email
   - Republish: Click **Publier** again

### "Only find wrong key when searching"

**Cause**: Multiple keys exist for that email, or old key hasn't been revoked.

**Fix**:
1. Verify fingerprint when importing:
   - In **Importer** → **Serveur de clefs**, check the full fingerprint
   - Match with what sender provided
2. If it's the wrong key:
   - Don't import it
   - Contact sender for correct fingerprint

---

## File Operations

### "File already exists" when encrypting/decrypting

**Cause**: Output file already exists (pgpilot doesn't overwrite).

**Fix**:
- pgpilot auto-adds counters: `document_1.gpg`, `document_2.gpg`, etc.
- Delete the old output file if you want to overwrite
- Or rename the old file before re-encrypting

### "Signature file not found" during verification

**Cause**: `.sig` file in different folder than original file.

**Fix**:
1. Move both files to same folder:
   ```bash
   # Both in ~/Downloads/
   ~/Downloads/document.pdf
   ~/Downloads/document.pdf.sig
   ```
2. In pgpilot, choose both files (pgpilot will auto-find `.sig` if same folder)

Or manually specify both:
- Click **Choisir le fichier** → document.pdf
- Click **Choisir la signature** → document.pdf.sig

### "Can't decrypt — missing key"

**Cause**: File was encrypted for someone else; you lack their public key (and your private key).

**Fix**:
1. Ask sender to re-encrypt for you
2. Or, ask original sender for a copy (unencrypted or re-encrypted for you)

There's no way to decrypt files encrypted for others.

---

## YubiKey / Smartcard

### "Card not detected"

**Cause**: YubiKey not plugged in, or `scdaemon` not running.

**Fix**:
1. Plug in the YubiKey (USB-A, USB-C, or NFC depending on model)
2. Check `scdaemon` is running:
   ```bash
   ps aux | grep scdaemon
   ```
3. If not running, start it:
   ```bash
   gpg-agent --daemon
   gpg --card-status  # Forces scdaemon to start
   ```
4. Restart pgpilot

### "Migration failed / Wrong PIN"

**Cause**: Incorrect admin PIN (default: `12345678`).

**Fix**:
1. Try again with default PIN: `12345678`
2. If that fails, you may have changed the PIN previously
   - Check your records
   - If lost, reset the card (see below — **data loss**)

### "Card full" / "Can't migrate third subkey"

**Cause**: OpenPGP Card 3.x holds max 3 subkeys (Sign, Encr, Auth).

**Fix**:
- You already have 3 subkeys on this card
- Options:
  1. Use a second card for a different key
  2. Overwrite an existing subkey (delete from card, re-migrate new one)

### "Reset / Factory reset the card"

**⚠️ WARNING**: This erases ALL keys on the card irreversibly.

Only do if:
- Card is lost/found
- PIN is forgotten and can't be recovered
- Card is physically damaged

**To reset**:
```bash
gpg --card-edit
admin
factoryreset
# Type "yes" to confirm
quit
```

Then reinitialize:
```bash
gpg --card-edit
admin
passwd  # Set new PIN
quit
```

---

## Health Checks

pgpilot's **Diagnostic** page checks 8 common issues. Red "Error" means:

1. **GPG installed**: GnuPG binary not found → Install GnuPG
2. **GPG version ≥ 2.1**: Too old → Update GnuPG
3. **gpg-agent running**: Agent crashed → Restart: `gpg-agent --daemon`
4. **pinentry found**: No password UI → Install pinentry (see above)
5. **default-cache-ttl**: Passphrase cached 0 seconds → Edit `~/.gnupg/gpg-agent.conf`
6. **max-cache-ttl**: Passphrase never cached → Edit config
7. **~/.gnupg permissions**: Not 700 → Fix: `chmod 700 ~/.gnupg`
8. **Revocation certs exist**: Missing `.rev` files → Restore from backup or accept warning

---

## Still Stuck?

1. **Check Diagnostic page** (sidebar) — tells you which checks are failing
2. **Search FAQ** — see [FAQ](7-faq.md)
3. **Search GitHub issues**: https://github.com/gfriloux/pgpilot/issues
4. **Report a bug**: Include:
   - Diagnostic output (screenshot)
   - OS (Ubuntu 24.04, Fedora 39, macOS 14, etc.)
   - GnuPG version: `gpg --version`
   - Exact error message
   - Steps to reproduce

---

## Manual Commands

If pgpilot UI doesn't work, try command-line:

```bash
# List your keys
gpg --list-keys

# Export public key
gpg --export --armor <fingerprint> > mykey.pub.asc

# Import a key
gpg --import theirkey.asc

# Encrypt a file
gpg --encrypt --recipient <email> --armor document.pdf

# Decrypt
gpg --decrypt document.pdf.asc

# Sign
gpg --detach-sign --armor document.pdf

# Verify
gpg --verify document.pdf.asc document.pdf

# Card status
gpg --card-status

# Set trust
echo '<fingerprint>:5:' | gpg --import-ownertrust
```

These commands do exactly what pgpilot does behind the scenes.
