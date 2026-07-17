# Axe 9 — Utilitaires : Expiry audit + Revocation manager

## Objectif

Deux petites features indépendantes, réalisables en parallèle de l'axe 1, incluses dans le
même commit de la Famille A.

---

## T9.1 — Expiry audit + rappels au lancement

**Complexité** : S
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : aucune (réutilise `KeyExpiry` et `SubkeyInfo` existants)

### Comportement

Au lancement, après `KeysLoaded`, scanner toutes les clefs et sous-clefs. Si une sous-clef
expire dans moins de **90 jours**, afficher une bannière d'avertissement en haut de `View::MyKeys`.

La bannière liste les clefs concernées avec un lien direct (clic → sélectionne la clef).

### Implémentation

**`src/app/nav.rs`** — dans `on_keys_loaded` :

```rust
fn on_keys_loaded(&mut self, result: Result<Vec<KeyInfo>, String>) -> Task<Message> {
    // ... logique existante ...
    self.expiry_warnings = self.compute_expiry_warnings();
    Task::none()
}

fn compute_expiry_warnings(&self) -> Vec<ExpiryWarning> {
    let threshold = chrono::Utc::now() + chrono::Duration::days(90);
    let mut warnings = vec![];
    for key in &self.keys {
        for subkey in &key.subkeys {
            if let Some(exp) = subkey.expiry_date() {
                if exp < threshold {
                    warnings.push(ExpiryWarning {
                        key_fp: key.fingerprint.clone(),
                        key_name: key.name.clone(),
                        subkey_fp: subkey.fingerprint.clone(),
                        subkey_type: subkey.subkey_type,
                        expires_at: exp,
                    });
                }
            }
        }
    }
    warnings
}
```

**`src/app/mod.rs`** — nouveau champ :
```rust
pub expiry_warnings: Vec<ExpiryWarning>,
```

**`src/ui/key_list.rs`** — bannière en haut :
```rust
if !app.expiry_warnings.is_empty() {
    // Bannière warning_bg(), icône ⚠, liste des sous-clefs avec bouton "Renew"
    // Clic sur une ligne → Message::KeySelected(key_fp)
}
```

**Nouveaux types** (`src/gpg/types.rs` ou `src/app/mod.rs`) :
```rust
pub struct ExpiryWarning {
    pub key_fp: String,
    pub key_name: String,
    pub subkey_fp: String,
    pub subkey_type: Option<SubkeyType>,
    pub expires_at: chrono::DateTime<Utc>,
}
```

**i18n** — nouvelles méthodes :
```rust
fn expiry_warning_banner(&self) -> &'static str;   // "X subkey(s) expiring within 90 days"
fn expiry_warning_renew(&self) -> &'static str;    // "Renew"
```

**Commit** : `feat(keys): expiry audit banner on launch, link to affected keys`

---

## T9.2 — Revocation certificate manager

**Complexité** : S
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : aucune (logique déjà dans `backup_key`)

### Comportement

Dans le detail panel d'une **clef privée** (`src/ui/key_detail.rs`), ajouter une section
"Revocation certificate" entre les action buttons et les subkeys :

```
┌─ Revocation Certificate ──────────────────┐
│  ✓ Found: ~/.gnupg/openpgp-revocs.d/      │
│  <fp>.rev                                  │
│  [Export .rev]  [Copy path]                │
└───────────────────────────────────────────┘
```

Si absent :
```
┌─ Revocation Certificate ──────────────────┐
│  ⚠ Not found                              │
│  [Generate]                               │
└───────────────────────────────────────────┘
```

### Implémentation

**`src/gpg/keyring.rs`** — nouvelle fonction :

```rust
/// Retourne le chemin du certificat de révocation si présent.
pub fn revocation_cert_path(homedir: &str, fp: &str) -> Result<Option<PathBuf>, String> {
    validate_fp(fp)?;
    let dir = PathBuf::from(homedir);
    let path = dir.join("openpgp-revocs.d").join(format!("{fp}.rev"));
    Ok(if path.exists() { Some(path) } else { None })
}

/// Génère un certificat de révocation via gpg --gen-revoke.
pub fn generate_revocation_cert(homedir: &str, fp: &str) -> Result<PathBuf, String>;
```

**`src/app/mod.rs`** — nouveaux `Message` :
```rust
Message::ExportRevocationCert(String),    // fp
Message::GenerateRevocationCert(String),  // fp
Message::RevocationCertGenerated(Result<PathBuf, String>),
```

**`src/app/card.rs`** (ou nouveau `src/app/revocation.rs`) — handlers :
```rust
pub(super) fn on_export_revocation_cert(&mut self, fp: String) -> Task<Message>;
pub(super) fn on_generate_revocation_cert(&mut self, fp: String) -> Task<Message>;
```

**`src/ui/key_detail.rs`** — section dans `left_column_items` :
```rust
fn revocation_cert_section(key: &KeyInfo, homedir: &str, s: &'static dyn Strings) -> Element<Message>;
```

**i18n** — nouvelles méthodes :
```rust
fn revocation_cert_title(&self) -> &'static str;    // "Revocation Certificate"
fn revocation_cert_found(&self) -> &'static str;    // "Certificate found"
fn revocation_cert_missing(&self) -> &'static str;  // "Certificate not found"
fn revocation_cert_export(&self) -> &'static str;   // "Export .rev"
fn revocation_cert_generate(&self) -> &'static str; // "Generate"
fn revocation_cert_copy_path(&self) -> &'static str; // "Copy path"
```

**Commit** : `feat(keys): revocation certificate section in key detail panel`

---

## Fichiers modifiés

```
src/gpg/types.rs         (+ ExpiryWarning struct)
src/gpg/keyring.rs       (+ revocation_cert_path, generate_revocation_cert)
src/app/mod.rs           (+ expiry_warnings field, + Message variants)
src/app/nav.rs           (+ compute_expiry_warnings dans on_keys_loaded)
src/app/card.rs          (+ on_export_revocation_cert, on_generate_revocation_cert)
src/ui/key_list.rs       (+ bannière expiry)
src/ui/key_detail.rs     (+ section revocation cert)
src/i18n/mod.rs          (+ 8 nouvelles méthodes)
src/i18n/english.rs      (+ implémentations EN)
src/i18n/french.rs       (+ implémentations FR)
```

## Critères d'acceptation

### Expiry audit
- [ ] Clef avec sous-clef expirant dans 30 jours → bannière visible au lancement
- [ ] Clic sur la ligne → clef sélectionnée dans le detail panel
- [ ] Clef sans expiration prochaine → pas de bannière
- [ ] `cargo build` ✓

### Revocation manager
- [ ] Clef avec `openpgp-revocs.d/<fp>.rev` → section "Found" avec bouton Export
- [ ] Clef sans certificat → section "Not found" avec bouton Generate
- [ ] Export → file dialog → fichier `.rev` copié dans le dossier choisi
- [ ] Generate → `gpg --gen-revoke` → section passe à "Found"
- [ ] `cargo build` ✓
