# Phase 0 — Audit de la base (v0.10.1)

Audit exécuté le **2026-09-27** sur `main` à `6dc18c5`
(`chore(release): update CHANGELOG.md for v0.10.0`).

Toutes les valeurs ci-dessous sont **mesurées**, pas déduites.

## Gate Rust

| Gate | Résultat |
|------|----------|
| `cargo fmt --check` (lib + `app/src-tauri`) | ✅ |
| `cargo clippy --all-targets -- -D warnings` | ✅ |
| `cargo test --package pgpilot` | ✅ |
| `cargo test --package pgpilot -- --ignored` | ✅ 3 tests GPG (`create_key_has_sign_encr_auth_subkeys`, `create_key_returns_valid_fingerprint`, `list_keys_empty_homedir`) |

## Gate frontend

| Gate | Résultat |
|------|----------|
| `npm ci` (app) | ✅ |
| `npm run build` (`tsc && vite build`) | ✅ 1848 modules, bundle 355 kB (gzip 112 kB) |
| `npm run test:e2e` | ✅ **92/92** |
| `npm run lint` (eslint) | ❌ **1 erreur préexistante** |

### Dette relevée — eslint casse déjà sur `main`

```
app/src/layout/AppLayout.tsx
  62:9  error  'theme' is assigned a value but never used  @typescript-eslint/no-unused-vars

app/src/lib/mock-event.ts
  9:1  warning  Unused eslint-disable directive
```

`AppLayout.tsx:62` (`const theme = useConfigStore((s) => s.theme);`) est bien présent sur
`main` — ce n'est pas une régression des PR Renovate. C'est cohérent avec le fait que
`npm run lint` **n'est pas** dans `just ci`. **Hors périmètre v0.10.1**, à traiter à part.

## Gate docs

| Gate | Résultat |
|------|----------|
| `npm ci` (docs) | ✅ |
| `npm run build` (astro 6.3.x) | ✅ 25 pages, **0 warning** |

Le « 0 warning » est la référence qui sert de critère d'acceptation à la migration
astro 7 (phase 2). Seule exception préexistante, présente aussi sur astro 6 :
`Entry docs → 404 was not found` (pas de `404.md` dans la collection ; starlight
retombe sur sa page 404 intégrée).

## Gate Nix

| Gate | Résultat |
|------|----------|
| `npmDeps.hash` enregistré | `sha256-lJmI8PZJ6caLLLu7uqUwFvUidWrmaf1UlI+v04sejMo=` |
| `prefetch-npm-deps app/package-lock.json` sur `main` | Identique ✅ — `main` est cohérent |

## Versions couplées npm ↔ Rust (état `main`)

| Composant | Rust (`Cargo.lock`) | npm (`app/package-lock.json`) |
|-----------|---------------------|-------------------------------|
| tauri / `@tauri-apps/api` | 2.11.5 | 2.11.1 |
| tauri-plugin-dialog | 2.7.1 | 2.7.1 |

Le CLI Tauri **refuse de builder** si ces paires divergent en major/minor. C'est le
point dur de cette version (cf. `plan.md`, décision D1).

## Écarts entre l'outillage local et Renovate

- npm local (dev shell) : **10.9.7**. Renovate génère ses lockfiles avec npm 11, qui
  écrit en plus les champs `libc` sur les paquets optionnels multi-plateformes. Un
  `npm install` local **retire** ces champs → churn de lockfile sans rapport avec la MAJ.
  → **Toujours reprendre le lockfile de Renovate tel quel** (cherry-pick), ne jamais le
  régénérer localement, sauf quand Renovate n'a pas su le faire (`renovate/artifacts`
  en échec).

## Angle mort de la CI (cause racine des ✅ trompeurs)

`.github/workflows/ci.yml` ne contient que des étapes Rust (`fmt`, `clippy`, `build`,
`test`, `audit`) plus `nix flake check`. Il n'exécute **ni** `npm ci`, **ni** `tsc`,
**ni** Playwright, **ni** le build docs, **ni** `nix build .#pgpilot`.

Conséquence mesurée : les **9 PR npm** de cette vague affichaient toutes
« Build, lint, audit : SUCCESS » alors que 4 d'entre elles sont inmergeables en l'état.
`nix flake check` passe aussi, car il ne construit pas le paquet.

→ Hors périmètre v0.10.1 sur décision utilisateur, **conservé au backlog**.
