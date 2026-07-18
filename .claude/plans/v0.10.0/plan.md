# Plan v0.10.0 — Retrait AppImage + consolidation Renovate

## Contexte

L'AppImage de PGPilot n'a jamais fonctionné correctement : v0.9.1 était déjà un
correctif dédié au crash EGL au démarrage de l'AppImage (`WEBKIT_DISABLE_DMABUF_RENDERER`,
patch AppRun en CI). Le renderer DMA-BUF de WebKitGTK 4.1 reste fragile sur toutes les
distributions, et l'artefact impose une infrastructure lourde (build hors `nix develop`,
dépendances système APT, shell FHS expérimental) pour un résultat non fiable. On abandonne
l'AppImage : les utilisateurs installent via `.deb`/`.rpm` (releases GitHub) ou via
home-manager (Nix), qui sont les canaux réellement supportés.

En parallèle, 10 PR Renovate s'accumulent depuis plusieurs semaines. Elles sont toutes des
bumps mineurs/patch **dans les majeures déjà en place** (vite 8.x, react 19.x, eslint 10.x,
storybook 10.x, typescript-eslint 8.x, deps Rust, GitHub Actions) — risque faible, mais à
valider ensemble par la porte qualité.

**Décisions validées avec l'utilisateur :**
- Version : **v0.10.0** (retrait d'un artefact de release = changement de distribution visible → bump mineur).
- Renovate : **consolider tous les bumps sur la branche du plan**. L'utilisateur merge la
  branche ; les PR Renovate se referment automatiquement au scan suivant (politique git hybride :
  Claude ne merge/push/tag jamais sur `main`).
- Périmètre : **les 10 MAJ dans ce seul plan**.

## Objectifs

1. Supprimer toute l'infrastructure AppImage (CI, shells Nix, config Tauri, docs).
2. Intégrer les 10 MAJ Renovate, lockfiles régénérés et cohérents, `just ci` vert.
3. Bump version v0.10.0 (5 fichiers + `npmDepsHash`), build Nix vérifié.

## Périmètre

**In scope**
- Retrait AppImage : `release.yml`, `shells/appimage/`, `shells/default/default.nix`,
  `app/src-tauri/tauri.conf.json`, `Justfile`, `README.md`, `CLAUDE.md`, `PROCEDURE_PLANS.md`.
- MAJ deps : npm (7 PR), Cargo (#19), GitHub Actions (#26).
- Bump version + `npmDepsHash` + index des plans.

**Out of scope**
- Le workaround EGL `main.rs:4-10` (`WEBKIT_DISABLE_DMABUF_RENDERER`) : **conservé**, c'est un
  filet de sécurité Linux général (Arch/NixOS aussi), pas spécifique à l'AppImage.
- Toute évolution fonctionnelle de l'app.
- CHANGELOG.md (généré par la CI via git-cliff au tag).

## État du working tree

- Branche de départ : `main` (à jour, `just ci` doit passer — cf. Phase 0).
- Artefacts non suivis à ignorer / ne pas commiter : `result`, `app/result`, `target-docker/`,
  `docker/`, `SECURITY_PLAN.md` (interne, jamais commité).
- Le `Justfile` actuel **ne contient pas** les recettes `build-appimage`/`patch-appimage`
  (elles figuraient dans le plan v0.9.1 mais ne sont pas dans le fichier réel) — rien à retirer
  de ce côté hormis le commentaire de la recette `build`.

## Détail des MAJ Renovate

| PR | Contenu | Fichiers |
|----|---------|----------|
| #30 | storybook monorepo → 10.5.2 | package.json + lock |
| #29 | react monorepo (patch 19.x) | package.json + lock |
| #28 | vite → 8.1.5 | package.json + lock |
| #26 | GitHub Actions (major) | ci.yml, docs.yml, release.yml |
| #25 | zustand → 5.0.14 | lock |
| #21 | typescript-eslint → 8.64.0 | package.json + lock |
| #19 | Rust deps (rustls 0.23.42, tokio 1.x, tauri 2.x, serde, …) | Cargo.lock |
| #16 | lucide-react → 1.25.0 | package.json + lock |
| #15 | eslint → 10.7.0 | package.json + lock |
| #13 | react-router-dom → 7.18.1 | package.json + lock |

**GitHub Actions (#26)** — bumps à appliquer sur les 3 workflows :
`actions/checkout` v6→v7 · `actions/setup-node` v6→v7 ·
`DeterminateSystems/magic-nix-cache-action` v13→v14 · `actions/cache` v5→v6.

## Phases

### Phase 0 — Audit obligatoire (PROCEDURE §2)
- Créer la branche `chore/v0.10.0-appimage-renovate`.
- `nix develop --command just ci` → consigner exit code + branche dans
  `.claude/plans/v0.10.0/phase0_results.md`.

### Phase 1 — Retrait de l'infrastructure AppImage
Commit `chore(nix,ci): remove non-functional AppImage build`
- **`.github/workflows/release.yml`** : supprimer les steps « Install system dependencies for
  AppImage » et « Build AppImage » ; retirer `target/release/bundle/appimage/*.AppImage` de la
  liste `files:` du step release.
- **`shells/appimage/`** : supprimer le répertoire (devShell `appimage` auto-découvert par
  snowfall ; aucune référence externe — vérifié).
- **`shells/default/default.nix`** : retirer le bloc `APPIMAGE_EXTRACT_AND_RUN=1` +
  commentaire linuxdeploy (lignes ~71-74) — plus de build AppImage local, linuxdeploy n'est
  plus invoqué par `just build` (deb,rpm).
- **`app/src-tauri/tauri.conf.json`** : retirer le bloc `bundle.linux.appimage`.
- **`Justfile`** : réécrire le commentaire de la recette `build` (deb/rpm uniquement, sans
  mention « use CI for AppImage »).
- **Docs** : `README.md` (l.98 `just build`, l.127 `build-bin`), `CLAUDE.md` (l.90, l.114
  release desc, section « AppImage in CI » l.251-253), `PROCEDURE_PLANS.md` (l.202 : artefacts
  `.deb` et `.rpm`).

### Phase 2 — MAJ dépendances npm (agent `voltagent-dev-exp:dependency-manager` + `react-specialist`)
Commit `chore(deps): bump npm dependencies (react, vite, storybook, eslint, …)`
- Appliquer dans `app/package.json` les versions cibles des PR #30/#29/#28/#25/#21/#16/#15/#13.
- `cd app && npm install` pour régénérer un `package-lock.json` cohérent unique.
- Valider : `cd app && npm run build` + `just e2e` (Playwright, VITE_MOCK).

### Phase 3 — MAJ dépendances Rust #19 (agent `voltagent-lang:rust-engineer`)
Commit `chore(deps): bump Rust dependencies`
- Reproduire les bumps de #19 via `cargo update -p <crate>` (rustls, tokio, tauri, serde, …)
  ou `git checkout renovate/rust-deps -- Cargo.lock` puis `cargo build` (seul `Cargo.lock`
  change dans #19). Vérifier qu'aucune nouvelle CVE hors liste ignorée n'apparaît : `just audit`.
- Valider : `cargo build` + `cargo clippy -- -D warnings` + `just test`.

### Phase 4 — MAJ GitHub Actions #26
Commit `ci: bump GitHub Actions versions`
- Appliquer les 4 bumps d'action sur `ci.yml`, `docs.yml`, et `release.yml` (release.yml déjà
  modifié en Phase 1 → édition coordonnée, pas de conflit).

### Phase 5 — Bump version v0.10.0 + hash Nix + index (PROCEDURE §5, §7a, §1)
Commit `chore(release): bump version to 0.10.0`
- `just release 0.10.0` (met à jour les 5 fichiers + ligne version de `package-lock.json` +
  recalcule `npmDepsHash`). ⚠️ Le `npmDepsHash` doit être recalculé **après** la Phase 2
  (lockfile figé) → soit lancer `just release` après Phase 2, soit `just update-nix-hash`
  séparément. Ordre retenu : bump version ici, puis `just update-nix-hash` pour intégrer le
  lockfile de Phase 2.
- Mettre à jour `.claude/plans/README.md` : entrée v0.10.0 + carte des versions.

### Phase 6 — Revue qualité + validation finale (agent `voltagent-qa-sec:code-reviewer`)
- `voltagent-qa-sec:code-reviewer` sur le diff complet de la branche.
- Porte qualité (PROCEDURE §8) :
  - `nix develop --command just ci` (fmt-check + clippy + tests Rust + E2E) → **vert**
  - `nix build` → produit `./result/bin/pgpilot-app` (build long ~20-40 min ; le lancer)
  - `nix flake check` → intégrité flake + module home-manager
  - `just audit` → aucune CVE hors liste ignorée

### Phase 7 — Tests manuels + clôture
- Rédiger `.claude/plans/v0.10.0/manual_tests.md` (voir Vérification).
- Branche prête, portes vertes. **Claude s'arrête ici** : l'utilisateur relit, merge sur `main`,
  push, tague `v0.10.0`. Les PR Renovate #13/#15/#16/#19/#21/#25/#26/#28/#29/#30 se referment
  au scan Renovate suivant.

## Agents mobilisés (PROCEDURE §4)
- `voltagent-dev-exp:dependency-manager` — MAJ npm (Phase 2).
- `voltagent-lang:rust-engineer` — MAJ crates (Phase 3).
- `voltagent-lang:react-specialist` — vérif build front après bumps react/vite (Phase 2).
- `voltagent-qa-sec:code-reviewer` — avant validation finale (Phase 6).
- Non requis : `security-auditor` (aucun code crypto/IPC/fichiers touché — `just audit` couvre
  les CVE deps), `refactoring-specialist`, `test-automator`, `technical-writer` (pas de feature
  visible ; docs éditées inline).

## Décisions techniques
- **Consolidation lockfiles** : un seul `npm install` produit un `package-lock.json` cohérent
  plutôt que cherry-pick de 7 branches Renovate divergentes (évite les conflits inter-PR). Les
  versions patch peuvent différer marginalement de Renovate — sans impact, Renovate re-résout.
- **`npmDepsHash`** recalculé après figement du lockfile (sinon build Nix casse — PROCEDURE §7a).
- **release.yml** édité une fois (Phase 1 retrait AppImage) puis complété (Phase 4 bumps actions)
  pour éviter tout conflit avec la PR #26.

## Vérification (end-to-end)

**Automatisé (dans `manual_tests.md` + porte qualité)**
- `nix develop --command just ci` — vert (fmt, clippy, tests Rust, 90 E2E Playwright).
- `nix build && ./result/bin/pgpilot-app` — l'app démarre, aucun crash EGL.
- `nix flake check` — OK (le devShell `appimage` a bien disparu).
- `grep -rniE appimage .` (hors `.claude/plans/` historiques, `node_modules`, artefacts) — plus
  aucune référence active dans le code/CI/docs.

**Manuel (`manual_tests.md`)**
- Vérifier que `just build` produit `.deb` + `.rpm` (pas d'appimage) sans erreur.
- Vérifier que le front buildé (`npm run build`) se lance en mock (`VITE_MOCK=true npm run dev`),
  navigation OK (deps react/router/vite/lucide/zustand à jour).
- Vérifier au tag (côté CI, après merge utilisateur) que la release GitHub ne publie que
  `.deb` + `.rpm`.
