# Security Remediation Plan

This page tracks the status of all security fixes identified during the pgpilot security
audit. Items are ordered by ANSSI priority (1 = critical → 4 = documentation).

**Legend**: ✅ Done · ⚠️ Partial / out of scope · ⏳ Blocked

**Summary**: 11/14 done · 2 partial or out of scope · 1 blocked

---

## Phase 1 — Critical fixes

### 1.1 — Signature verification: signer trust level ✅

**Problem**: `verify_signature` returned `VerifyOutcome::Valid` on any `VALIDSIG` token,
regardless of whether the signer's key was trusted. An attacker could import any public
key and produce a signature shown as green "Valid" in the UI.

**Fix**: `VerifyResult` now carries `signer_trust: TrustLevel`. The UI shows three
distinct states:
- ✅ Green — `Valid` + `Full` or `Ultimate` trust
- 🟠 Orange — `Valid` + `Marginal` trust
- 🟡 Yellow — `Valid` + `Undefined` trust ("Signature correct — identity not verified")

**Files**: `src/gpg/types.rs`, `src/gpg/keyring.rs`, `src/ui/verify.rs`

---

### 1.2 — HTTP requests: size cap, HTTPS enforcement, redirect limit ✅

**Problem**: `read_to_string()` with no size limit risked OOM. `http://` and `file://`
URLs were accepted, enabling MitM and local SSRF. No redirect limit.

**Fix**: `safe_get(url)` in `keyring.rs`:
- Rejects any URL not starting with `https://`
- Caps response body at 1 MiB (`MAX_RESPONSE_BYTES = 1 << 20`)
- Limits redirects to 3 (`max_redirects(3)`)

All network reads go through `safe_get`. The keyserver POST uses the same builder
settings.

**Files**: `src/gpg/keyring.rs`

---

### 1.3 — Output files: no silent overwrite, no `--yes` ✅

**Problem**: output path built by string concatenation + `--yes` passed to gpg → silent
overwrite of existing files and symlink follow.

**Fix**: anti-collision loop (`while output.exists()`) adds a counter suffix
(`file_1.pdf.gpg`, `file_2.pdf.gpg`, …). `--yes` removed from `encrypt_files` and
`sign_file`. `--` used as separator before the output path.

**Files**: `src/gpg/keyring.rs`

---

## Phase 2 — High priority fixes

### 2.1 — Fingerprint and keyserver query validation ✅

**Problem**: no format validation on fingerprints before passing to gpg. Short 8-char
IDs (Evil32 attack) were accepted.

**Fix**: `validate_fp(fp)` enforces exactly 40 ASCII hex characters. Called as the first
line in every function that accepts a fingerprint. Keyserver queries are validated and
percent-encoded. The "short ID (8 chars)" hint removed from the import UI.

**Files**: `src/gpg/keyring.rs`, `src/ui/import.rs`

---

### 2.2 — Error messages: no raw paths or gpg stderr tokens ✅

**Problem**: error messages shown to users included absolute `$HOME/...` paths and raw
`[GNUPG:]` tokens from gpg stderr.

**Fix**: `display_path(path)` and `sanitize_gpg_stderr(stderr)` in `src/gpg/mod.rs`.
Used throughout all user-visible error formatting.

**Files**: `src/gpg/mod.rs`, `src/gpg/keyring.rs`, `src/app/`

---

### 2.3 — Clipboard auto-clear ⚠️

**Problem**: content copied to the clipboard is never cleared.

**Status — out of scope**: on Wayland, clipboard lifetime is tied to the source window's
focus; reliable auto-clearing is not possible without compositor-level support. On X11,
no standard mechanism prevents other processes from reading the clipboard.

pgpilot never copies private key material or passphrases to the clipboard. Only public
key armour and `paste.rs` URLs are ever copied.

**Files**: `src/app/export.rs`, `src/app/card.rs`

---

## Phase 3 — Medium priority fixes

### 3.1 — Clean environment for gpg subprocesses ✅

**Problem**: gpg subprocesses inherited the full parent environment, including
`LD_PRELOAD` and NixOS dev-shell `LD_LIBRARY_PATH`.

**Fix**: `gpg_command(homedir: &str)` in `src/gpg/mod.rs` calls `env_clear()` then
re-injects only the required variables: `HOME`, `GNUPGHOME`, `LANG`, `PATH`, `DISPLAY`,
`WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR`, `DBUS_SESSION_BUS_ADDRESS`, `GPG_TTY`,
`PINENTRY_USER_DATA`. All `Command::new("gpg")` calls replaced.

**Files**: `src/gpg/mod.rs`, `src/gpg/keyring.rs`, `src/gpg/card.rs`

---

### 3.2 — Consistent GNUPGHOME: single resolution + systematic `--homedir` ✅

**Problem**: `gnupg_dir()` called per-invocation; some functions forgot `--homedir`,
risking operations on the wrong keyring.

**Fix**: `gpg_command(homedir)` injects `--homedir` on every subprocess. `gnupg_dir()`
validates that `HOME` or `GNUPGHOME` are set and fails with an explicit error otherwise.
All functions call `gnupg_dir()?` and pass the result to `gpg_command`.

**Files**: `src/gpg/mod.rs`, `src/gpg/keyring.rs`, `src/gpg/card.rs`, `src/gpg/health.rs`

---

### 3.3 — Subkey revocation robustness ⚠️

**Problem**: the `key N\nrevkey\ny\n2\n\ny\nsave\n` stdin sequence sent to
`gpg --edit-key` is fragile and depends on the exact prompt order of the installed gpg
version.

**Status — partial**: `revoke_subkey_at_pos()` still uses `--edit-key` but now passes
`--status-fd 2 --command-fd 0` (token-driven dialogue: reads `[GNUPG:]` tokens before
each send). Combined with the `rotate_subkey` rollback (3.5), keyring integrity is
preserved on failure. A full rewrite using sequoia to construct the revocation signature
in Rust remains a future improvement.

**Files**: `src/gpg/keyring.rs`

---

### 3.4 — Post-migration YubiKey verification ⏳

**Problem**: after `keytocard + save`, no `--card-status` read to verify the correct
fingerprint landed in the expected slot.

**Status — blocked**: the verification requires the ability to reset a YubiKey slot in
a test environment (to validate the full migrate → read → compare cycle). Until that
tooling is available, automated testing is not possible.

**Planned implementation**: in `move_key_to_card()`, after `keytocard + save`, call
`card_status()` and compare the slot fingerprint against the migrated subkey's
fingerprint. Return an error if they don't match.

**Files**: `src/gpg/card.rs`

---

### 3.5 — `rotate_subkey` rollback on failure ✅

**Problem**: if `add_subkey` succeeded but `revoke_subkey_at_pos` failed, two valid
subkeys of the same type would coexist and potentially be published to keyservers.

**Fix**: before `add_subkey`, the secret key is exported to a `NamedTempFile`. If
revocation fails, the snapshot is automatically re-imported to restore the keyring. The
temp file is deleted in all cases (success, failure, panic).

**Files**: `src/gpg/keyring.rs`

---

## Phase 4 — Documentation and CI

### 4.1 — `cargo audit` in CI ✅

`cargo audit` added to `.github/workflows/ci.yml`. Non-blocking (advisories are reported
as warnings to avoid blocking builds on a CVE discovered after merge).

---

### 4.2 — `unsafe` block documented ✅

`// SAFETY:` comment added on `unsafe { NullPolicy::new() }` in `src/gpg/keyring.rs`:

```rust
// SAFETY: NullPolicy skips algorithm checks — used only to enumerate subkey metadata
// (read-only). Needed for legacy keys using SHA-1. Never used to verify signatures.
let policy = unsafe { NullPolicy::new() };
```

---

### 4.3 — Threat model document ✅

`THREAT_MODEL.md` written and committed at the repository root. Covers 10 threats for
key management and 8 threats for the PGP Chat subsystem (v0.6.0), with mitigations,
out-of-scope items, and security assumptions.

---

## Status summary

| # | Title | Status |
|---|-------|--------|
| 1.1 | Signer trust in signature verification | ✅ |
| 1.2 | HTTPS-only, 1 MiB cap, redirect limit | ✅ |
| 1.3 | Output files: no silent overwrite | ✅ |
| 2.1 | Fingerprint validation (40 hex, no Evil32) | ✅ |
| 2.2 | Sanitized error messages | ✅ |
| 2.3 | Clipboard auto-clear | ⚠️ out of scope (Wayland) |
| 3.1 | Clean gpg subprocess environment | ✅ |
| 3.2 | Consistent GNUPGHOME + `--homedir` | ✅ |
| 3.3 | Robust subkey revocation | ⚠️ partial (token-driven + rollback) |
| 3.4 | Post-migration YubiKey slot verification | ⏳ blocked (needs card reset tooling) |
| 3.5 | `rotate_subkey` rollback | ✅ |
| 4.1 | `cargo audit` in CI | ✅ |
| 4.2 | `SAFETY` comment on `unsafe` block | ✅ |
| 4.3 | `THREAT_MODEL.md` | ✅ |
