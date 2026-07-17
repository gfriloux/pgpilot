# Axe 2 — Internationalisation (i18n)

## Objectif

1. Passer tous les textes UI du français à l'**anglais** (langue du code)
2. Supporter **anglais + français** sélectionnable par l'utilisateur
3. Config persistante YAML en `~/.config/pgpilot/config.yaml` (XDG)
4. Détection automatique de la locale système si pas de config

## Stratégie technique

- Pas de fichiers `.po` / gettext — contrainte iced (UI 100 % Rust)
- Trait `Strings` avec une impl par langue — méthodes statiques, zéro allocation
- Config YAML via `serde_yaml` — extensible pour futures prefs
- Détection locale via `$LANG` / `$LC_ALL` — fallback `en`

---

## T2.1 — Architecture i18n + module config

**Complexité** : M  
**Agent** : `voltagent-lang:rust-engineer`  
**Dépendances** : aucune

### Fichiers à créer

**`src/i18n/mod.rs`** :
```rust
pub mod english;
pub mod french;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum Language {
    #[default]
    English,
    French,
}

pub trait Strings: Send + Sync {
    // Navigation
    fn nav_my_keys(&self) -> &'static str;
    fn nav_public_keys(&self) -> &'static str;
    fn nav_import(&self) -> &'static str;
    fn nav_create_key(&self) -> &'static str;
    fn nav_encrypt(&self) -> &'static str;
    fn nav_sign(&self) -> &'static str;
    fn nav_verify(&self) -> &'static str;
    fn nav_health(&self) -> &'static str;
    fn nav_settings(&self) -> &'static str;

    // Boutons communs
    fn btn_ok(&self) -> &'static str;
    fn btn_cancel(&self) -> &'static str;
    fn btn_confirm(&self) -> &'static str;
    fn btn_back(&self) -> &'static str;
    fn btn_create(&self) -> &'static str;
    fn btn_delete(&self) -> &'static str;
    fn btn_export(&self) -> &'static str;
    fn btn_import(&self) -> &'static str;
    fn btn_copy(&self) -> &'static str;
    fn btn_publish(&self) -> &'static str;
    fn btn_backup(&self) -> &'static str;
    fn btn_migrate(&self) -> &'static str;
    fn btn_renew(&self) -> &'static str;
    fn btn_rotate(&self) -> &'static str;
    fn btn_add_subkey(&self) -> &'static str;

    // ... ~150 méthodes supplémentaires (voir ci-dessous)
    // Clefs / détail
    fn key_fingerprint(&self) -> &'static str;
    fn key_created(&self) -> &'static str;
    fn key_expires(&self) -> &'static str;
    fn key_never_expires(&self) -> &'static str;
    fn key_trust(&self) -> &'static str;
    fn key_subkeys(&self) -> &'static str;
    fn key_no_subkeys(&self) -> &'static str;

    // Trust levels
    fn trust_undefined(&self) -> &'static str;
    fn trust_marginal(&self) -> &'static str;
    fn trust_full(&self) -> &'static str;
    fn trust_ultimate(&self) -> &'static str;

    // Status messages
    fn status_key_created(&self) -> &'static str;
    fn status_key_deleted(&self) -> &'static str;
    fn status_key_exported(&self) -> &'static str;
    fn status_key_imported(&self) -> &'static str;
    fn status_published(&self) -> &'static str;
    fn status_publish_failed(&self) -> &'static str;
    fn status_backup_done(&self) -> &'static str;
    fn status_preferences_saved(&self) -> &'static str;

    // Erreurs
    fn err_gpg_not_found(&self) -> &'static str;
    fn err_invalid_key(&self) -> &'static str;
    fn err_import_not_pgp(&self) -> &'static str;
    fn err_export_failed(&self) -> &'static str;

    // Encrypt
    fn encrypt_title(&self) -> &'static str;
    fn encrypt_add_files(&self) -> &'static str;
    fn encrypt_recipients(&self) -> &'static str;
    fn encrypt_no_recipients(&self) -> &'static str;
    fn encrypt_trust_warning_title(&self) -> &'static str;
    fn encrypt_trust_warning_body(&self) -> &'static str;
    fn encrypt_format_binary(&self) -> &'static str;
    fn encrypt_format_armor(&self) -> &'static str;

    // Sign / Verify
    fn sign_title(&self) -> &'static str;
    fn sign_select_file(&self) -> &'static str;
    fn sign_select_key(&self) -> &'static str;
    fn verify_title(&self) -> &'static str;
    fn verify_select_file(&self) -> &'static str;
    fn verify_outcome_valid(&self) -> &'static str;
    fn verify_outcome_bad_sig(&self) -> &'static str;
    fn verify_outcome_unknown_key(&self) -> &'static str;
    fn verify_outcome_expired_key(&self) -> &'static str;
    fn verify_outcome_revoked_key(&self) -> &'static str;

    // Health / Diagnostic
    fn health_title(&self) -> &'static str;
    fn health_ok(&self) -> &'static str;
    fn health_warning(&self) -> &'static str;
    fn health_error(&self) -> &'static str;
    fn health_info(&self) -> &'static str;

    // Import
    fn import_title(&self) -> &'static str;
    fn import_tab_file(&self) -> &'static str;
    fn import_tab_url(&self) -> &'static str;
    fn import_tab_keyserver(&self) -> &'static str;
    fn import_tab_paste(&self) -> &'static str;

    // Keyserver
    fn keyserver_openpgp(&self) -> &'static str;
    fn keyserver_ubuntu(&self) -> &'static str;
    fn keyserver_status_unknown(&self) -> &'static str;
    fn keyserver_status_published(&self) -> &'static str;
    fn keyserver_status_not_published(&self) -> &'static str;

    // Settings
    fn settings_title(&self) -> &'static str;
    fn settings_language(&self) -> &'static str;
    fn settings_language_english(&self) -> &'static str;
    fn settings_language_french(&self) -> &'static str;
}

pub fn strings_for(lang: Language) -> &'static dyn Strings {
    match lang {
        Language::English => &english::EnglishStrings,
        Language::French => &french::FrenchStrings,
    }
}
```

**`src/i18n/english.rs`** :
```rust
use super::Strings;
pub struct EnglishStrings;
impl Strings for EnglishStrings {
    fn nav_my_keys(&self) -> &'static str { "My Keys" }
    fn nav_public_keys(&self) -> &'static str { "Public Keys" }
    // ... toutes les méthodes en anglais
}
```

**`src/i18n/french.rs`** :
```rust
use super::Strings;
pub struct FrenchStrings;
impl Strings for FrenchStrings {
    fn nav_my_keys(&self) -> &'static str { "Mes clefs" }
    fn nav_public_keys(&self) -> &'static str { "Clefs publiques" }
    // ... textes actuels du code (v0.4.x)
}
```

**`src/config/mod.rs`** :
```rust
use serde::{Deserialize, Serialize};
use crate::i18n::Language;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub language: Language,
}

impl Default for Config {
    fn default() -> Self {
        Self { language: detect_system_language() }
    }
}

impl Config {
    pub fn load() -> Result<Self, String> {
        // Lire ~/.config/pgpilot/config.yaml
        // Si absent → Config::default()
        // Si présent mais malformé → Err(...)
    }
    pub fn save(&self) -> Result<(), String> {
        // Écrire ~/.config/pgpilot/config.yaml
        // Créer ~/.config/pgpilot/ si absent (std::fs::create_dir_all)
    }
    pub fn path() -> std::path::PathBuf {
        // dirs::config_dir() / "pgpilot" / "config.yaml"
        // Fallback : $HOME/.config/pgpilot/config.yaml
    }
}

fn detect_system_language() -> Language {
    // Lire $LANG ou $LC_ALL
    // Si "fr" prefix → French
    // Sinon → English
}
```

### Dépendances Cargo à ajouter dans `Cargo.toml`

```toml
serde_yaml = "0.9"
dirs = "5"
```

**Commit** : `feat(i18n): add Language enum, Strings trait, and config module`

---

## T2.2 — Refactor App struct + passage du contexte i18n

**Complexité** : L  
**Agent** : `voltagent-lang:rust-engineer`  
**Dépendances** : T2.1

### Modifications `src/app/mod.rs`

```rust
pub struct App {
    // ... champs existants ...
    pub config: Config,
    pub strings: &'static dyn Strings,
}
```

Dans `App::new` :
```rust
let config = Config::load().unwrap_or_default();
let strings = i18n::strings_for(config.language);
```

Ajouter à l'enum `Message` :
```rust
Message::ChangeLanguage(Language),
```

Handler `on_language_changed` dans `src/app/card.rs` ou nouveau `src/app/settings.rs` :
```rust
pub(super) fn on_language_changed(&mut self, lang: Language) -> Task<Message> {
    self.config.language = lang;
    self.strings = i18n::strings_for(lang);
    if let Err(e) = self.config.save() {
        return self.set_status(StatusKind::Error, format!("Config save error: {e}"));
    }
    self.set_status(StatusKind::Success, self.strings.status_preferences_saved().to_string())
}
```

### Passage du contexte dans les views

Toutes les fonctions `view()` reçoivent `app: &App` → accèdent à `app.strings`.

**Commit** : `refactor(app): add language/config fields and wire ChangeLanguage message`

---

## T2.3 — Extraire tous les textes → EnglishStrings

**Complexité** : L  
**Agent** : `voltagent-lang:rust-engineer`  
**Dépendances** : T2.2

### Périmètre — fichiers à toucher

Tous les `text("...")`, `button("...")`, `format!("Erreur...")`, messages de status :

```
src/app/export.rs   src/app/import.rs   src/app/create.rs
src/app/subkeys.rs  src/app/keyserver.rs src/app/encrypt.rs
src/app/sign.rs     src/app/card.rs      src/app/nav.rs
src/ui/key_list.rs  src/ui/key_detail.rs src/ui/create_key.rs
src/ui/import.rs    src/ui/health.rs     src/ui/encrypt.rs
src/ui/sign.rs      src/ui/verify.rs     src/ui/mod.rs
```

### Exemple de transformation

```rust
// AVANT
text("Mes clefs")
button("Créer").on_press(Message::CreateKey)
self.set_status(StatusKind::Error, format!("Erreur d'export : {e}"))

// APRÈS
text(app.strings.nav_my_keys())
button(app.strings.btn_create()).on_press(Message::CreateKey)
self.set_status(StatusKind::Error, format!("{}: {e}", app.strings.err_export_failed()))
```

**Important** : `cargo clippy -D warnings` doit passer à chaque module terminé — commits atomiques.

**Commit** : `feat(i18n): replace hardcoded French strings with EnglishStrings methods`

---

## T2.4 — Implémenter FrenchStrings

**Complexité** : M  
**Agent** : `voltagent-lang:rust-engineer`  
**Dépendances** : T2.3

Implémenter toutes les méthodes du trait `Strings` dans `src/i18n/french.rs` en utilisant les
textes issus du code v0.4.x (les textes français retirés en T2.3 deviennent la base de cette impl).

**Commit** : `feat(i18n): implement complete French language strings`

---

## T2.5 — Vue Settings avec language picker

**Complexité** : M  
**Agent** : `voltagent-core-dev:fullstack-developer`  
**Dépendances** : T2.2, T2.3, T2.4

### Modifications

1. Ajouter `View::Settings` à l'enum `View` dans `src/app/mod.rs`
2. Ajouter entrée "Settings" / "Paramètres" dans la sidebar (`src/ui/mod.rs`), section OUTILS
3. Créer `src/ui/settings.rs` :
   - Layout identique aux autres full-page views (outer container SIDEBAR_BG + inner scrollable card)
   - Afficher deux radio buttons : `○ English` / `○ Français`
   - Au clic : `Message::ChangeLanguage(lang)`
4. Ajouter routing dans `update()` et `view()` dans `src/app/mod.rs` et `src/ui/mod.rs`

### Navigation Settings

Pas de `previous_view` nécessaire — Settings est une vue terminale (bouton "back" renvoie à `View::MyKeys`).

**Commit** : `feat(ui): add Settings view with language picker`

---

## T2.6 — Détection locale système + fallback

**Complexité** : S  
**Agent** : `voltagent-lang:rust-engineer`  
**Dépendances** : T2.1

Implémenter `detect_system_language()` dans `src/config/mod.rs` :

```rust
fn detect_system_language() -> Language {
    let locale = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_default();
    if locale.starts_with("fr") {
        Language::French
    } else {
        Language::English
    }
}
```

**Commit** : `feat(i18n): detect system locale on first launch`

---

## T2.7 — Tests i18n + validation clippy

**Complexité** : M  
**Agent** : `voltagent-qa-sec:test-automator`  
**Dépendances** : T2.2, T2.3, T2.4, T2.5, T2.6

### Tests à écrire dans `src/config/mod.rs` (#[cfg(test)])

```rust
#[test]
fn config_default_language_from_env() { ... }

#[test]
fn config_save_and_load_roundtrip() {
    // Écrire une config avec TempDir, relire, vérifier language
}

#[test]
fn config_missing_file_returns_default() { ... }

#[test]
fn config_malformed_yaml_returns_error() { ... }
```

### Vérifications

- `cargo clippy -D warnings` ✓
- `cargo test` ✓
- `cargo fmt --check` ✓
- Test manuel : changer langue dans Settings → redémarrer → langue persistée

**Commit** : `test(i18n): unit tests for config load/save/detect`

---

## T2.8 — Merge + mise à jour CLAUDE.md

**Complexité** : S  
**Agent** : `voltagent-core-dev:fullstack-developer`  
**Dépendances** : T2.7

Mettre à jour `CLAUDE.md` section "Architecture" :
- Mentionner `src/i18n/` et `src/config/`
- Mentionner que la langue du code est l'anglais, FR via `FrenchStrings`
- Mention config YAML XDG

**Commit** : `feat(i18n): merge English/French i18n with persistent config (v0.5.0)`

---

## Fichiers créés / modifiés

**Nouveaux** :
```
src/i18n/mod.rs
src/i18n/english.rs
src/i18n/french.rs
src/config/mod.rs
src/ui/settings.rs
```

**Modifiés** :
```
Cargo.toml                 (+serde_yaml, +dirs)
src/app/mod.rs             (App struct, Message enum, update router)
src/app/*.rs               (tous les handlers — passages strings)
src/ui/mod.rs              (sidebar Settings entry, view routing)
src/ui/*.rs                (tous — text() via app.strings)
CLAUDE.md
```

## Critères d'acceptation

- [ ] `cargo clippy -D warnings` ✓
- [ ] `cargo test` ✓ (config roundtrip)
- [ ] UI entièrement traduite EN + FR (aucun texte français codé en dur dans le code Rust)
- [ ] Settings view : language picker fonctionnel
- [ ] Config persiste : redémarrage → langue OK
- [ ] Si pas de config : locale système détectée, fallback EN
- [ ] Aucune régression fonctionnelle (export, import, encrypt, sign, verify, health)
