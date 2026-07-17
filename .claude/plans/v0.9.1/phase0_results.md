# Phase 0 — Résultats audit initial v0.9.1

Date : 2026-05-31
Branche : v0.9.1 (depuis main @ b294520)

## cargo build
```
Compiling pgpilot v0.9.0
Finished `dev` profile in 3.77s
```
✅ Propre

## cargo clippy -- -D warnings
```
Finished `dev` profile in 2.25s
```
✅ 0 warning

## cargo test --package pgpilot --lib
```
59 passed; 0 failed; 0 ignored; finished in 0.04s
```
✅ Tous verts

## npm run test:e2e (VITE_MOCK=true)

### Premier run (sans override)
```
92 failed — Error: browserType.launch: Target page, context or browser has been closed
```
❌ Chromium ne se lance pas

### Diagnostic
`playwright.config.ts` contient deux paths Nix hardcodés devenus obsolètes :
- `/nix/store/5d32m7b4znr83wh3ajaxwr5kynplqri3-playwright-browsers/...` → absent
- `/nix/store/6js92rbq8qyyyrblvn7s2nvr0grmyydh-playwright-chromium/...` → absent

Le Chromium système actuel est :
```
/nix/store/r7ifk1v95jfl02775kgbrd61dyr1rfsx-chromium-148.0.7778.178/bin/chromium
```

### Run avec override explicite
```
PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH=/nix/store/r7ifk1v95jfl02775kgbrd61dyr1rfsx-chromium-148.0.7778.178/bin/chromium
→ 92 passed (20.8s)
```
✅ Tous verts avec le bon binaire

## Conclusion

Base propre (Rust ✅, E2E ✅ avec override). Un problème pré-existant et distinct du bug EGL : les
paths hardcodés Nix dans `playwright.config.ts` ne sont plus valides après mise à jour nixpkgs.

**Correction à inclure en Phase 1** : remplacer la détection statique par un glob dynamique
`/nix/store/*-chromium-N.*/bin/chromium` pour ne plus être sensible aux hashes Nix.
Ce fix est hors périmètre strict v0.9.1 mais trivial et améliore le workflow dev NixOS.
À valider avec l'utilisateur.
