# Axe 1 — Corrections i18n

## Objectif

Éliminer tous les textes français hardcodés dans le code UI anglais. Les textes passent par le
trait `Strings` — aucun string littéral français ne doit subsister dans `src/ui/` quand
la langue est `Language::English`.

Aussi : bump `Cargo.toml` vers `v0.6.0`.

---

## T1.1 — Audit exhaustif

**Complexité** : S
**Agent** : `Explore` (subagent_type=Explore, thorough)
**Dépendances** : aucune

### Ce qui est à faire

Scanner l'intégralité du codebase et produire trois listes :

1. **Méthodes de `EnglishStrings`** dont la valeur retournée contient des mots français
   (lire `src/i18n/english.rs` en entier)

2. **String literals français hardcodés dans `src/ui/*.rs`** — pas routés via `s.xxx()`
   (chercher des patterns comme `"Sur`, `"Chiffr`, `"Publique`, `"Nom /`, `"État`, `"Expire`, `"Sélection`)

3. **Appels `theme::flavor(_, ussr_text)`** où `ussr_text` est en français
   (chercher `flavor(` dans tous les fichiers UI)

**Format de sortie attendu** : pour chaque item trouvé :
```
fichier:ligne | valeur actuelle | traduction anglaise proposée
```

**Commit** : aucun — livrable = liste uniquement

---

## T1.2 — Implémentation

**Complexité** : M
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T1.1 (utilise sa liste comme spec exacte)

### Ce qui est à faire

**1. Bump version**
- `Cargo.toml` ligne `version` : `"0.5.1"` → `"0.6.0"`

**2. Nouvelles méthodes dans le trait `Strings` (`src/i18n/mod.rs`)**

Ajouter exactement ces méthodes (plus celles identifiées par T1.1) :

```rust
fn key_list_header_name(&self) -> &'static str;
fn key_list_header_expires(&self) -> &'static str;
fn key_list_header_status(&self) -> &'static str;
fn key_type_on_card(&self) -> &'static str;
fn key_type_public_private(&self) -> &'static str;
fn key_type_public_only(&self) -> &'static str;
fn subkey_type_signature(&self) -> &'static str;
fn subkey_type_encryption(&self) -> &'static str;
fn subkey_type_ssh_auth(&self) -> &'static str;
fn export_menu_save_disk(&self) -> &'static str;
fn export_menu_copy_clipboard(&self) -> &'static str;
fn export_menu_paste_link(&self) -> &'static str;
```

**3. Implémentation `EnglishStrings` (`src/i18n/english.rs`)**

```
key_list_header_name      → "Name / Email"
key_list_header_expires   → "Expires"
key_list_header_status    → "Status"
key_type_on_card          → "On YubiKey"
key_type_public_private   → "Public + Private"
key_type_public_only      → "Public"
subkey_type_signature     → "Signature"
subkey_type_encryption    → "Encryption"
subkey_type_ssh_auth      → "SSH Auth"
export_menu_save_disk     → "Save to disk"
export_menu_copy_clipboard → "Copy to clipboard"
export_menu_paste_link    → "Get public link (paste.rs)"
```

**4. Implémentation `FrenchStrings` (`src/i18n/french.rs`)**

```
key_list_header_name      → "Nom / Email"
key_list_header_expires   → "Expire"
key_list_header_status    → "État"
key_type_on_card          → "Sur YubiKey"
key_type_public_private   → "Publique + Privée"
key_type_public_only      → "Publique"
subkey_type_signature     → "Signature"
subkey_type_encryption    → "Chiffrement"
subkey_type_ssh_auth      → "Auth SSH"
export_menu_save_disk     → "Enregistrer sur le disque"
export_menu_copy_clipboard → "Copier dans le presse-papier"
export_menu_paste_link    → "Obtenir un lien public (paste.rs)"
```

**5. Remplacement des hardcodes UI**

`src/ui/key_list.rs` :
- `"Nom / Email"` → `s.key_list_header_name()`
- `"Expire"` → `s.key_list_header_expires()`
- `"État"` → `s.key_list_header_status()`
- Toute erreur hardcodée en FR → méthode i18n existante ou nouvelle

`src/ui/key_detail.rs` :
- `"Enregistrer sur le disque"` → `s.export_menu_save_disk()`
- `"Copier dans le presse-papier"` → `s.export_menu_copy_clipboard()`
- `"Obtenir un lien public (paste.rs)"` → `s.export_menu_paste_link()`
- `"Sur YubiKey"` → `s.key_type_on_card()`
- `"Publique + Privée"` → `s.key_type_public_private()`
- `"Publique"` (standalone) → `s.key_type_public_only()`
- `"Signature"` (subkey) → `s.subkey_type_signature()`
- `"Chiffrement"` → `s.subkey_type_encryption()`
- `"Auth SSH"` → `s.subkey_type_ssh_auth()`

Plus tous les items trouvés par T1.1 non listés ici.

**6. Textes USSR en français → anglais soviétique**

Pour chaque `theme::flavor(normal, ussr_text)` où `ussr_text` est en français, remplacer par
l'équivalent anglais en gardant le ton soviétique. Exemples de registre :
- "Workers of the world, unite!"
- "Comrade, your key serves the People!"
- "Glory to the People's Cryptography!"
- "The Party approves this operation!"
- "For the Motherland!"

**7. Format**
```bash
cargo fmt -- --config tab_spaces=2
```

**Commit** : `feat(i18n): translate remaining French strings to English, bump to v0.6.0`

---

## T1.3 — Tests unitaires i18n

**Complexité** : S
**Agent** : `voltagent-qa-sec:test-automator`
**Dépendances** : T1.2

### Fichier : `tests/i18n.rs`

```rust
use pgpilot::i18n::{strings_for, Language};

fn en() -> &'static dyn pgpilot::i18n::Strings {
    strings_for(Language::English)
}

fn fr() -> &'static dyn pgpilot::i18n::Strings {
    strings_for(Language::French)
}

// Toutes les méthodes retournent une chaîne non vide
#[test]
fn english_strings_all_non_empty() {
    let s = en();
    assert!(!s.key_list_header_name().is_empty());
    assert!(!s.key_list_header_expires().is_empty());
    assert!(!s.key_list_header_status().is_empty());
    assert!(!s.key_type_on_card().is_empty());
    assert!(!s.key_type_public_private().is_empty());
    assert!(!s.key_type_public_only().is_empty());
    assert!(!s.subkey_type_signature().is_empty());
    assert!(!s.subkey_type_encryption().is_empty());
    assert!(!s.subkey_type_ssh_auth().is_empty());
    assert!(!s.export_menu_save_disk().is_empty());
    assert!(!s.export_menu_copy_clipboard().is_empty());
    assert!(!s.export_menu_paste_link().is_empty());
    // ... toutes les autres méthodes du trait
}

#[test]
fn french_strings_all_non_empty() {
    let s = fr();
    // même liste
}

// EN et FR diffèrent pour les méthodes qui doivent être traduites
#[test]
fn english_differs_from_french() {
    assert_ne!(en().key_list_header_name(), fr().key_list_header_name());
    assert_ne!(en().key_list_header_expires(), fr().key_list_header_expires());
    assert_ne!(en().key_list_header_status(), fr().key_list_header_status());
    assert_ne!(en().key_type_on_card(), fr().key_type_on_card());
    assert_ne!(en().key_type_public_private(), fr().key_type_public_private());
    assert_ne!(en().subkey_type_encryption(), fr().subkey_type_encryption());
    assert_ne!(en().subkey_type_ssh_auth(), fr().subkey_type_ssh_auth());
    assert_ne!(en().export_menu_save_disk(), fr().export_menu_save_disk());
    assert_ne!(en().export_menu_copy_clipboard(), fr().export_menu_copy_clipboard());
    assert_ne!(en().export_menu_paste_link(), fr().export_menu_paste_link());
}

// Les strings EN ne contiennent pas de mots français caractéristiques
#[test]
fn english_strings_contain_no_french_words() {
    let french_markers = ["sur ", "chiffr", "publique", "privée", "sélection",
                          "état", "clef", "signatur", "obten", "copier", "enregist"];
    let s = en();
    let all_strings = [
        s.key_list_header_name(), s.key_list_header_expires(), s.key_list_header_status(),
        s.key_type_on_card(), s.key_type_public_private(), s.key_type_public_only(),
        s.subkey_type_signature(), s.subkey_type_encryption(), s.subkey_type_ssh_auth(),
        s.export_menu_save_disk(), s.export_menu_copy_clipboard(), s.export_menu_paste_link(),
    ];
    for string in &all_strings {
        let lower = string.to_lowercase();
        for marker in &french_markers {
            assert!(
                !lower.contains(marker),
                "English string {:?} contains French word {:?}", string, marker
            );
        }
    }
}
```

**Commit** : `test(i18n): completeness and language correctness tests`

---

## T1.4 — Revue de code

**Complexité** : S
**Agent** : `voltagent-qa-sec:code-reviewer`
**Dépendances** : T1.2, T1.3

### Ce qui est à vérifier

1. Aucun string littéral français ne subsiste dans `src/ui/` (grep `"[A-ZÀÂÉÈÊËÎÏÔÙÛÜ][a-zàâéèêëîïôùûü]` dans les literals)
2. Toutes les méthodes ajoutées au trait ont une implémentation EN **et** FR (pas de `todo!()`)
3. Les textes USSR anglais gardent un ton soviétique cohérent
4. `cargo clippy -- -D warnings` serait vert (pas d'exécution, analyse statique)
5. Pas de régression sur les méthodes existantes du trait (comparer avec FR)

**Output** : liste de points à corriger ou "✓ RAS"

---

## T1.5 — Validation utilisateur

**Qui** : Guillaume (utilisateur)

```bash
nix develop
cargo build
cargo test
cargo run
```

Checklist manuelle dans l'app :
- [ ] Settings → Language → English : aucun texte français visible dans MY KEYS, detail panel, menu export, types de sous-clefs
- [ ] Settings → Language → French : aucune régression (textes FR corrects)
- [ ] Theme USSR en anglais : flavor texts en anglais avec ton soviétique
- [ ] `cargo clippy -- -D warnings` ✓
- [ ] `cargo fmt -- --check` ✓

**Gate** : validation OK → commit + push → démarrer Axe 2

---

## Fichiers modifiés

```
Cargo.toml                    (version 0.5.1 → 0.6.0)
src/i18n/mod.rs               (+ 12 méthodes dans le trait Strings)
src/i18n/english.rs           (+ 12 implémentations EN)
src/i18n/french.rs            (+ 12 implémentations FR)
src/ui/key_list.rs            (remplacement ~4 hardcodes)
src/ui/key_detail.rs          (remplacement ~9 hardcodes)
src/ui/*.rs                   (autres hardcodes identifiés par T1.1)
tests/i18n.rs                 (nouveau — tests i18n)
```

## Critères d'acceptation

- [ ] `cargo build` ✓
- [ ] `cargo test` ✓ (incluant `tests/i18n.rs`)
- [ ] `cargo clippy -- -D warnings` ✓
- [ ] Zero string littéral français dans `src/ui/` en mode Language::English
- [ ] Toutes les méthodes du trait ont impl EN + FR
