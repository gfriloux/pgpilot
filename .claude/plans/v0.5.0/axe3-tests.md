# Axe 3 — Tests unitaires

## Objectif

- Tester la couche `src/gpg/` (fonctions pures, parsing, opérations GPG)
- Tester la logique métier `src/app/` (handlers sans GUI iced)
- CI : `cargo test` intégré au workflow GitHub Actions
- Coverage cible : **≥ 60 %** sur `src/gpg/` et `src/app/`

## Stratégie

- Tests GPG : vrais processus `gpg` dans un `$GNUPGHOME` temporaire (pas de mocks)
- Tests app : logique des handlers sans dépendance à iced rendering
- Fixtures : clefs PGP de test pré-générées (armored text en dur dans `tests/fixtures/`)
- Slow tests marqués `#[ignore]` — lancés explicitement en CI

---

## T3.1 — Setup infrastructure tests

**Complexité** : M  
**Agent** : `voltagent-qa-sec:test-automator`  
**Dépendances** : aucune

### Fichiers à créer

**`tests/common/mod.rs`** :
```rust
use std::path::PathBuf;
use tempfile::TempDir;

/// Crée un GNUPGHOME temporaire isolé. Retourne (TempDir, chemin string).
/// TempDir doit rester en scope pendant le test — il supprime le dossier à sa mort.
pub fn setup_test_gnupghome() -> (TempDir, String) {
    let dir = TempDir::new().expect("tempdir");
    // chmod 700 obligatoire pour gpg
    std::fs::set_permissions(dir.path(), std::os::unix::fs::PermissionsExt::from_mode(0o700))
        .expect("chmod 700");
    let path = dir.path().to_str().unwrap().to_string();
    (dir, path)
}

/// Importe une clef armored dans le homedir. Retourne le fingerprint.
pub fn import_armored(homedir: &str, armored: &str) -> String {
    use std::process::{Command, Stdio};
    use std::io::Write;
    let mut child = Command::new("gpg")
        .args(["--homedir", homedir, "--batch", "--import"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("gpg spawn");
    child.stdin.as_mut().unwrap().write_all(armored.as_bytes()).unwrap();
    child.wait().expect("gpg wait");
    // Retourner le fingerprint depuis gpg --list-keys --with-colons
    list_fingerprints(homedir).into_iter().next().expect("imported key fp")
}

pub fn list_fingerprints(homedir: &str) -> Vec<String> {
    // gpg --homedir <h> --list-keys --with-colons | grep "^fpr" | ...
}
```

**`tests/fixtures/mod.rs`** :
```rust
// Clef de test "Test User <test@pgpilot.test>" — générée hors ligne, sans passphrase
pub const TEST_SECRET_KEY: &str = include_str!("test_secret.asc");
pub const TEST_PUBLIC_KEY: &str = include_str!("test_public.asc");
pub const TEST_THIRD_PARTY_PUBLIC: &str = include_str!("third_party_public.asc");
```

**`tests/fixtures/test_secret.asc`** : clef privée de test (armored, générée une fois)
**`tests/fixtures/test_public.asc`** : clef publique correspondante
**`tests/fixtures/third_party_public.asc`** : clef tierce pour tests multi-destinataires

### Dépendances Cargo à ajouter

```toml
[dev-dependencies]
tempfile = "3"
```

**Commit** : `test: add test infrastructure helpers and key fixtures`

---

## T3.2 — Tests `src/gpg/types.rs`

**Complexité** : M  
**Agent** : `voltagent-qa-sec:test-automator`  
**Dépendances** : T3.1

Tests dans `src/gpg/types.rs` (module `#[cfg(test)]`) :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // validate_fp
    #[test]
    fn validate_fp_valid() { assert!(validate_fp("A".repeat(40)).is_ok()); }
    #[test]
    fn validate_fp_too_short() { assert!(validate_fp("ABCD1234").is_err()); }
    #[test]
    fn validate_fp_non_hex() { assert!(validate_fp("Z".repeat(40)).is_err()); }
    #[test]
    fn validate_fp_empty() { assert!(validate_fp("").is_err()); }

    // TrustLevel
    #[test]
    fn trust_level_parsing() {
        assert_eq!(TrustLevel::from_gpg_char('-'), TrustLevel::Undefined);
        assert_eq!(TrustLevel::from_gpg_char('m'), TrustLevel::Marginal);
        assert_eq!(TrustLevel::from_gpg_char('f'), TrustLevel::Full);
        assert_eq!(TrustLevel::from_gpg_char('u'), TrustLevel::Ultimate);
    }
    #[test]
    fn trust_level_sufficient() {
        assert!(!TrustLevel::Undefined.is_sufficient());
        assert!(!TrustLevel::Marginal.is_sufficient());
        assert!(TrustLevel::Full.is_sufficient());
        assert!(TrustLevel::Ultimate.is_sufficient());
    }

    // SubkeyType
    #[test]
    fn subkey_type_from_usage() {
        assert_eq!(SubkeyType::from_usage_flags("sign"), Some(SubkeyType::Sign));
        assert_eq!(SubkeyType::from_usage_flags("encr"), Some(SubkeyType::Encr));
        assert_eq!(SubkeyType::from_usage_flags("auth"), Some(SubkeyType::Auth));
        assert_eq!(SubkeyType::from_usage_flags("cert"), None);
    }

    // VerifyOutcome
    #[test]
    fn verify_outcome_variants_distinct() {
        // S'assurer que chaque variant est non-égal aux autres (propriété enum)
        assert_ne!(VerifyOutcome::Valid, VerifyOutcome::BadSig);
        assert_ne!(VerifyOutcome::UnknownKey, VerifyOutcome::ExpiredKey);
    }
}
```

**Commit** : `test(gpg): unit tests for types.rs (validate_fp, TrustLevel, SubkeyType, VerifyOutcome)`

---

## T3.3 — Tests `src/gpg/keyring.rs`

**Complexité** : L  
**Agent** : `voltagent-qa-sec:test-automator`  
**Dépendances** : T3.1

Tests dans `tests/gpg_keyring.rs` (intégration — vrais processus gpg) :

```rust
mod common;
use common::{setup_test_gnupghome, import_armored};
use pgpilot::gpg::keyring::*;

// --- list_keys ---
#[test]
#[ignore] // slow: ~3s
fn list_keys_empty_homedir() {
    let (_dir, homedir) = setup_test_gnupghome();
    let keys = list_keys(&homedir).unwrap();
    assert!(keys.is_empty());
}

// --- create_key ---
#[test]
#[ignore]
fn create_key_returns_fingerprint() {
    let (_dir, homedir) = setup_test_gnupghome();
    let fp = create_key(&homedir, "Test User", "test@pgpilot.test").unwrap();
    assert_eq!(fp.len(), 40);
    assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
#[ignore]
fn create_key_creates_sign_encr_auth_subkeys() {
    let (_dir, homedir) = setup_test_gnupghome();
    let fp = create_key(&homedir, "Test User", "test@pgpilot.test").unwrap();
    let keys = list_keys(&homedir).unwrap();
    let key = keys.iter().find(|k| k.fingerprint == fp).unwrap();
    assert!(key.subkeys.iter().any(|sk| sk.subkey_type == Some(SubkeyType::Sign)));
    assert!(key.subkeys.iter().any(|sk| sk.subkey_type == Some(SubkeyType::Encr)));
    assert!(key.subkeys.iter().any(|sk| sk.subkey_type == Some(SubkeyType::Auth)));
}

// --- import / export ---
#[test]
fn import_key_rejects_non_pgp_text() {
    let (_dir, homedir) = setup_test_gnupghome();
    let result = import_key_from_text(&homedir, "<html>404 Not Found</html>");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("PGP"));
}

#[test]
#[ignore]
fn import_then_export_roundtrip() {
    let (_dir, homedir) = setup_test_gnupghome();
    import_armored(&homedir, fixtures::TEST_PUBLIC_KEY);
    let fps = list_fingerprints(&homedir);
    let exported = export_public_key(&homedir, &fps[0]).unwrap();
    assert!(exported.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----"));
}

// --- trust ---
#[test]
#[ignore]
fn set_key_trust_full_then_read() {
    let (_dir, homedir) = setup_test_gnupghome();
    let fp = import_armored(&homedir, fixtures::TEST_PUBLIC_KEY);
    set_key_trust(&homedir, &fp, TrustLevel::Full).unwrap();
    let keys = list_keys(&homedir).unwrap();
    let key = keys.iter().find(|k| k.fingerprint == fp).unwrap();
    assert_eq!(key.trust, TrustLevel::Full);
}

// --- sign / verify ---
#[test]
#[ignore]
fn sign_and_verify_roundtrip() {
    let (_dir, homedir) = setup_test_gnupghome();
    let fp = import_armored(&homedir, fixtures::TEST_SECRET_KEY);
    // Créer un fichier temporaire à signer
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), b"hello pgpilot").unwrap();
    let sig_path = sign_file(&homedir, file.path(), &fp).unwrap();
    let result = verify_signature(&homedir, file.path(), Some(&sig_path)).unwrap();
    assert_eq!(result.outcome, VerifyOutcome::Valid);
    assert_eq!(result.signer_fp.as_deref(), Some(&fp[24..])); // last 16 chars
}

#[test]
#[ignore]
fn verify_tampered_file_returns_bad_sig() {
    let (_dir, homedir) = setup_test_gnupghome();
    let fp = import_armored(&homedir, fixtures::TEST_SECRET_KEY);
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), b"original content").unwrap();
    let sig_path = sign_file(&homedir, file.path(), &fp).unwrap();
    std::fs::write(file.path(), b"tampered content").unwrap(); // corrompt le fichier
    let result = verify_signature(&homedir, file.path(), Some(&sig_path)).unwrap();
    assert_eq!(result.outcome, VerifyOutcome::BadSig);
}

// --- encrypt ---
#[test]
#[ignore]
fn encrypt_produces_gpg_file() {
    let (_dir, homedir) = setup_test_gnupghome();
    let fp = import_armored(&homedir, fixtures::TEST_PUBLIC_KEY);
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), b"secret data").unwrap();
    encrypt_files(&homedir, &[file.path().to_path_buf()], &[fp], false, true).unwrap();
    let gpg_path = file.path().with_extension("gpg");
    assert!(gpg_path.exists());
}

#[test]
#[ignore]
fn encrypt_collision_adds_suffix() {
    // Déjà un .gpg → doit créer _1.gpg
}
```

**Commit** : `test(gpg): integration tests for keyring operations`

---

## T3.4 — Tests `src/gpg/card.rs`

**Complexité** : M  
**Agent** : `voltagent-qa-sec:test-automator`  
**Dépendances** : T3.1

```rust
// tests/gpg_card.rs
#[test]
fn card_status_no_card_returns_error_or_none() {
    let (_dir, homedir) = setup_test_gnupghome();
    let result = card_status(&homedir);
    // Sans carte physique : soit Err soit Ok(None)
    // Le test vérifie seulement que ça ne panique pas
    let _ = result;
}
```

Tests complets de migration bloqués jusqu'à disponibilité de l'outillage (SECURITY_PLAN.md §3.4).
Marquer `#[ignore]` avec commentaire explicatif.

**Commit** : `test(gpg): smoke tests for card.rs (graceful no-card behavior)`

---

## T3.5 — Tests `src/gpg/health.rs`

**Complexité** : S  
**Agent** : `voltagent-qa-sec:test-automator`  
**Dépendances** : T3.1

```rust
// src/gpg/health.rs — module #[cfg(test)]
#[test]
fn all_checks_run_without_panic() {
    let (_dir, homedir) = setup_test_gnupghome();
    let results = run_health_checks(&homedir);
    assert_eq!(results.len(), 8); // 8 checks définis
    // Chaque check retourne un des 4 statuts valides
    for r in &results {
        assert!(matches!(r.status, HealthStatus::Ok | HealthStatus::Info | HealthStatus::Warning | HealthStatus::Error));
    }
}

#[test]
fn gpg_binary_check_passes_in_test_env() {
    // gpg doit être trouvable dans PATH de l'environnement de test
    let (_dir, homedir) = setup_test_gnupghome();
    let results = run_health_checks(&homedir);
    let gpg_check = results.iter().find(|r| r.name.contains("gpg") || r.name.contains("GPG")).unwrap();
    assert_ne!(gpg_check.status, HealthStatus::Error);
}
```

**Commit** : `test(gpg): unit tests for health diagnostics`

---

## T3.6 — Tests `src/app/` handlers

**Complexité** : L  
**Agent** : `voltagent-qa-sec:test-automator`  
**Dépendances** : T3.1, T3.3

### Approche

Pas de tests de rendu iced. Tester uniquement la logique : mutations de `App`, retour de `Task`.

```rust
// tests/app_handlers.rs
use pgpilot::app::{App, Message};
use pgpilot::gpg::types::StatusKind;

fn make_test_app(homedir: &str) -> App {
    App::new_for_test(homedir) // constructeur test-only
}
```

Ajouter à `src/app/mod.rs` :
```rust
#[cfg(test)]
impl App {
    pub fn new_for_test(homedir: &str) -> Self {
        Self {
            homedir: homedir.to_string(),
            keys: vec![],
            selected: None,
            pending: None,
            status: None,
            status_generation: 0,
            loading: false,
            view: crate::ui::View::MyKeys,
            previous_view: None,
            config: Config::default(),
            strings: i18n::strings_for(Language::English),
            // ... autres champs à default
        }
    }
}
```

Tests :
```rust
#[test]
fn key_by_fp_returns_none_for_unknown() {
    let (_dir, homedir) = setup_test_gnupghome();
    let app = make_test_app(&homedir);
    assert!(app.key_by_fp("A".repeat(40).as_str()).is_none());
}

#[test]
fn reset_pending_ops_clears_state() {
    let (_dir, homedir) = setup_test_gnupghome();
    let mut app = make_test_app(&homedir);
    app.pending = Some(PendingOp::Delete("ABCD".repeat(10)));
    app.reset_pending_ops();
    assert!(app.pending.is_none());
    assert!(app.status.is_none());
}

#[test]
fn on_nav_changed_to_create_saves_previous_view() {
    let (_dir, homedir) = setup_test_gnupghome();
    let mut app = make_test_app(&homedir);
    app.view = crate::ui::View::MyKeys;
    let _ = app.on_nav_changed(crate::ui::View::CreateKey);
    assert_eq!(app.previous_view, Some(crate::ui::View::MyKeys));
}

#[test]
#[ignore]
fn on_create_key_workflow() {
    let (_dir, homedir) = setup_test_gnupghome();
    let mut app = make_test_app(&homedir);
    app.create_form.name = "Test User".to_string();
    app.create_form.email = "test@pgpilot.test".to_string();
    // Exécuter la Task de manière synchrone en appelant directement la fonction gpg
    let fp = pgpilot::gpg::keyring::create_key(&homedir, "Test User", "test@pgpilot.test").unwrap();
    app.keys = pgpilot::gpg::keyring::list_keys(&homedir).unwrap();
    assert!(app.key_by_fp(&fp).is_some());
}
```

**Commit** : `test(app): unit tests for App handlers and state mutations`

---

## T3.7 — Intégration dans CI

**Complexité** : M  
**Agent** : `voltagent-infra:deployment-engineer`  
**Dépendances** : T3.2, T3.3, T3.4, T3.5, T3.6

### Modification `.github/workflows/ci.yml`

Ajouter après `cargo build` :

```yaml
- name: cargo test (unit)
  run: nix develop --command cargo test --lib --bins
  env:
    GNUPGHOME: /tmp/pgpilot-test-${{ github.run_id }}

- name: cargo test (integration, slow)
  run: nix develop --command cargo test -- --ignored
  timeout-minutes: 10
  env:
    GNUPGHOME: /tmp/pgpilot-test-slow-${{ github.run_id }}
  continue-on-error: false
```

**Note** : `continue-on-error: false` — les tests lents sont obligatoires en CI.

**Commit** : `ci: integrate cargo test into GitHub Actions workflow`

---

## T3.8 — Documentation des tests

**Complexité** : M  
**Agent** : `voltagent-biz:technical-writer`  
**Dépendances** : T3.7

### Sections à ajouter dans `CLAUDE.md`

```markdown
## Testing

### Lancer les tests

```bash
cargo test                      # tests unitaires rapides
cargo test -- --ignored         # + tests d'intégration GPG (lents, ~30s)
cargo test -- --nocapture       # avec logs stdout
cargo test gpg_keyring          # un seul fichier de tests
```

### Structure

- `tests/common/mod.rs` — helpers partagés (setup_test_gnupghome, import_armored)
- `tests/fixtures/` — clefs PGP de test (armored text)
- `tests/gpg_keyring.rs` — intégration keyring
- `tests/gpg_card.rs` — smartcard (smoke tests)
- `tests/app_handlers.rs` — logique app sans GUI

### Conventions

- Tests lents (création de clef GPG > 1s) → `#[ignore]`
- Toujours utiliser `setup_test_gnupghome()` — jamais `$GNUPGHOME` réel
- TempDir doit rester en scope pendant tout le test
- Pas de mocks GPG — vrais processus gpg dans homedir temporaire
```

**Commit** : `docs(testing): add testing guidelines to CLAUDE.md`

---

## T3.9 — Vérification finale CI

**Complexité** : S  
**Agent** : `voltagent-qa-sec:test-automator`  
**Dépendances** : T3.7, T3.8

- Push vers `main` → vérifier CI `cargo test` vert
- Corriger les flaky tests si présents
- Commit: `test: fix flaky tests and verify CI passing`

---

## Fichiers créés / modifiés

**Nouveaux** :
```
tests/common/mod.rs
tests/fixtures/mod.rs
tests/fixtures/test_secret.asc
tests/fixtures/test_public.asc
tests/fixtures/third_party_public.asc
tests/gpg_keyring.rs
tests/gpg_card.rs
tests/gpg_health.rs
tests/app_handlers.rs
```

**Modifiés** :
```
Cargo.toml                      (+tempfile en [dev-dependencies])
.github/workflows/ci.yml        (+ cargo test steps)
src/app/mod.rs                  (+ new_for_test constructeur)
src/gpg/types.rs                (+ #[cfg(test)] module)
src/gpg/health.rs               (+ #[cfg(test)] module)
CLAUDE.md                       (+ section Testing)
```

## Critères d'acceptation

- [ ] `cargo test --lib --bins` ✓ (tests rapides)
- [ ] `cargo test -- --ignored` ✓ (tests lents)
- [ ] CI GitHub Actions : étapes `cargo test` vertes
- [ ] Aucun test qui touche le `$GNUPGHOME` réel de l'utilisateur
- [ ] Coverage ≥ 60 % sur `src/gpg/` et `src/app/` (mesuré via `cargo tarpaulin` optionnel)
- [ ] `CLAUDE.md` section Testing à jour
