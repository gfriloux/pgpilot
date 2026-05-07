# Security

pgpilot is designed around one principle: **delegate all cryptography to GnuPG**.
pgpilot itself never reads, holds, or manipulates private key material — that always
stays inside `gpg-agent`, protected by the passphrase or hardware PIN you set up.

## Security model

```
You ──► pgpilot ──► gpg subprocess ──► gpg-agent ──► private key
                         │
                    env_clear()
                    --homedir
                    HTTPS only
                    1 MiB cap
```

pgpilot acts as a UI shell. Every cryptographic operation is a `gpg` subprocess call
with a clean environment (`env_clear()`). Private keys never cross the pgpilot process
boundary.

## Assets protected

| Asset | Where it lives | pgpilot's role |
|-------|---------------|----------------|
| Private keys | `gpg-agent` (encrypted) | Never read — fully delegated |
| Public keyring | `~/.gnupg/pubring.gpg` | Import / export via `gpg` subprocess |
| Trust database | `~/.gnupg/trustdb.gpg` | Set via `gpg --import-ownertrust` |
| Chat rooms | `~/.config/pgpilot/rooms.yaml` | Room IDs and participant fingerprints only — no messages |
| Chat messages | RAM only (500 per room, FIFO) | Encrypted blobs, lost on exit |

## Threat model

### 1. Malicious key import

An attacker controls the UID displayed in pgpilot. The `signer_name` field comes from
GnuPG's `GOODSIG` token — it reflects what is stored in the key, not a cryptographic
guarantee of identity.

**Mitigation**: every key shows a trust badge (`Undefined` / `Marginal` / `Full` /
`Ultimate`). `Undefined` keys never show a green badge, even when a signature is
mathematically valid. The user is responsible for verifying fingerprints out-of-band
before elevating trust.

---

### 2. MitM on keyserver HTTP

A network attacker intercepts a key import and substitutes a different key.

**Mitigation**: HTTPS only on all outbound requests (`safe_get` rejects `http://`).
TLS certificates are validated against the system trust store. The full 40-character
fingerprint is always shown after import.

---

### 3. OOM via oversized server response

A malicious or compromised keyserver returns gigabytes of data.

**Mitigation**: all network reads are capped at 1 MiB (`MAX_RESPONSE_BYTES`). Beyond
that limit, the read is interrupted and a user-visible error is returned.

---

### 4. HKP parameter injection

A keyserver query built from unvalidated input could inject extra URL parameters.

**Mitigation**: strict input validation — only 40-hex fingerprints, 16-hex long key IDs,
or well-formed email addresses are accepted. All query values are percent-encoded before
URL construction.

---

### 5. Evil32 fingerprint collision

Short 8-character IDs (32 bits) are trivially collisible — an attacker can generate a
key matching any target's short ID.

**Mitigation**: `validate_fp()` enforces exactly 40 hexadecimal characters (160-bit
fingerprint) on every gpg function that accepts a fingerprint. Short IDs are not accepted
for any cryptographic operation.

---

### 6. Drag-and-drop of a crafted file

A malicious file is dropped into the encryption view.

**Residual risk**: pgpilot encrypts whatever the user explicitly drops — it does not
inspect file contents. GnuPG does not decompress or execute content. The risk is limited
to the user encrypting data they did not intend to.

---

### 7. Clipboard exposure

A copied public key armour or `paste.rs` URL remains on the clipboard until replaced.

**Residual risk**: pgpilot never copies private key material or passphrases to the
clipboard. On Wayland, clipboard lifetime is tied to window focus. Automatic clearing is
not reliably implementable without compositor-level support.

---

### 8. Screen recording

A screen recorder captures fingerprints, UIDs, and trust statuses.

**Out of scope**: pgpilot never displays private key material. Screen protection is an
OS/compositor responsibility.

---

### 9. LD_PRELOAD / environment injection

A malicious library injected via `LD_PRELOAD` could intercept gpg operations.

**Mitigation**: every `gpg` subprocess is spawned with `env_clear()`. Only the required
variables (`HOME`, `GNUPGHOME`, `PATH`, `DISPLAY`, `WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR`,
`DBUS_SESSION_BUS_ADDRESS`, `GPG_TTY`, `PINENTRY_USER_DATA`) are re-injected explicitly.

---

### 10. GNUPGHOME undefined or unexpected

If `GNUPGHOME` is unset, GnuPG silently falls back to `~/.gnupg`. In test or CI
environments this can cause operations on the wrong keyring.

**Mitigation**: `gnupg_dir()` is called at the start of every gpg operation and returns
an explicit error if neither `GNUPGHOME` nor `HOME` is set. No silent fallback.

---

## Chat-specific threats (v0.6.0)

| Threat | Mitigation | Residual |
|--------|-----------|---------|
| Relay reads messages | E2E PGP encryption (`gpg --encrypt --sign --armor`) | Relay sees timestamps, topics, participant count |
| Sender impersonation | `VALIDSIG` primary-key fingerprint matched against `wire.sender` and `room.participants` | Only keys in local keyring are trusted |
| Third-party message injection | Signer fingerprint verified against room participant list before display | Anyone who knows the topic can publish; only listed recipients can decrypt |
| Forged join code | `JoinCode::verify()` checks PGP signature over `room_id ‖ relay ‖ invited_by`; relay must be `mqtts://` | Inviter's key must already be in local keyring |
| Fake presence status | *None in v0.6.0* — presence is not signed | Malicious broker can publish false Online/Offline |
| Message replay | `\|wire.ts − now\| ≤ 86400 s`; signature covers `id ‖ sender ‖ ts ‖ payload` | Replay within 24 h window if msg_id not already seen |
| Oversized payload DoS | Payloads > 64 KiB rejected before decryption | Broker may accept larger; client drops them |
| Topic fingerprinting | Topic = `sha256(room_id)[0..8]` (opaque) | 64-bit hash exposed; timing and cardinality observable |
| Metadata surveillance | Opaque topics, truncated fingerprints in presence/ACK topics | Relay sees timing and room cardinality |
| rooms.yaml DoS | File rejected if > 1 MiB; fields validated after deserialization | Partial corruption fails the whole load |
| No forward secrecy | *Out of scope v0.6.0* — documented limitation | Key compromise allows retrospective decryption of captured traffic |

## Out of scope

The following are acknowledged but outside pgpilot's threat boundary:

- **Local machine compromise** — rootkits, keyloggers, memory inspection
- **Physical attacks on YubiKey** — side-channel extraction, decapsulation
- **Compromised TLS chain** — rogue CA, HTTPS stripping at scale
- **Hardware side channels** — timing, power, EM
- **Compromised GnuPG binary or gpg-agent**
- **Malicious pinentry** — passphrase interception

## Security assumptions

| Assumption | Why it matters |
|-----------|---------------|
| The `gpg` binary in `PATH` is trustworthy | pgpilot delegates all cryptography to GnuPG |
| `gpg-agent` is not compromised | Private keys reside there |
| `pinentry` works correctly and is not replaced | Passphrase entry must be isolated |
| The OS is not compromised at runtime | Memory, process, and filesystem primitives must be reliable |

## See also

- [Remediation Plan](10-security-plan.md) — status of all security fixes (14 items across 4 phases)
- [`THREAT_MODEL.md`](https://github.com/gfriloux/pgpilot/blob/main/THREAT_MODEL.md) — source document (French)
- [`SECURITY_PLAN.md`](https://github.com/gfriloux/pgpilot/blob/main/SECURITY_PLAN.md) — implementation guide with code references (French)
