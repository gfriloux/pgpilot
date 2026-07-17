# Plan v0.9.0 — Qualité & Sécurité

## Contexte

Après stabilisation de la v0.8.x (migration iced→Tauri, file pickers, CI fixes), un audit global
a été mené par 5 agents spécialisés (sécurité, qualité Rust, architecture, CI/CD, TypeScript).
Résultat : base saine mais plusieurs gaps vs standards industrie (Seahorse, Kleopatra, projets
Tauri matures). Cette version adresse ces gaps sans ajouter de fonctionnalités.

## Objectifs

- Corriger les 3 failles sécurité High dans la couche GPG subprocess
- Supprimer 2 fonctions IPC mortes qui crasheraient en production
- Réduire la duplication du code Rust (37 blocs spawn_blocking identiques, 9 patterns stderr)
- Ajouter des tests unitaires sur les parseurs critiques (validate_fp, verify_signature)
- Corriger les anti-patterns Zustand dans le frontend
- Stabiliser le pipeline CI/CD (cache key docs erroné, release CHANGELOG sans fallback)

## Périmètre

### In scope
- `--` systématique avant args positionnels gpg (H-1/H-2)
- `--no-options` dans `gpg_command()` (L-3)
- `JoinCode::verify` : contrôle signer_fp == invited_by + trust (H-3)
- `set_permissions` : propager l'erreur au lieu de `let _ = ...` (keyring.rs:340)
- Sanitisation des control chars dans le nom de room (M-4)
- Suppression `getCardInfo` et `verifySignature` de `ipc/keys.ts` (bugs réels)
- Fix mock `mock-tauri.ts` : retirer `get_card_info`, corriger le mock verify
- Dead write `dataset['theme']` dans `AppLayout.tsx`
- Extraction helper `gpg_run_check()` dans `keyring.rs`
- `[workspace.dependencies]` dans `Cargo.toml`
- `tempfile` déplacé de `[dependencies]` vers `[dev-dependencies]`
- `tokio = { features = ["full"] }` → features minimales
- Tests unitaires : `validate_fp`, `validate_keyserver_url`, `validate_keyserver_query`
- Tests unitaires : parseur `verify_signature` (extraction pure function + fixtures)
- `ENV_LOCK` partagé entre fichiers de test → `tests/common/env.rs`
- Zustand selectors granulaires (pas de `.filter()` sur résultat selector)
- `deleteRoom` : rollback si IPC échoue
- `import React` déplacé en haut de `Chat.tsx`
- CI docs : cache key `package-lock.json` au lieu de `package.json`
- CI release : `continue-on-error: true` sur le push CHANGELOG

### Out of scope (trop gros pour v0.9.0)
- Fingerprint newtype `struct Fingerprint([u8; 20])` — touche tous les modules, version dédiée
- tauri-specta — changement significatif du pipeline de build
- Factory methods `Room::new()`, `WireMessage::new()` dans la lib
- Épinglage des actions GitHub par SHA — faible risque, peut être v0.9.1
- Matrice CI macOS
- Implémentation du `WireMessage.signature` canonique (M-1)
- Rate limiting par sender dans le chat (M-2)

## État du working tree

- Version actuelle dans le code : `0.8.9` (à bumper vers `0.9.0` en Phase 8)
- Branches worktree anciennes dans `.claude/worktrees/` : ignorées (iced era)
- Tests : 30 lib passent, 92 E2E passent (voir phase0_results.md)

## Phases

### Phase 0 — Audit état initial ✅ DONE
**Résultats :** voir `phase0_results.md`

---

### Phase 1 — Sécurité GPG subprocess
**Agent :** `voltagent-lang:rust-engineer`
**Fichiers :** `src/gpg/keyring.rs`, `src/gpg/mod.rs`, `src/chat/rooms.rs`
**Livrable :** 5 corrections de sécurité

Travail :
1. Ajouter `cmd.arg("--")` avant tout arg positionnel path dans :
   - `import_key` (keyring.rs)
   - `verify_signature` (keyring.rs)
   - `decrypt_files` (keyring.rs)
   - `inspect_decrypt` (keyring.rs)
   - `JoinCode::verify` tempfiles (rooms.rs)
2. Ajouter `--no-options` dans `gpg_command()` (mod.rs) pour neutraliser le `gpg.conf` utilisateur
3. Propager l'erreur `set_permissions(0o600)` dans `backup_key` au lieu de `let _ = ...` (keyring.rs:340)
4. `JoinCode::verify` : parser le `VALIDSIG` via `--status-fd`, vérifier que le signer_fp == `invited_by`
5. Sanitiser les control chars (`\n`, `\r`, `\t`, chars de contrôle) dans le nom de room (lib.rs)

**Post-phase :** re-audit sécurité par `voltagent-qa-sec:security-auditor` sur les fichiers touchés

---

### Phase 2 — Bugs critiques frontend
**Agent :** `voltagent-lang:typescript-pro`
**Fichiers :** `app/src/ipc/keys.ts`, `app/src/lib/mock-tauri.ts`, `app/src/layout/AppLayout.tsx`
**Livrable :** 3 bugs réels corrigés

Travail :
1. Supprimer `getCardInfo` (invoque `'get_card_info'` inexistant) de `ipc/keys.ts`
2. Supprimer `verifySignature` (invoque `'verify_signature'` inexistant) de `ipc/keys.ts`
3. Retirer le handler `get_card_info` du mock `mock-tauri.ts` (masquait le bug ci-dessus)
4. Améliorer le mock `verify_signature_cmd` pour couvrir les variantes d'outcome (bad_sig, unknown_key)
5. Supprimer `document.documentElement.dataset['theme'] = theme` dans `AppLayout.tsx` (dead write — aucune règle CSS ne consomme `[data-theme]`, le thème passe par la classe `.theme-ussr`)

---

### Phase 3 — Qualité Rust : réduction duplication + Cargo
**Agent :** `voltagent-lang:rust-engineer`
**Fichiers :** `src/gpg/keyring.rs`, `Cargo.toml`, `app/src-tauri/Cargo.toml`
**Livrable :** code plus maintenable, Cargo workspace propre

Travail :
1. Extraire `fn gpg_run_check(output: &Output, ctx: &str) -> anyhow::Result<()>` dans `keyring.rs`
   pour dédupliquer les 9 occurrences du pattern `status.success() || return Err(sanitize_stderr(...))`
2. Ajouter `[workspace.dependencies]` dans `Cargo.toml` racine pour `serde`, `serde_json`, `chrono`, `tokio`
3. Déplacer `tempfile` de `[dependencies]` vers `[dev-dependencies]` dans `Cargo.toml` racine
4. Changer `tokio = { features = ["full"] }` → `["rt", "sync", "time", "macros"]` dans `Cargo.toml` racine
   (une bibliothèque ne doit pas forcer `"full"` sur ses consumers)

**Post-phase :** `voltagent-dev-exp:refactoring-specialist` sur les parties touchées

---

### Phase 4 — Tests unitaires Rust
**Agent :** `voltagent-qa-sec:test-automator`
**Fichiers :** `src/gpg/keyring.rs`, `tests/common/`, `tests/gpg_keyring.rs`, `tests/chat_crypto.rs`
**Livrable :** couverture des parseurs critiques + infrastructure de test propre

Travail :
1. Ajouter `#[cfg(test)] mod tests` dans `keyring.rs` couvrant :
   - `validate_fp` : 40 hex valide, 39 chars, 41 chars, non-hex, minuscule, mixte, vide
   - `validate_keyserver_url` : whitelisté, non-whitelisté, http, trailing slash
   - `validate_keyserver_query` : email, short ID, fp complet, caractères illégaux
2. Extraire `fn parse_verify_status(stdout: &str, stderr: &str) -> VerifyResult` comme pure function
   et ajouter des tests avec fixtures réels gpg (GOODSIG, BADSIG, NO_PUBKEY, EXPKEYSIG, REVKEYSIG)
3. Créer `tests/common/env.rs` avec un `ENV_LOCK: Mutex<()>` unique partagé par tous les fichiers
   de test (actuellement chaque fichier a le sien, race condition possible entre suites)

---

### Phase 5 — Qualité frontend : Zustand + patterns React
**Agent :** `voltagent-lang:typescript-pro`
**Fichiers :** `app/src/hooks/useKeys.ts`, `app/src/pages/Chat.tsx`, `app/src/pages/MyKeys.tsx`
**Livrable :** re-renders réduits, comportement correct de deleteRoom

Travail :
1. `useKeys.ts` : utiliser des selectors granulaires au lieu de `useKeysStore()` sans selector
2. `Chat.tsx:486` : déplacer le `.filter()` dans le selector pour éviter une nouvelle référence array à chaque render
3. `MyKeys.tsx:38,66` : wraper le filter dans `useMemo` ou déplacer dans le selector
4. `Chat.tsx:581` : implémenter le rollback si `chatDeleteRoom` IPC échoue
5. `Chat.tsx:223` : déplacer `import React from 'react'` en haut du fichier

---

### Phase 6 — CI/CD fixes
**Agent :** `voltagent-infra:deployment-engineer`
**Fichiers :** `.github/workflows/docs.yml`, `.github/workflows/release.yml`
**Livrable :** pipeline plus robuste

Travail :
1. `docs.yml` : corriger la cache key npm — `hashFiles('docs/package-lock.json')` au lieu de `package.json`
2. `release.yml` : ajouter `continue-on-error: true` sur le step push CHANGELOG + fallback si push rejected
   (les artifacts release ne doivent pas être bloqués par un échec de push CHANGELOG)

---

### Phase 7 — Code review + security re-audit global
**Agent :** `voltagent-qa-sec:code-reviewer` + `voltagent-qa-sec:security-auditor`
**Périmètre :** uniquement les fichiers touchés par les phases 1–6

Validation que :
- Les corrections sécurité phase 1 sont correctes et complètes
- Pas de régression introduite par la refactorisation
- Les nouveaux tests sont pertinents
- Aucun nouveau problème introduit

---

### Phase 8 — Bump version + Nix + validation finale
**Agent :** aucun (opérations mécaniques)

1. Bumper `0.8.9` → `0.9.0` dans les 5 fichiers :
   - `Cargo.toml` (racine)
   - `app/src-tauri/Cargo.toml`
   - `app/package.json`
   - `app/src-tauri/tauri.conf.json`
   - `packages/pgpilot/default.nix`
2. `cargo build` pour mettre à jour `Cargo.lock`
3. Vérifier `nix flake check`
4. Si `app/package-lock.json` a changé : recalculer `npmDepsHash`
5. Checklist finale (voir `manual_tests.md`)

## Décisions techniques

| Décision | Justification |
|---|---|
| `--no-options` dans `gpg_command()` | Standard industrie (Seahorse, Kleopatra, sequoia-sq) pour comportement déterministe |
| `--` avant args positionnels | Règle de base GNUPG Coding Guidelines — déjà appliquée sur encrypt/sign, manquante sur import/verify/decrypt |
| `tokio features = ["full"]` → minimal | Une lib ne doit pas forcer les features runtime sur ses consumers |
| Fingerprint newtype : out of scope | Touche ~40 fichiers, mérite une version dédiée pour des tests exhaustifs |
| tauri-specta : out of scope | Changement significatif de pipeline, impact build Nix à évaluer séparément |
| Tests parseur verify_signature | Extraction en pure function obligatoire avant tests — légère refactorisation |
