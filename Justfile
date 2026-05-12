dev:
    cd tauri-app && cargo-tauri dev

build:
    cd tauri-app && cargo-tauri build

build-bin:
    cd tauri-app && cargo-tauri build --no-bundle

test:
    cargo test

test-all:
    cargo test -- --ignored

fmt:
    cargo fmt -- --config tab_spaces=2
    cd tauri-app && cargo fmt --manifest-path src-tauri/Cargo.toml -- --config tab_spaces=2

check:
    cargo fmt --check -- --config tab_spaces=2
    cargo clippy -- -D warnings
    cargo test
    cargo audit

screenshots:
    cd tauri-app && VITE_MOCK=true node scripts/screenshots.mjs

docs-dev:
    cd docs && npm run dev

docs-build:
    cd docs && npm run build
