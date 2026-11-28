# Tests manuels — v0.10.2

Version de correction : build et lancement sur nixpkgs 26.11, mises à jour de sécurité
de la pile TLS. Aucune fonctionnalité ajoutée. Ces tests valident que l'application
installée par Nix démarre et que les chemins réseau (chat, keyserver) fonctionnent
toujours après la mise à jour de `rustls`.

Lancer le binaire avec `env -u LD_LIBRARY_PATH` : un `LD_LIBRARY_PATH` hérité fausse le
résultat (cf. plan, correctif `EGL_BAD_PARAMETER`).

## 1. Portes qualité automatisées

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | `nix develop --command just ci` | Vert : fmt-check + clippy + tests Rust + 92 E2E | ☐ |
| 2 | `nix build` | Produit `./result/bin/pgpilot-app`, aucune erreur `aws-lc-sys` | ☐ |
| 3 | `nix flake check` | `all checks passed!` | ☐ |
| 4 | `cargo audit` avec la liste d'exclusion de `ci.yml` | 0 vulnérabilité, 8 avertissements | ☐ |

## 2. Lancement du binaire Nix

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | `env -u LD_LIBRARY_PATH ./result/bin/pgpilot-app` | Fenêtre PGPilot avec l'interface affichée, pas de fenêtre vide | ☐ |
| 2 | Lire la sortie du terminal | Aucune ligne `EGL_BAD_PARAMETER`, aucune ligne `GLIBC_… not found` | ☐ |
| 3 | Version affichée dans la sidebar | `0.10.2` | ☐ |
| 4 | Basculer Catppuccin ↔ USSR | Les deux thèmes s'affichent | ☐ |

## 3. Chemins TLS (rustls 0.23.46, webpki-roots 1.0.12)

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | Chat : rejoindre une room sur le relais par défaut | Connexion établie, présence visible | ☐ |
| 2 | Chat : envoyer un message et le recevoir depuis un second client | Message déchiffré des deux côtés | ☐ |
| 3 | Import : recherche d'une clef sur `keys.openpgp.org` | Résultat renvoyé, pas d'erreur de certificat | ☐ |
| 4 | Publier une clef publique sur `keys.openpgp.org` | Toast de succès | ☐ |

## 4. Dialogues de fichier (KDE/Plasma)

Le nixpkgs change de sept mois : GTK, WebKitGTK et le portail sont tous mis à jour.

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | Encrypt : choisir un fichier d'entrée | Le dialogue natif KDE s'ouvre | ☐ |
| 2 | Backup d'une clef vers un dossier choisi | Fichier écrit, permissions `0600` | ☐ |

## 5. Installation home-manager

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | Pointer l'input du flake système sur la branche puis `nixos-rebuild` / `home-manager switch` | Build sans erreur, y compris avec `inputs.nixpkgs.follows` | ☐ |
| 2 | Lancer PGPilot depuis le menu d'applications | L'application démarre | ☐ |
