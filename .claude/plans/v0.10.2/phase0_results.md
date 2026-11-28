# Phase 0 — Audit de la base (v0.10.2)

Audit exécuté le **2026-11-28** sur `main` à `0d9a3c0` (`Merge branch 'chore/flake-update'`),
dans le dev shell rechargé après la mise à jour de `flake.lock` (nixpkgs 2026-11-24).

Toutes les valeurs ci-dessous sont **mesurées**.

## Porte qualité

| Gate | Résultat |
|------|----------|
| `nix develop --command just ci` | ✅ exit 0 |
| `cargo fmt --check` (lib + `app/src-tauri`) | ✅ |
| `cargo clippy -- -D warnings` | ✅ |
| `cargo test --package pgpilot` | ✅ 59 tests unitaires + 48 tests d'intégration, 0 échec |
| `cargo test --package pgpilot -- --ignored` | ✅ 9 tests (4 crypto chat, 1 MQTT, 1 join code, 3 keyring GPG) |
| Playwright E2E | ✅ **92/92** |

## `cargo audit`

Lancé avec la liste d'exclusion de `ci.yml` (7 identifiants).

**9 vulnérabilités hors liste**, toutes publiées après `v0.10.1` :

| Advisory | Crate | Date | Correctif disponible |
|----------|-------|------|----------------------|
| RUSTSEC-2026-0346 | `rustls` 0.23.45 | 2026-10-14 | 0.23.46 |
| RUSTSEC-2026-0347 | `rustls` 0.23.45 | 2026-10-14 | 0.23.46 |
| RUSTSEC-2026-0348 | `rustls` 0.23.45 | 2026-10-14 | 0.23.46 |
| RUSTSEC-2026-0349 | `rustls` 0.23.45 | 2026-10-14 | 0.23.46 |
| RUSTSEC-2026-0382 | `rustls-webpki` 0.103.15 | 2026-10-20 | 0.103.16 |
| RUSTSEC-2026-0382 | `rustls-webpki` 0.102.8 | 2026-10-20 | ❌ aucun sur la branche 0.102 (via `rumqttc 0.25.1`) |
| RUSTSEC-2026-0381 | `webpki-roots` 1.0.9 | 2026-10-19 | 1.0.10+ |
| RUSTSEC-2026-0592 | `url` 2.5.8 | 2026-11-20 | 2.5.9 |
| RUSTSEC-2026-0594 | `cookie` 0.18.1 | 2026-11-23 | 0.18.3 |

Avertissements (non bloquants pour `cargo audit`) : 10, dont deux `unsound` corrigeables
par le lock (`tokio` 1.53.1 → RUSTSEC-2026-0386, `rustix` 1.1.4 → RUSTSEC-2026-0486).
Les 8 autres (`chrono`, `proc-macro-error`, `unsafe-libyaml`, `zbus_names`, `zvariant`,
`zvariant_derive`, `event-listener`, `glib` 0.18.5) n'ont pas de correctif accessible
par une mise à jour du lock.

`cargo update --dry-run` ciblé : 8 paquets changent, 167 restent en place.

→ Traité en phase 2 du plan.

## Nix

| Contrôle | Résultat |
|----------|----------|
| `flake.lock` | nixpkgs du 2026-11-24 |
| `LD_LIBRARY_PATH` de la session | bibliothèques du nouveau nixpkgs (glib 2.90.1, webkitgtk 2.54.3) : session bien rechargée |
| `GTK_USE_PORTAL=1` dans le dev shell (`shells/default/default.nix:69`) | ✅ |
| `--set GTK_USE_PORTAL 1` dans `makeWrapperArgs` (`packages/pgpilot/default.nix:101`) | ✅ |

## Écarts documentaires relevés

- `.claude/plans/README.md` : la ligne `v0.10.1` de la carte des versions porte encore
  `_pending_` alors que le tag existe (`b02c31c`).
- Les trois correctifs post-release étaient consignés dans `v0.10.1/plan.md` alors
  qu'ils produisent `v0.10.2`.

→ Corrigés dans le commit du plan.

## PR Renovate ouvertes

10 PR : #36, #39, #40, #41, #42, #43, #44, #45, #46, #54. Hors périmètre (cf. plan).
