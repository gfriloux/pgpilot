{
  pkgs,
  mkShell,
  ...
}:
mkShell {
  packages = with pkgs; [
    alejandra
    deadnix
    statix
    pre-commit
    cargo
    rustc
    clippy
    rustfmt
    rust-analyzer
    cargo-audit
    git-cliff
    just
    linuxdeploy

    # sequoia-openpgp (nettle backend)
    clang
    nettle
    pkg-config
    gmp

    # Wayland/X11 libs (Tauri WebKit)
    wayland
    libxkbcommon
    libGL
    vulkan-loader

    # rfd + Tauri (native file dialog + WebKit)
    gtk3

    # Tauri v2
    cargo-tauri
    nodejs_22
    xdotool

    # gpg (for testing)
    gnupg

    # CA bundle — requis pour rustls-native-certs (connexions TLS MQTT, HTTPS)
    cacert
  ];

  buildInputs = with pkgs; [
    # Tauri v2 — stack GTK/WebKit requise sur Linux
    dbus
    openssl
    glib
    glib-networking
    webkitgtk_4_1
    libsoup_3
    cairo
    pango
    gdk-pixbuf
    atk
    librsvg
  ];

  shellHook = ''
        export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
        export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"

        # Sur KDE/Plasma, GTK doit déléguer les dialogues fichier à xdg-desktop-portal-kde
        # (KDialog natif) plutôt que d'utiliser GtkFileChooser qui exige les schémas GNOME.
        export GTK_USE_PORTAL=1
        export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath (with pkgs; [
      wayland
      libxkbcommon
      libGL
      vulkan-loader
      dbus
      openssl
      glib
      gtk3
      webkitgtk_4_1
      libsoup_3
      cairo
      pango
      gdk-pixbuf
      atk
      librsvg
    ])}:$LD_LIBRARY_PATH

        echo "[pgpilot] Ready."
        echo "  cargo build          — build library (gpg + chat)"
        echo "  just dev             — cargo-tauri dev"
        echo "  just build           — cargo-tauri build"

        if [ ! -f .pre-commit-config.yaml ]; then
          echo "Generating .pre-commit-config.yaml..."
          cat > .pre-commit-config.yaml <<'EOF'
    ---
    repos:
      - repo: local
        hooks:
          - id: alejandra
            name: alejandra
            language: system
            entry: alejandra --check
            files: \.nix$
            pass_filenames: true
          - id: deadnix
            name: deadnix
            language: system
            entry: deadnix --fail
            files: \.nix$
            pass_filenames: true
          - id: rustfmt
            name: rustfmt
            language: system
            entry: cargo fmt -- --check --config tab_spaces=2
            files: \.rs$
            pass_filenames: false
          - id: clippy
            name: clippy
            language: system
            entry: cargo clippy -- -D warnings
            files: \.rs$
            pass_filenames: false
    EOF
        else
          echo ".pre-commit-config.yaml already exists. Skipping generation."
        fi

        if [ -d .git ]; then
          if [ ! -f .git/hooks/pre-commit ]; then
            echo "Installing pre-commit hook..."
            pre-commit install -f --install-hooks
          fi
        else
          echo "Not a git repository. Skipping pre-commit installation."
        fi
  '';
}
