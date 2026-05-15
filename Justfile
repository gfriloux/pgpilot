dev:
    cd app && cargo-tauri dev

# Builds .deb and .rpm only — AppImage requires linuxdeploy which relies on a
# FHS-compatible system (/lib64/ld-linux-x86-64.so.2). Use CI (Ubuntu) for AppImage.
build:
    cd app && cargo-tauri build --bundles deb,rpm

build-bin:
    cd app && cargo-tauri build --no-bundle

test:
    cargo test --package pgpilot

test-all:
    cargo test --package pgpilot -- --ignored

fmt:
    cargo fmt -- --config tab_spaces=2
    cargo fmt --manifest-path app/src-tauri/Cargo.toml -- --config tab_spaces=2

check:
    cargo fmt --check -- --config tab_spaces=2
    cargo clippy -- -D warnings
    cargo test --package pgpilot
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
