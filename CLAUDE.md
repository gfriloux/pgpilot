# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Before any code change, read [`DESIGN.md`](./DESIGN.md) then
> [`PROCEDURE_PLANS.md`](./PROCEDURE_PLANS.md). We don't code without a validated plan.**

## Guardrails (what does not change)

- **DESIGN.md is law.** Outside the invariants → no. Changing an invariant is an explicit
  DESIGN decision, not an implementation PLAN.
- **Git: hybrid.** Claude works on a **dedicated branch** (`feat/…`, `fix/…`, `chore/…`,
  `refactor/…`, `docs/…`), commits **atomically** (Conventional Commits, cf. PROCEDURE_PLANS.md
  §3), and **never** runs `merge`/`push`/`tag`. The user reviews, merges to `main`, pushes, tags.
  Each plan ends with a merge to `main`.
- **Quality gate: `just ci`** (fmt-check + clippy + Rust tests + E2E) is the single definition
  of the gates — pre-commit and CI call it. Every commit passes `just ci` on its own.
- **Doc in the same commit** as the code it describes.
- **Nix dev shell**: always `nix develop --command …` for non-interactive commands.

## Dev environment

This project requires a Nix dev shell. Always enter it before building:

```bash
nix develop
```

The shell sets `LIBCLANG_PATH`, `LD_LIBRARY_PATH`, and `GTK_USE_PORTAL=1` (required for KDE file dialogs via Tauri).

## Project structure

```
pgpilot/               ← Cargo workspace root + Rust library (GPG logic + chat)
├── src/
│   ├── lib.rs         — re-exports config, gpg, chat
│   ├── config/        — Config, Language, ThemeVariant (load/save YAML)
│   ├── gpg/           — all GPG operations (keyring, card, health, types)
│   └── chat/          — MQTT encrypted chat (rooms, crypto, wire, mqtt, presence)
│
app/                   ← Tauri v2 desktop app (the UI)
├── src/               — React 18 + TypeScript frontend
│   ├── layout/        — AppLayout (sidebar, routing)
│   ├── pages/         — MyKeys, PublicKeys, Encrypt, Decrypt, Sign, Verify,
│   │                    Chat, Health, Settings, CreateKey, Import
│   ├── components/    — KeyDetail, SubkeyCard, KeyListRow, UssrBanner, …
│   ├── store/         — Zustand slices: config, keys, ui, chat
│   ├── ipc/           — typed invoke() wrappers (keys.ts, chat.ts)
│   ├── hooks/         — useAsync, useKeys, useChatEvents
│   ├── styles/        — theme.css (CSS variables Catppuccin + USSR)
│   └── mock-tauri.ts  — VITE_MOCK=true stub for dev/tests without binary
│
└── src-tauri/         — Tauri Rust backend
    └── src/lib.rs     — 36 #[tauri::command] functions
│
docs/                  ← Astro + Starlight documentation site (EN + FR)
└── src/content/docs/  — root = English, fr/ = French
│
packages/pgpilot/      ← Nix package derivation (build + install via home-manager)
modules/home/pgpilot/  ← Nix home-manager module (pgpilot.pgpilot.enable)
```

## Commands

### Library (src/)
```bash
cargo build                              # compile workspace (lib + Tauri backend)
cargo clippy -- -D warnings             # lint
cargo fmt -- --config tab_spaces=2      # format (2-space tabs)
cargo audit                             # CVE scan
cargo test --package pgpilot --lib      # unit tests
cargo test --package pgpilot -- --ignored  # + slow GPG integration tests
```

### Quality gate (src/ + app/)
```bash
just ci         # full gate: fmt-check + clippy + Rust tests + E2E (pre-commit + CI call this)
just fmt        # format in place (Rust lib + Tauri backend)
just fmt-check  # verify formatting, fails if unformatted
just lint       # clippy -D warnings
just test       # fast Rust unit tests
just test-all   # + slow GPG integration tests (--ignored, ~30 s)
just e2e         # Playwright E2E (VITE_MOCK=true, no binary needed)
just audit      # cargo audit (CVE scan, non-blocking in CI)
```

### Tauri app (app/)
```bash
just dev        # cargo-tauri dev (starts Vite + Tauri)
just build      # cargo-tauri build --bundles deb,rpm
just build-bin  # cargo-tauri build --no-bundle        (binary only, fastest)

# Dev without binary (mock mode):
cd app && VITE_MOCK=true npm run dev
```

### Docs (docs/)
```bash
just docs-dev   # cd docs && npm run dev
cd docs && npm run build
```

Indentation is **2 spaces** (configured via `tab_spaces=2` in rustfmt).

## CI/CD

- **CI** (`.github/workflows/ci.yml`) — push/PR to `main`:
  `cargo fmt` → `cargo clippy -D warnings` → `cargo build` → `cargo test --package pgpilot --lib` → `cargo test --package pgpilot -- --ignored` → `cargo audit` (non-blocking, known CVEs ignored via `--ignore`)

- **Docs** (`.github/workflows/docs.yml`) — push to `main` touching `docs/**`:
  `npm install && npm run build` → deploy to GitHub Pages via `actions/deploy-pages`

- **Release** (`.github/workflows/release.yml`) — on `v*` tag:
  `cargo-tauri build` → generate RELEASE_NOTES.md with git-cliff → update CHANGELOG.md → GitHub Release with `.deb` + `.rpm`

To publish a release:
```bash
git tag v1.2.3
git push --tags
```

## Testing

```bash
# Rust library
cargo test --package pgpilot            # fast unit tests
cargo test --package pgpilot -- --ignored  # + slow GPG integration tests (~30 s)
cargo test --package pgpilot -- --nocapture  # with stdout logs

# E2E (Playwright, no Tauri binary needed)
cd app && npm run test:e2e
```

### Test structure
- `tests/common/` — shared helpers (`setup_test_gnupghome`, `import_armored`)
- `tests/fixtures/` — pre-generated PGP test keys (armored, no passphrase)
- `tests/gpg_keyring.rs` — keyring integration tests (real gpg processes)
- `tests/gpg_card.rs` — smartcard smoke tests (graceful no-card behavior)
- `tests/chat_*.rs` — chat unit tests (wire, rooms, crypto, MQTT, presence)
- `app/tests/` — Playwright E2E (90 tests, 8 suites)

### Conventions
- Slow tests → mark `#[ignore]`
- Always use `setup_test_gnupghome()` — never use real `$GNUPGHOME`
- `TempDir` must stay in scope for the full test duration
- No GPG mocks — real gpg processes in a temp homedir
- Tests needing a physical YubiKey → mark `#[ignore]` with comment

## Architecture

### Library crate (`src/`)

Features:
- `default = ["chat"]` — chat enabled by default
- `chat` — enables rumqttc, rustls, webpki-roots, async-trait

**`config/`** — `Config` struct (language, theme, scale_factor, mqtt_default_relay, chat_local_fp). Loaded from `~/.config/pgpilot/config.yaml` via serde_yaml. `Language` (English/French) and `ThemeVariant` (Catppuccin/Ussr) defined here.

**`gpg/`** — all cryptographic operations delegated to the `gpg` binary as subprocesses. Sequoia-openpgp used only for parsing binary export output.
- `mod.rs` — `gnupg_dir()`, `gpg_command()`, `display_path()`, `sanitize_gpg_stderr()`, `gnupg_homedir()` (pub, for Tauri)
- `keyring.rs` — create, export, import, delete, publish, encrypt, sign, verify, backup, subkey ops. `validate_fp()` rejects anything not exactly 40 ASCII hex chars. `validate_keyserver_url()` whitelist: `keys.openpgp.org`, `keyserver.ubuntu.com`.
- `card.rs` — YubiKey/smartcard operations
- `health.rs` — diagnostic checks (4 statuses: Ok/Info/Warning/Error)
- `types.rs` — KeyInfo, SubkeyInfo, CardInfo, TrustLevel, VerifyResult, VerifyOutcome (all Serialize/Deserialize for Tauri IPC)

**`chat/`** — E2E encrypted chat via MQTT + GPG subprocesses.
- Transport: rumqttc 0.25 over TLS (rustls 0.23 + webpki-roots bundle)
- Crypto: gpg --encrypt --sign / gpg --decrypt subprocesses
- Messages ephemeral (RAM only, 500 max per room via VecDeque FIFO)
- Rooms persist in `~/.config/pgpilot/rooms.yaml`
- `mqtt.rs` — `MqttHandle` (owns tokio task), `MqttConfig`, `MqttEvent`, `MqttCmd`; communicates via mpsc channels

### Tauri backend (`app/src-tauri/src/lib.rs`)

36 `#[tauri::command]` functions, all spawning blocking gpg operations via `spawn_blocking`. Events sent to frontend via `AppHandle::emit()`.

Security guards applied at every command entry point:
- `validate_fp()` — fingerprint format check
- `validate_keyserver_url()` — URL whitelist
- `canonicalize()` — path traversal prevention

Chat state managed as `ChatState(tokio::Mutex<Option<ChatSession>>)` in Tauri state.

### Frontend (`app/src/`)

**Tech**: React 18 + TypeScript strict + Vite 6 + CSS Modules + Zustand + React Router v6

**Theming**: CSS custom properties in `styles/theme.css`. Two themes: Catppuccin Frappé (default, dark) and USSR (Soviet-inspired, cream content). Theme applied by swapping a `data-theme` attribute on `<html>`. No Tailwind.

**Mock mode**: `VITE_MOCK=true` aliases `@tauri-apps/api/core` → `mock-tauri.ts`. All 36 commands have mock implementations returning realistic data. Used by Playwright E2E and screenshot script.

**IPC pattern**: direct `invoke()` calls (no tauri-specta). Types defined by hand in `src/types/ipc.ts` mirroring Rust structs.

**USSR theme assets**: propaganda banners as PNG files in `public/banners/`. Banners clip to card border-radius via `overflow: hidden` on cards + banner outside padded content wrapper.

**Real-time chat**: `AppHandle::emit()` on Rust side → `listen()` in `useChatEvents` hook. Hook called once in `AppLayout` for app lifetime. StrictMode double-registration fixed by storing `Promise<unlisten>[]` in cleanup.

### Documentation (`docs/`)

Astro 5 + Starlight 0.34, bilingual EN (root locale) + FR (`/fr/` prefix).

Config: Astro 6 + Starlight 0.39. Content uses the new Content Layer API via `docsLoader()`.
Content: `src/content.config.ts` with `docsLoader()` + `docsSchema()` (Astro 6 content layer).
Screenshots: `public/screenshots/` — 18 PNG captures (9 views × 2 themes), generated by `app/scripts/screenshots.mjs` (NixOS-aware Chromium path detection).
Deploy: GitHub Actions → `actions/deploy-pages` (GitHub Pages → "GitHub Actions" source must be set in repo settings).

## Conventions

### GPG layer
- Always use `gpg_command(homedir)` — never `Command::new("gpg")`
- Call `validate_fp(fp)` as first line of any function taking a fingerprint
- Use `safe_get(url)` for HTTP GET (HTTPS-only, 1 MiB cap, 3 redirects, timeouts)
- `backup_key` uses `canonicalize()` + `0o600` permissions on secret key
- `export_public_key` uses `OpenOptions::new().create_new(true)` (no silent overwrite)

### Tauri commands
- All blocking GPG ops: `spawn_blocking(move || ...)`
- Error type: `Result<T, String>` — no bare panics
- Chat commands: validate fp with `validate_fp()` before use

### Frontend
- `exactOptionalPropertyTypes` enabled — use spread pattern for optional props:
  `{...(value !== undefined ? { prop: value } : {})}`
- Zustand selectors: never return new array inside selector (`?? []` outside)
- `onChange` on `Input` component: `(e) => setState(e.currentTarget.value)` (ChangeEventHandler)

## Nix packaging

### Installation via home-manager

pgpilot exposes a home-manager module. Users add it to their NixOS flake:

```nix
inputs.pgpilot.url = "github:gfriloux/pgpilot/v0.8.7";

# home-manager config:
imports = [ inputs.pgpilot.homeModules.pgpilot ];
pgpilot.pgpilot.enable = true;
```

### Updating `npmDepsHash` after `app/package-lock.json` changes

The Nix package pre-fetches npm deps offline. After any change to `app/package-lock.json`, recompute the hash:

```bash
nix run nixpkgs#prefetch-npm-deps -- app/package-lock.json
```

Then update `npmDepsHash` in `packages/pgpilot/default.nix`.

## Known issues / backlog

- **rustls-webpki CVEs** (RUSTSEC-2026-{0098,0099,0104,0049}): `rumqttc 0.25.1` has a direct dep on `rustls-webpki 0.102.x`. Cannot fix without upstream rumqttc release. Ignored in CI via `--ignore`.
- **YubiKey post-migration verification** (SECURITY_PLAN.md §3.4): blocked on ability to delete a key from YubiKey slot for testing.
- **Post-quantum cryptography**: blocked on GnuPG stable PQC support.
- **Dashboard métriques**: home screen redesign with key stats (count, expiring, published, on YubiKey).
- **Import clef publique via file picker** : la page Import propose `import_key_file` mais le bouton file picker n'est pas implémenté côté frontend — l'utilisateur ne peut importer un fichier `.asc`/`.gpg` que via copier-coller de texte ou URL, pas depuis le système de fichiers.

## Future PLANs (from DESIGN.md)

These items emerge from the rules established in `DESIGN.md` and require dedicated PLANs.

- **PLAN: Conformité modèle de clef** — Enforcer les règles DESIGN.md à la création :
  bloquer expiration > 2 ans (sous-clefs) / > 3 ans (primaire) ; imposer les 3 sous-clefs
  `[S]` `[E]` `[A]` ; interdire RSA à la création ; refuser import RSA < 2048 bits.

- **PLAN: Certificat de révocation** — Générer et exporter le certificat de révocation
  à la création de chaque clef primaire ; bloquer la fin du wizard si non exporté ;
  mémoriser et afficher l'emplacement dans la vue clef.

- **PLAN: Diagnostics DESIGN.md (Health)** — Remonter dans la page Health :
  clefs sans expiration (erreur), clefs RSA legacy (warning + incitation rotation),
  sous-clefs manquantes, clef primaire en keyring local vs YubiKey (info).

- **PLAN: Indicateur YubiKey** — Afficher un indicateur visible dans la sidebar si une
  YubiKey est détectée ; mettre en avant l'état "optimal" quand la primaire est sur YubiKey ;
  badge informatif si elle est uniquement en keyring local.

- **PLAN: Rotation clef primaire** — Wizard guidé pas-à-pas : cross-certification,
  déclaration de transition, export des sous-clefs secrètes avant archivage, état "archivée"
  en lecture seule, badge période de grâce, flux urgent compromission.

- **PLAN: Docs rotation** — Documentation bilingue EN/FR sur le cycle de vie et la
  rotation de clef primaire, à destination des débutants (site Astro/Starlight).

- **PLAN: Keyoxide — identité décentralisée** — Lire et afficher les notations d'identité
  présentes sur une clef (preuves GitHub, GitLab, Mastodon, domaine perso, etc.) ; wizard
  pour ajouter une notation `proof@ariadne.id=<url>` sur sa propre clef et guider
  l'utilisateur pour créer la preuve correspondante (gist signé, post Mastodon…) ;
  lien direct vers le profil keyoxide.org.

- **PLAN: DANE — publication DNS** — Générer l'enregistrement DNS `OPENPGPKEY` (RFC 7929)
  prêt à copier pour les utilisateurs possédant un domaine avec DNSSEC ; afficher les
  instructions par registrar courants ; vérifier si un enregistrement DANE existe déjà
  pour une clef donnée (lookup DNS).
