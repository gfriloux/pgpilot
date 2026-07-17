default:
    @just --list

dev:
    cd app && cargo-tauri dev

# Builds the .deb and .rpm bundles (the supported distribution artifacts).
build:
    cd app && cargo-tauri build --bundles deb,rpm

build-bin:
    cd app && cargo-tauri build --no-bundle

# Full quality gate (fmt-check + lint + Rust tests + E2E) — single source of truth, run before every commit.
ci: fmt-check lint test e2e

# Format in place (Rust lib + Tauri backend).
fmt:
    cargo fmt -- --config tab_spaces=2
    cargo fmt --manifest-path app/src-tauri/Cargo.toml -- --config tab_spaces=2

# Verify formatting without modifying — fails if anything is unformatted.
fmt-check:
    cargo fmt --check -- --config tab_spaces=2
    cargo fmt --manifest-path app/src-tauri/Cargo.toml --check -- --config tab_spaces=2

# Static lint (clippy). No warning tolerated.
lint:
    cargo clippy -- -D warnings

# Fast Rust unit tests.
test:
    cargo test --package pgpilot

# + slow GPG integration tests (real gpg, ~30 s).
test-all:
    cargo test --package pgpilot -- --ignored

# Playwright E2E (VITE_MOCK=true, no Tauri binary needed).
e2e:
    cd app && npm run test:e2e

# CVE scan (non-blocking in CI: known CVEs ignored).
audit:
    cargo audit

screenshots:
    cd app && VITE_MOCK=true node scripts/screenshots.mjs

docs-dev:
    cd docs && npm run dev

docs-build:
    cd docs && npm run build

# Recompute npmDepsHash and update packages/pgpilot/default.nix
update-nix-hash:
    #!/usr/bin/env bash
    set -euo pipefail
    hash=$(nix run nixpkgs#prefetch-npm-deps -- app/package-lock.json)
    sed -i 's|hash = "sha256-[^"]*"|hash = "'"${hash}"'"|' packages/pgpilot/default.nix
    echo "npmDepsHash updated: ${hash}"

# Bump version everywhere and update Nix hash — usage: just release 0.8.8
release VERSION:
    #!/usr/bin/env bash
    set -euo pipefail
    v="{{VERSION}}"
    sed -i "s/^version = \"[0-9.]*\"/version = \"${v}\"/" Cargo.toml
    sed -i "s/^version = \"[0-9.]*\"/version = \"${v}\"/" app/src-tauri/Cargo.toml
    sed -i "s/\"version\": \"[0-9.]*\"/\"version\": \"${v}\"/" app/package.json
    sed -i "s/\"version\": \"[0-9.]*\"/\"version\": \"${v}\"/" app/src-tauri/tauri.conf.json
    sed -i "s/version = \"[0-9.]*\";/version = \"${v}\";/" packages/pgpilot/default.nix
    sed -i "1,15s/\"version\": \"[0-9.]*\"/\"version\": \"${v}\"/" app/package-lock.json
    just update-nix-hash
    echo "Done — review changes then: git add -A && git commit -m 'chore(release): v${v}' && git tag v${v} && git push && git push --tags"
