dev:
    cd app && cargo-tauri dev

build:
    cd app && cargo-tauri build

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
