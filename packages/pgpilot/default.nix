{
  pkgs,
  lib,
  inputs,
  ...
}:
# pgpilot — Tauri v2 + React 18 desktop app.
#
# Build split into two phases:
#   1. frontend  — npm/Vite build of the React app (produces dist/)
#   2. backend   — rustPlatform.buildRustPackage for the Tauri crate,
#                  with tauri.conf.json patched to embed the pre-built dist/
#
# The final binary is self-contained: Tauri embeds the frontend at compile time.
# Runtime deps: webkit2gtk (WebView), gtk3, libsoup3.
#
# To recompute npmDepsHash after any change to app/package-lock.json:
#   nix run nixpkgs#prefetch-npm-deps -- app/package-lock.json
let
  pname = "pgpilot";
  version = "0.8.0";
  src = inputs.self;

  frontend = pkgs.buildNpmPackage {
    pname = "pgpilot-frontend";
    inherit version;
    src = src + "/app";

    npmDepsHash = "sha256-ZgPg7QvUiWuEJW96T3hCp0/fPiV1FrOWE2a+LAszqns=";

    buildPhase = "npm run build";
    installPhase = "cp -r dist $out";
  };
in
  pkgs.rustPlatform.buildRustPackage {
    inherit pname version src;

    cargoLock.lockFile = "${src}/Cargo.lock";

    cargoBuildFlags = ["--package" "pgpilot-app"];

    # Tauri integration tests require a display — skip in sandbox
    doCheck = false;

    env.LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

    nativeBuildInputs = with pkgs; [
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

    # Patch tauri.conf.json to use the pre-built frontend and skip the npm step
    preBuild = ''
      substituteInPlace app/src-tauri/tauri.conf.json \
        --replace-fail '"beforeBuildCommand": "npm run build"' '"beforeBuildCommand": ""' \
        --replace-fail '"frontendDist": "../dist"' '"frontendDist": "${frontend}"'
    '';

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
