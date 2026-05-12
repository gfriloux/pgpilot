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
