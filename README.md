# pgpilot

A desktop GUI for day-to-day PGP key management, built with [Tauri v2](https://v2.tauri.app/) (Rust backend) and [React 18](https://react.dev/) (TypeScript frontend), using [sequoia-openpgp](https://sequoia-pgp.org/) for key parsing.

pgpilot wraps GnuPG's command-line interface behind a themeable graphical interface. All cryptographic operations are performed by the `gpg` binary on your system; sequoia-openpgp is used solely to parse binary key exports.

[![CI](https://github.com/gfriloux/pgpilot/actions/workflows/ci.yml/badge.svg)](https://github.com/gfriloux/pgpilot/actions/workflows/ci.yml)
![License](https://img.shields.io/badge/license-Apache%202.0-blue)
![Rust](https://img.shields.io/badge/rust-2021%20edition-orange)
![Tauri](https://img.shields.io/badge/tauri-v2-blue)

---

## Features

**Key management**
- List your private and public keys in a master-detail panel
- Create new keys (Ed25519 cert-only primary + Sign / Encrypt / Auth subkeys)
- Import keys from a local file, a URL, a keyserver (keys.openpgp.org or keyserver.ubuntu.com), or pasted armored text
- Export a public key to a file, copy it to the clipboard, or upload it directly to keys.openpgp.org
- Backup a private key (exports secret key + revocation certificate into a folder of your choice)
- Delete keys (public, secret, or YubiKey stub)
- Set the owner-trust level (Undefined / Marginal / Full) per key

**Subkey management**
- View Sign, Encrypt, and Auth subkeys per key with expiry dates
- Add a missing subkey type
- Renew a subkey's expiry
- Rotate a subkey (create replacement + revoke old, marked Superseded)

**Keyserver**
- Publish to keys.openpgp.org or keyserver.ubuntu.com
- Per-key publication status badge (Unknown / Checking / Published / Not published)
- Auto-republish after subkey operations when the key was already published

**File operations**
- Encrypt files to one or more recipients (binary `.gpg` or armored `.asc`); warns when a recipient has insufficient trust and offers a force-trust override
- Decrypt files (checks whether a matching private key is available before attempting)
- Sign a file (produces a detached `.sig` alongside the original)
- Verify a signature (auto-detects `.sig`; reports Valid / Bad signature / Unknown key / Expired key / Revoked key)

**Diagnostics**
- GPG health page: checks across Installation, GPG Agent, and Security categories
- Checks include: GPG version, agent status, pinentry configuration, cache TTL values, `~/.gnupg` directory permissions, and presence of revocation certificates

**PGP Chat**
- End-to-end encrypted ephemeral chat via MQTT + OpenPGP
- Messages are encrypted with `gpg --encrypt --sign` for all room participants; only the intended recipients can read them
- The relay server sees only encrypted blobs — zero trust transport
- Rooms persist in `~/.config/pgpilot/rooms.yaml` (room IDs and participant fingerprints only); messages live in RAM only and are lost on exit
- Invite others via a signed join code (`pgpilot:join:...`) — the signature prevents broker-redirect attacks
- Presence indicators (● / ○) and per-recipient delivery receipts (✓ / ⏳)
- Click the lock icon on any message to see the sender's verified fingerprint, email, and trust level
- Works with software keys and YubiKey (gpg-agent handles PIN prompts and touch policy)
- Default relay: `mqtts://broker.hivemq.com:8883` (TLS; bring your own MQTT broker for production)

**YubiKey / smartcard**
- Detect connected card and display serial number
- Migrate private subkeys to a YubiKey

**UI**
- Two themes: **Catppuccin Frappé** (default, Mauve accent) and **USSR** (Soviet-inspired, Bebas Neue + Russo One fonts)
- English and French UI (auto-detected from system locale, configurable in Settings)
- Minimum window size 1000 × 540 px

---

## Prerequisites

- `gpg` 2.1 or later on your system (verified by the Diagnostic page at runtime)
- A working `gpg-agent` and `pinentry` program

> pgpilot does **not** ship a GnuPG binary. It invokes the `gpg` found in your `PATH`.

**Build dependencies** (Linux):

| Library | Purpose |
|---|---|
| `libclang` | sequoia-openpgp (nettle backend) — set `LIBCLANG_PATH` |
| `nettle`, `gmp` | sequoia-openpgp cryptographic backend |
| `webkit2gtk-4.1` | Tauri WebView (Linux) |
| `gtk3`, `libglib2.0`, `libsoup3` | Tauri system integration |
| `pkg-config` | dependency discovery |
| `cargo-tauri` | Tauri CLI build tool |
| `Node.js 22`, `npm` | frontend build (React + Vite) |

On NixOS, the Nix dev shell (see below) sets everything automatically.

---

## Building and running

### With the Nix dev shell (recommended on NixOS)

```bash
nix develop
just dev     # starts Vite + Tauri in dev mode (hot reload)
just build   # production bundle (.deb, .AppImage)
```

### With direnv

If you have [direnv](https://direnv.net/) and [nix-direnv](https://github.com/nix-community/nix-direnv):

```bash
direnv allow
just dev
```

### Nix flake check

```bash
nix flake check
```

This runs four checks: `alejandra` (Nix formatting), `deadnix` (dead Nix code), `rustfmt`, and `statix`.

---

## Development

| Command | Purpose |
|---|---|
| `just dev` | Start Tauri + Vite dev server (hot reload) |
| `just build` | Build production bundles |
| `just build-bin` | Build binary only (no .deb/.AppImage) |
| `just test` | Run Playwright E2E tests (mock mode, no binary needed) |
| `just screenshots` | Capture all screenshots in both themes |
| `cargo clippy -- -D warnings` | Lint Rust (warnings are errors) |
| `cargo fmt -- --config tab_spaces=2` | Format Rust (2-space tabs) |
| `cargo audit` | CVE scan |
| `pre-commit run --all-files` | Run all pre-commit checks |

**Code style:** Rust indentation is 2 spaces, enforced via `tab_spaces=2` in rustfmt.

The pre-commit hooks (alejandra, deadnix, rustfmt, clippy) are installed automatically the first time you enter the dev shell in a git working tree.

**Mock mode** (frontend dev without Tauri binary):

```bash
cd app && VITE_MOCK=true npm run dev
```

---

## Architecture overview

pgpilot is a Cargo workspace with two Rust crates:

```
pgpilot (lib, workspace root)   — GPG logic, chat engine, config
pgpilot-app (app/src-tauri/)    — Tauri v2 backend (36 #[tauri::command] functions)
```

The frontend (`app/src/`) is React 18 + TypeScript, built by Vite 6, communicating with the Rust backend via Tauri's IPC (`invoke()`).

```
src/                  — Rust library crate
├── config/           — Config struct (YAML, ~/ config)
├── gpg/              — all GPG operations (keyring, card, health)
└── chat/             — MQTT encrypted chat (rooms, crypto, wire, presence)

app/
├── src/              — React 18 + TypeScript frontend
│   ├── pages/        — route components
│   ├── components/   — reusable UI
│   ├── store/        — Zustand state slices
│   ├── ipc/          — typed invoke() wrappers
│   └── hooks/        — useAsync, useKeys, useChatEvents
└── src-tauri/        — Tauri Rust backend
    └── src/lib.rs    — 36 tauri::command functions
```

All GPG operations run via `gpg` subprocesses. Sequoia-openpgp is used only to parse the binary output of `gpg --export`.

---

## Configuration

### GnuPG location

pgpilot reads GnuPG from the standard location (`~/.gnupg`) unless the `GNUPGHOME` environment variable is set.

### Application preferences

pgpilot stores its preferences (language, theme) in `~/.config/pgpilot/config.yaml`. This file is created automatically on first launch:

```yaml
language: english   # or french
theme: catppuccin   # or ussr
```

This file does **not** contain any GPG keys or sensitive data. You can safely delete it to reset pgpilot to defaults.

---

## Packaging

The following assets are provided for downstream packaging:

| File | Install path |
|------|-------------|
| `share/applications/pgpilot.desktop` | `$out/share/applications/pgpilot.desktop` |
| `share/icons/hicolor/scalable/apps/pgpilot.svg` | `$out/share/icons/hicolor/scalable/apps/pgpilot.svg` |

---

## Roadmap

Planned / in progress:

- **Post-quantum cryptography** — blocked on stable GnuPG support.
- **Dashboard métriques** — home screen redesign with key stats.

---

## Contributing

1. Fork the repository and create a feature branch.
2. Enter the Nix dev shell: `nix develop`
3. Make your changes.
4. Run `pre-commit run --all-files` before committing.
5. Open a pull request with a clear description of what changed and why.

---

## License

Apache License, Version 2.0. See [LICENSE](LICENSE) for the full text.
