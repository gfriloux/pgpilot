{
  pkgs,
  lib,
  inputs,
  ...
}:
# pgpilot — Tauri v2 + React 18 desktop app.
#
# Build uses cargo-tauri.hook (nixpkgs standard for Tauri v2 apps) which:
#   - Runs `cargo tauri build` instead of plain `cargo build`
#   - Sets TAURI_ENV_DEBUG=false → production binary (no dev server connection)
#   - Handles frontend embedding correctly
#
# npm deps are provided offline via fetchNpmDeps + npmHooks.npmConfigHook.
# The `beforeBuildCommand` in tauri.conf.json runs `npm run build` which
# succeeds because npmConfigHook pre-configured the offline registry.
#
# To recompute npmDepsHash after any change to app/package-lock.json:
#   nix run nixpkgs#prefetch-npm-deps -- app/package-lock.json
let
  pname = "pgpilot";
  version = "0.8.0";
  src = inputs.self;
in
  pkgs.rustPlatform.buildRustPackage {
    inherit pname version src;

    cargoLock.lockFile = "${src}/Cargo.lock";

    # npm deps for the React frontend
    # cargo-tauri.hook pushd's to tauriRoot before running `cargo tauri build`.
    # Without this, the hook runs from the workspace root where there is no
    # src-tauri/, causing Tauri CLI to search the tree and find docs/package.json
    # instead of app/package.json.
    tauriRoot = "app";

    npmDeps = pkgs.fetchNpmDeps {
      src = src + "/app";
      hash = "sha256-ZgPg7QvUiWuEJW96T3hCp0/fPiV1FrOWE2a+LAszqns=";
    };
    npmRoot = "app";

    # No tests: Tauri integration tests require a display
    doCheck = false;

    env.LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

    nativeBuildInputs = with pkgs; [
      cargo-tauri.hook # Runs `cargo tauri build`, sets TAURI_ENV_DEBUG=false
      nodejs
      npmHooks.npmConfigHook # Feeds offline npm deps to the build
      pkg-config
      clang
      wrapGAppsHook4
      gobject-introspection
    ];

    buildInputs = with pkgs; [
      webkitgtk_4_1
      gtk3
      glib
      glib-networking
      libsoup_3
      cairo
      pango
      gdk-pixbuf
      atk
      librsvg
      dbus
      openssl
      nettle
      gmp
    ];

    postInstall = ''
      install -Dm644 share/applications/pgpilot.desktop \
        $out/share/applications/pgpilot.desktop
      install -Dm644 share/icons/hicolor/scalable/apps/pgpilot.svg \
        $out/share/icons/hicolor/scalable/apps/pgpilot.svg
    '';

    meta = {
      description = "PGP key manager GUI built with Tauri and React";
      homepage = "https://github.com/gfriloux/pgpilot";
      license = lib.licenses.asl20;
      platforms = lib.platforms.linux;
      mainProgram = "pgpilot-app";
    };
  }
