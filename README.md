# pgpilot

A desktop GUI for day-to-day PGP key management, built with [iced](https://github.com/iced-rs/iced) 0.14 and [sequoia-openpgp](https://sequoia-pgp.org/) 2.

pgpilot wraps GnuPG's command-line interface behind a keyboard-friendly, themeable interface. All cryptographic operations are performed by the `gpg` binary on your system; sequoia-openpgp is used solely to parse binary key exports.

[![CI](https://github.com/gfriloux/pgpilot/actions/workflows/ci.yml/badge.svg)](https://github.com/gfriloux/pgpilot/actions/workflows/ci.yml)
![License](https://img.shields.io/badge/license-Apache%202.0-blue)
![Rust](https://img.shields.io/badge/rust-2021%20edition-orange)
![iced](https://img.shields.io/badge/iced-0.14-purple)

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
- File drag-and-drop onto the Encrypt and Decrypt views (X11; Wayland support depends on compositor)

**Diagnostics**
- GPG health page: 8 checks across Installation, GPG Agent, and Security categories
- Checks include: GPG version, agent status, pinentry configuration, cache TTL values, `~/.gnupg` directory permissions, and presence of revocation certificates

**YubiKey / smartcard**
- Detect connected card and display serial number
- Migrate private subkeys to a YubiKey

**UI**
- Catppuccin Frappé theme with Mauve accent
- Dark sidebar with Nerd Font icons
- Timed status bar (auto-dismisses after 4 seconds)
- Minimum window size 1000 × 540 px

---

## Prerequisites

- Rust toolchain (stable, 2021 edition) — install via [rustup](https://rustup.rs/)
- `gpg` 2.1 or later on your system (verified by the Diagnostic page at runtime)
- A working `gpg-agent` and `pinentry` program

> pgpilot does **not** ship a GnuPG binary. It invokes the `gpg` found in your `PATH`.

**System libraries required at build time** (Linux):

| Library | Purpose |
|---|---|
| `libclang` | sequoia-openpgp (nettle backend) — set `LIBCLANG_PATH` |
| `nettle`, `gmp` | sequoia-openpgp cryptographic backend |
| `pkg-config` | dependency discovery |
| `wayland`, `libxkbcommon`, `libGL`, `vulkan-loader` | iced rendering |
| `gtk3` | native file dialog (rfd) |
| `dbus` | rfd file dialog on NixOS / some DEs — must be in `LD_LIBRARY_PATH` |

On most distributions these are available through your package manager. The Nix dev shell (see below) sets everything automatically.

---

## Building and running

### Standard (cargo)

```bash
cargo build
cargo run
```

Make sure the system libraries listed under [Prerequisites](#prerequisites) are installed and that `LIBCLANG_PATH` points to your libclang installation before building.

### With the Nix dev shell (recommended on NixOS)

The dev shell sets `LIBCLANG_PATH` and `LD_LIBRARY_PATH` automatically so that sequoia-openpgp and iced can find their native dependencies:

```bash
nix develop
cargo build
cargo run
```

### With direnv

If you have [direnv](https://direnv.net/) and [nix-direnv](https://github.com/nix-community/nix-direnv):

```bash
direnv allow
```

The shell is then entered automatically when you `cd` into the project.

### Nix flake check

```bash
nix flake check
```

This runs four checks: `alejandra` (Nix formatting), `deadnix` (dead Nix code), `rustfmt`, and `statix`.

---

## Development

| Command | Purpose |
|---|---|
| `cargo build` | Compile |
| `cargo run` | Run the application |
| `cargo clippy -- -D warnings` | Lint (warnings are errors) |
| `cargo fmt -- --config tab_spaces=2` | Format (2-space tabs) |
| `cargo audit` | CVE scan |
| `pre-commit run --all-files` | Run all pre-commit checks |

**Code style:** indentation is 2 spaces, enforced via `tab_spaces=2` in rustfmt.

The pre-commit hooks (alejandra, deadnix, rustfmt, clippy) are installed automatically the first time you enter the dev shell in a git working tree.

---

## Architecture overview

pgpilot uses the iced 0.14 elm-like `application` API. There is no `Sandbox` or `Application` trait — functions are passed directly:

```rust
iced::application(App::new, App::update, App::view)
    .title("pgpilot")
    .run()
```

All blocking work (gpg subprocesses) runs through `tokio::task::spawn_blocking` wrapped in `Task::perform`. File dialogs use `rfd::AsyncFileDialog` in async `Task::perform` blocks.

```
src/
├── main.rs          — entry point; wires file-drop subscription
├── app/             — App struct, Message enum, update() router, per-domain handlers
├── gpg/             — GPG layer (keyring, card, health checks, types)
└── ui/              — views (key list, key detail, create, import, encrypt, sign, verify, health…)
```

The GPG layer (`src/gpg/`) calls the `gpg` binary directly for all mutations. Sequoia-openpgp is used only to parse the binary output of `gpg --export`.

All state that references a key uses the full 40-hex fingerprint, never a list index.

---

## Configuration

pgpilot reads GnuPG from the standard location (`~/.gnupg`) unless the `GNUPGHOME` environment variable is set:

```bash
GNUPGHOME=/path/to/custom/gnupg cargo run
```

No other configuration file is required or supported.

---

## NixOS / Wayland note

On NixOS, `pkgs.dbus` must be present in `LD_LIBRARY_PATH` for the native file dialog (`rfd` 0.17) to work. The dev shell (`shells/default/default.nix`) sets this automatically. If you run a pre-built binary outside the dev shell, ensure `libdbus` is in your library path.

---

## Roadmap

The following items are implemented in the current release (v0.2.0):

- Key listing and detail panel
- Export, backup, import, delete
- Create key (Ed25519 + subkeys)
- Subkey management: add, renew, rotate
- Keyserver publish and publication status
- YubiKey migration
- File encryption and decryption
- File signing and signature verification
- Trust level management
- GPG health diagnostics

Planned / in progress:

- **Post-quantum cryptography** — blocked on stable GnuPG support. When GnuPG ships stable PQC, the plan is to add composite schemes per draft-ietf-openpgp-pqc: Dilithium3+Ed25519 (signing) and ML-KEM-768+X25519 (encryption).

---

## Contributing

1. Fork the repository and create a feature branch.
2. Ensure the system libraries listed under [Prerequisites](#prerequisites) are available (or enter the Nix dev shell with `nix develop`).
3. Make your changes. Keep handlers in the appropriate `app/<domain>.rs` submodule and return `Task<Message>` from every handler.
4. Run `pre-commit run --all-files` before committing. The hooks enforce formatting and linting.
5. Open a pull request with a clear description of what changed and why.

---

## License

Apache License, Version 2.0. See [LICENSE](LICENSE) for the full text.
