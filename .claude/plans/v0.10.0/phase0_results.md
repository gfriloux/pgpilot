# Phase 0 — Résultats de l'audit (v0.10.0)

## Branche
- Départ : `main` @ `e5d436d` (Merge branch 'chore/docs-in-git')
- Branche de travail créée : `chore/v0.10.0-appimage-renovate`

## `nix develop --command just ci`
- **Exit code final : 0 (vert)** — fmt-check + clippy + tests Rust + 92 E2E Playwright OK.

### Note environnement (NixOS) — E2E
Au premier lancement, l'E2E échouait avec :
```
NixOS cannot run dynamically linked executables intended for generic
.../.cache/ms-playwright/chromium_headless_shell-.../chrome-headless-shell
```
Cause : **aucun Chromium wrappé dans `/nix/store`**, donc `app/playwright.config.ts`
(`resolveChromiumExecutable()`) tombe sur le binaire Playwright téléchargé, incompatible NixOS.
Ce n'est **pas** une régression de code — état identique sur `main`.

**Correctif local (hors dépôt)** : `nix build nixpkgs#chromium` place
`/nix/store/…-chromium-149.0.7827.196` dans le store ; la config le détecte automatiquement.
Après ça : **92 passed**. Rien à committer — la CI (Ubuntu) utilise le Chromium téléchargé.

## Working tree au départ
- Non suivis à ne pas committer : `result`, `app/result`, `target-docker/`, `docker/`,
  `shells/appimage/` (suivi, sera supprimé), `SECURITY_PLAN.md` (interne, jamais committé).
