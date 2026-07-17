# Plan v0.9.1 — Correctif AppImage EGL crash

## Contexte

Remontée utilisateur après la release v0.9.0 : l'AppImage est inutilisable sur toute
distribution autre qu'Ubuntu. Testé sur deux machines :
- Arch Linux, i3, X11
- NixOS, Plasma 6, Wayland

Erreur fatale au démarrage :
```
Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...
```

**Cause racine** : WebKitGTK 4.1 active par défaut le renderer DMA-BUF depuis une
version récente. Ce renderer initialise EGL directement. Dans le contexte AppImage,
les libs WebKit bundlées proviennent du CI Ubuntu ; leurs chemins EGL divergent de
ceux du système hôte, provoquant un `abort()` fatal. Le même chemin de code peut
échouer sur Arch/NixOS même hors AppImage si la config EGL diffère.

## Objectifs

- Corriger le crash EGL au démarrage de l'AppImage sur toutes les distributions Linux
- Préserver le comportement sur Ubuntu (DMA-BUF fonctionne, pas de régression)
- Ne pas forcer le software rendering (impact perf)

## Périmètre (révisé après test v0.9.1)

### In scope
- `app/src-tauri/src/main.rs` : injection `WEBKIT_DISABLE_DMABUF_RENDERER=1` avant init Tauri
- Bump version dans les 5 fichiers réglementaires (0.9.0 → 0.9.1)
- Patch AppRun en CI (`release.yml`) : injecter la var avant l'ELF, pour couvrir les sous-processus WebKit (WebKitGPUProcess) qui ré-exécutent l'AppImage
- `shells/appimage/default.nix` : environnement `buildFHSEnv` pour build AppImage local sur NixOS
- `Justfile` : recettes `build-appimage` et `patch-appimage`

### Out of scope
- Toute autre feature ou correctif non lié à ce crash
- Changements frontend, docs

## État du working tree

Branche `v0.9.1` depuis `main` (tag `v0.9.0`). Working tree propre attendu.

## Décisions techniques

### Pourquoi `WEBKIT_DISABLE_DMABUF_RENDERER` et pas une autre variable ?

| Variable | Effet | Verdict |
|---|---|---|
| `WEBKIT_DISABLE_DMABUF_RENDERER=1` | Désactive uniquement le renderer DMA-BUF, WebKit choisit un autre path HW-accelerated | ✅ Ciblé |
| `WEBKIT_DISABLE_COMPOSITING_MODE=1` | Désactive tout le compositing GPU | ❌ Trop large |
| `LIBGL_ALWAYS_SOFTWARE=1` | Force Mesa software (llvmpipe) | ❌ Impact perf |

### Pourquoi dans `main.rs` ET dans AppRun ?

- `main.rs` couvre les distributions non-AppImage (deb, rpm, binaire brut) et sert de filet de sécurité.
- AppRun est nécessaire pour les **sous-processus WebKit** : WebKitGPUProcess se ré-exécute
  en passant par AppRun avant que `main()` soit atteint. Si la var n'est pas dans AppRun,
  le GPU process crashe silencieusement → fenêtre blanche.
- Le patching AppRun est stable : on injecte une ligne après le shebang via `head/tail/mksquashfs`.
  Aucune dépendance au hash ou à appimagetool — on opère sur le SquashFS directement.

### Pourquoi `#[cfg(target_os = "linux")]` ?

WebKitGTK n'existe que sur Linux. Sur macOS/Windows Tauri utilise WKWebView/WebView2,
non affectés par ce renderer.

### Pourquoi garder l'override utilisateur (`is_none()`) ?

Un utilisateur qui a EGL cassé pour une raison différente pourrait avoir mis
`WEBKIT_DISABLE_DMABUF_RENDERER=0` explicitement. On respecte son choix.

### `unsafe` nécessaire ?

Non. `std::env::set_var` est une fonction **safe** en Rust edition 2021 (stable).
L'appel se fait avant que Tauri ne spawne des threads, donc aucun risque de data race.

## Phases

### Phase 0 — Audit baseline (obligatoire)

**Agent** : aucun — exécution directe  
**Commandes** :
```bash
nix develop --command sh -c '
  cargo build 2>&1 | tail -5
  cargo clippy -- -D warnings 2>&1 | tail -10
  cargo test --package pgpilot --lib 2>&1 | tail -10
'
cd app && npm run test:e2e 2>&1 | tail -30
```
**Livrable** : `phase0_results.md` — consigner les résultats avant tout touche au code.

---

### Phase 1 — Implémentation du correctif

**Agent** : `voltagent-lang:rust-engineer`  
**Dépendance** : Phase 0 terminée  
**Fichiers touchés** :
- `app/src-tauri/src/main.rs` — injection env var
- `Cargo.toml` (racine) — version 0.9.1
- `app/src-tauri/Cargo.toml` — version 0.9.1
- `app/package.json` — version 0.9.1
- `app/src-tauri/tauri.conf.json` — version 0.9.1
- `packages/pgpilot/default.nix` — version 0.9.1

**Contenu exact du correctif** dans `main.rs` (après le `cfg_attr`) :
```rust
fn main() {
  #[cfg(target_os = "linux")]
  if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
  }

  pgpilot_lib::run();
}
```

**Livrable** : les 6 fichiers modifiés, `cargo build` sans erreur.

---

### Phase 2 — Tests automatisés

**Agent** : `voltagent-qa-sec:test-automator`  
**Dépendance** : Phase 1 terminée  
**Commandes** :
```bash
nix develop --command sh -c '
  cargo build
  cargo clippy -- -D warnings
  cargo test --package pgpilot --lib
'
cd app && npm run test:e2e
```
Aucun nouveau test automatisé à écrire : le fix est un env var set avant init,
non testable en unit test sans mock du runtime WebKit.

**Livrable** : confirmation que les suites existantes passent toujours.

---

### Phase 3 — Audit sécurité

**Agent** : `voltagent-qa-sec:security-auditor`  
**Dépendance** : Phase 2 terminée  
**Scope** : uniquement `app/src-tauri/src/main.rs` (le seul fichier modifié dans la couche runtime)  
**Points à vérifier** :
- L'env var est-elle inoffensive côté sécurité ? (oui — lecture-seule par WebKit, aucun shell, aucun IPC)
- Risque de pollution d'env pour les sous-processus GPG ? (non — gpg ne lit pas cette var)
- Aucune autre surface d'attaque introduite

**Livrable** : rapport sécurité (même court).

---

### Phase 4 — Code review

**Agent** : `voltagent-qa-sec:code-reviewer`  
**Dépendance** : Phase 3 terminée  
**Scope** : diff complet de la branche vs `main`  
**Points à vérifier** :
- Style Rust (indentation 2 espaces, pas de commentaire inutile)
- Les 5 fichiers de version bumps sont cohérents entre eux
- Pas d'ajout non désiré

**Livrable** : rapport qualité, corrections appliquées si nécessaire.

---

### Phase 5 — Vérification Nix

**Agent** : aucun — exécution directe  
**Dépendance** : Phase 4 terminée  
**Commandes** :
```bash
nix flake check
nix build
./result/bin/pgpilot-app
```
`npmDepsHash` ne change pas (aucun changement à `app/package-lock.json`).

**Checklist KDE/Plasma** (depuis PROCEDURE_PLANS.md §8) :
- [ ] `GTK_USE_PORTAL=1` présent dans shellHook
- [ ] `GTK_USE_PORTAL=1` présent dans `makeWrapperArgs` de `packages/pgpilot/default.nix`

**Livrable** : `./result/bin/pgpilot-app` se lance sans erreur.

---

### Phase 6 — AppRun patching + build local AppImage

**Agent** : aucun — exécution directe  
**Dépendance** : Phase 5 terminée  
**Fichiers touchés** :
- `.github/workflows/release.yml` — step "Patch AppImage AppRun" après "Build AppImage"
- `shells/appimage/default.nix` — `buildFHSEnv` pour build AppImage local sur NixOS
- `Justfile` — recettes `build-appimage` et `patch-appimage`

**Vérification** :
```bash
# Vérifier que la syntaxe du workflow est valide
nix develop --command sh -c 'python3 -c "import yaml; yaml.safe_load(open(\".github/workflows/release.yml\"))"'
# Vérifier que le shell appimage est reconnu par Nix (sans builder)
nix eval .#devShells.x86_64-linux.appimage
```

**Livrable** : les 3 fichiers, syntaxe vérifiée.

---

### Phase 7 — Validation finale + tag

**Dépendance** : Phase 6 terminée, `manual_tests.md` exécuté par l'utilisateur  
**Checklist automatisée** :
```bash
nix develop --command sh -c '
  cargo build
  cargo clippy -- -D warnings
  cargo test --package pgpilot --lib
'
cd app && npm run test:e2e
nix build
nix flake check
```
**Checklist manuelle** : exécuter `manual_tests.md`

**Tag** (par l'utilisateur) :
```bash
git tag v0.9.1
git push --tags
```
La CI génère le CHANGELOG, patche l'AppRun, et publie `.deb`, `.rpm`, `.AppImage`.
