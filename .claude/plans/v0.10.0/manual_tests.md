# Tests manuels — v0.10.0

Version de maintenance : retrait de l'AppImage + consolidation des 10 MAJ Renovate.
Aucune fonctionnalité utilisateur ajoutée. Ces tests valident la distribution et la
non-régression après les bumps de dépendances.

## 1. Portes qualité automatisées

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | `nix develop --command just ci` | Vert : fmt-check + clippy + tests Rust + 92 E2E | ☐ |
| 2 | `nix build && ./result/bin/pgpilot-app` | L'app démarre, fenêtre PGPilot, aucun crash EGL | ☐ |
| 3 | `nix flake check` | `all checks passed!` (le devShell `appimage` a bien disparu) | ☐ |
| 4 | `nix develop --command cargo audit` | Seules des advisories connues/ignorées (rustls-webpki, rustls-pemfile, quick-xml build-time, unmaintained gtk3) — aucune nouvelle vulnérabilité runtime | ☐ |

> Note NixOS E2E : Playwright a besoin d'un Chromium *wrappé* dans `/nix/store`
> (`nix build nixpkgs#chromium` une fois). Sinon `just ci` échoue sur l'E2E avec
> « NixOS cannot run dynamically linked executables ». Voir `phase0_results.md`.

## 2. Retrait de l'AppImage

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | `nix develop --command just build` | Produit `.deb` + `.rpm` dans `target/release/bundle/`, **aucun** dossier `appimage/`, aucune erreur linuxdeploy | ☐ |
| 2 | `grep -rniE appimage . ` (hors `.claude/plans/`, `node_modules`, artefacts) | Aucune référence AppImage active dans code/CI/docs | ☐ |
| 3 | Relire `.github/workflows/release.yml` | Plus de steps « Install system dependencies for AppImage » / « Build AppImage » ; le step Release n'upload que `deb` + `rpm` | ☐ |
| 4 | (au tag, côté CI) Vérifier la release GitHub | Assets = `.deb` + `.rpm` uniquement | ☐ |

## 3. Non-régression après MAJ dépendances

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | `cd app && VITE_MOCK=true npm run dev` puis ouvrir http://localhost:1421 | L'app se charge (vite 8.1.5) | ☐ |
| 2 | Naviguer entre toutes les pages via la sidebar (react-router 7.18.1) | Navigation fluide, icônes lucide 1.25.0 visibles, aucun écran blanc | ☐ |
| 3 | Basculer thème Catppuccin ↔ USSR (Settings) | Bascule immédiate, bannières USSR affichées (state zustand 5.0.14) | ☐ |
| 4 | Basculer langue EN ↔ FR | Textes traduits, persistance après reload | ☐ |
| 5 | Ouvrir une clef, vérifier le panneau détail (sous-clefs S/E/A) | Rendu correct | ☐ |

## 4. Renovate (post-merge, côté GitHub)

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | Après merge de la branche sur `main` + scan Renovate | Les PR #13/#15/#16/#19/#21/#25/#26/#28/#29/#30 se referment automatiquement | ☐ |
