# Plan — Implémentation thème USSR v2 (post-mockups)

## Context

Les mockups du thème USSR ont été intégralement validés (`tmp/mockups/01–11`). Ce plan implémente les changements visuels dans le code Rust/iced pour que l'app corresponde exactement aux mockups. Le thème USSR existait déjà partiellement (`src/ui/theme.rs`) mais manquait de nombreux éléments : bannières propagandistes, badges SVG, séparateurs étoile, nouveau settings, chips destinataires Encrypt, etc.

**Contraintes techniques confirmées :**
- Bannières PNG → `include_bytes!()` dans le binaire
- Scale factor → iced `slider` + tick marks textuels
- Aucun `widget::image` ni `widget::svg` n'existe actuellement dans le code

---

## Phase 1 — Assets et module ussr_assets

**Nouveau fichier : `src/ui/ussr_assets.rs`**

Bundler les bannières utilisées et les badges SVG via `include_bytes!()` + `once_cell::sync::Lazy` pour les handles :

```rust
// Bannières (image::Handle)
pub fn banner_handle(id: u8) -> image::Handle { ... }
// Séparateur étoile (svg::Handle)
pub fn sep_star_rouge() -> svg::Handle { ... }
// Badges (svg::Handle)
pub fn badge_keyserver() -> svg::Handle { ... }
pub fn badge_yubikey()   -> svg::Handle { ... }
pub fn badge_trust_full()     -> svg::Handle { ... }
pub fn badge_trust_marginal() -> svg::Handle { ... }
pub fn badge_trust_undef()    -> svg::Handle { ... }
```

**Assets à copier vers `assets/banners/` :**
Bannières utilisées par vue (12 fichiers depuis `tmp/theme_assets/bannieres/`) :

| Vue | Fichier source |
|-----|---------------|
| Mes Clefs (liste) | 18.png |
| Mes Clefs (pied détail) | 25.png |
| Clefs Publiques (liste) | 23.png |
| Clefs Publiques (pied détail) | 17.png |
| Vérifier | 17.png (même) |
| Signer | 20.png |
| Déchiffrer | 19.png |
| Chiffrer | 16.png |
| Importer | 24.png |
| Créer une clef | 27.png |
| Diagnostic | 12.png |
| Paramètres | 29.png |
| Chat (rooms panel) | 26.png |

**Assets à copier vers `assets/` :**
- `badge_keyserver.svg`, `badge_yubikey.svg`
- `badge_trust_full.svg`, `badge_trust_marginal.svg`, `badge_trust_undef.svg`
- `sep_etoile_rouge.svg`

---

## Phase 2 — Fondations partagées

### `src/ui/common.rs`

1. **Standardiser les card widths à 700px** (mockup validé) :
   - `card_narrow` : 480 → garder (Settings est OK à 700 de toute façon)
   - `card_medium` : 560 → 700
   - `card_wide` : max 760 → max 700
   - Toutes les vues héritent automatiquement du changement.

2. **Ajouter `pub fn star_separator<'a, M: 'a>() -> Element<'a, M>`** :
   ```rust
   // Ligne ─────★───── avec étoile FA4 centrée
   row![
     rule::horizontal(1),
     text("\u{f005}").font(ICONS).size(10).color(theme::accent()),
     rule::horizontal(1),
   ].spacing(8).align_y(Center)
   ```

3. **Ajouter `pub fn card_banner<'a>(handle: image::Handle) -> Element<'a, Message>`** :
   ```rust
   // Bannière en bas de card — doit être le DERNIER enfant de la card column
   // Le container a background = card_bg() pour que le fond crème de l'image
   // se fonde dans la card ; les coins sont arrondis par overflow du container parent.
   Image::new(handle).width(Length::Fill)
   ```
   Usage : `column![...content..., ussr_assets::card_banner(banner_handle(N))]`

### `src/ui/theme.rs`

Ajuster les valeurs USSR pour coller aux mockups validés :
- `USSR_ACCENT` : vérifier que `(0.800, 0.133, 0.000)` correspond au `#cc3333` des mockups
  → `#cc3333` = (0.800, 0.200, 0.200) — corriger si décalé
- `USSR_CARD_BG` : doit correspondre à `#f2ead8` des mockups
- `USSR_DETAIL_BG` : `#eee6d9`
- Ajouter `pub fn list_panel_bg() -> Color` (fond du panel liste 320px, plus sombre que detail_bg)

---

## Phase 3 — Sidebar (mod.rs)

Vérifier que les étoiles `\u{f005}` apparaissent bien devant chaque `section_label` en mode USSR. Si pas encore fait, les ajouter via `theme::flavor()` :

```rust
// src/ui/mod.rs ~line 217
section_label(
    theme::flavor("", "\u{f005} ").to_string() + s.sidebar_section_keys()
)
```

---

## Phase 4 — Page Settings (src/ui/settings.rs) — MAJEUR

**Trois changements :**

### 4a. Aperçus thème (mini-preview cards)
Remplacer les 2 radios de thème par 2 containers cliquables avec un aperçu CSS-like :
- Container Catppuccin : fond `#303446`, mini-sidebar violette, barre mauve
- Container USSR : fond `#eee6d9`, mini-sidebar quasi-noire, barre rouge
- Sélectionné = border `theme::accent()`, non sélectionné = `theme::border()`
- Clic envoie `Message::ThemeChanged(variant)`

### 4b. Scale factor — règle graduée
Remplacer les 7 radios par :
```rust
column![
  slider(0.0..=6.0, scale_index as f64, |v| Message::ScaleFactorChanged(INDEX_TO_SCALE[v as usize])),
  // Tick marks textuels
  row![
    text("50%").size(10), Space::Fill,
    text("75%").size(10), Space::Fill,
    text("100%").size(10), Space::Fill,
    text("125%").size(10).color(accent()), // actif
    Space::Fill, text("150%").size(10),
    Space::Fill, text("175%").size(10),
    Space::Fill, text("200%").size(10),
  ]
]
```
`scale_index` = position (0–6) dans `[0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0]`.

### 4c. Langue — grille de boutons
Remplacer les 2 radios langue par 2 `button` côte à côte :
```rust
row![
  button("English").style(if English { primary() } else { ghost_neutral() }).on_press(ChangeLanguage(English)),
  button("Français").style(if French  { primary() } else { ghost_neutral() }).on_press(ChangeLanguage(French)),
].spacing(8)
```

### 4d. Bannière en bas de card
Ajouter `card_banner(banner_handle(29))` comme dernier enfant.

**Fichier :** `src/ui/settings.rs` (131 lignes → ~200 lignes)

---

## Phase 5 — Pages opérations (card-based)

Chaque vue reçoit un `star_separator()` entre les sections majeures et un `card_banner()` en dernier enfant. Les changements fonctionnels sont limités.

### `src/ui/sign.rs` (~257 lignes)
- Remplacer `rule::horizontal` inter-sections par `star_separator()`
- Ajouter `card_banner(banner_handle(20))` en dernier enfant
- **Pas de changement fonctionnel**

### `src/ui/verify.rs` (~391 lignes)
- Idem + `card_banner(banner_handle(17))`

### `src/ui/decrypt.rs` (nouveau ou existant)
- Si absent : créer `src/ui/decrypt.rs` sur le modèle de `sign.rs` (card_wide, drop zone, bouton Decrypt)
- Note : vérifier si decrypt est dans `encrypt.rs` ou séparé
- `card_banner(banner_handle(19))`

### `src/ui/encrypt.rs` — MAJEUR
**Restructurer de 2 colonnes → 1 colonne avec chips :**

Actuel : `row![recipients_col, rule::vertical(1), files_col]`

Nouveau :
```
column![
  info_banner,
  star_separator,
  section "Destinataires",
  chips_row (Mes clefs),
  chips_row (Clefs publiques),
  star_separator,
  section "Fichiers",
  drop_zone,
  choose_btn,
  star_separator,
  bottom_row (format_toggle + encrypt_btn),
  card_banner(16),
]
```

**Chips de destinataires :**
```rust
let chip = button(row![key_name, badge_svg])
  .style(if selected { primary() } else { ghost_neutral() })
  .on_press(Message::EncryptToggleRecipient(fp));
```

### `src/ui/import.rs`
- Ajouter `card_banner(banner_handle(24))` en dernier enfant
- Ajouter `star_separator()` entre les 4 sources
- **Pas de changement fonctionnel**

### `src/ui/create_key.rs`
- Labels inline (row label + input) pour Nom et Email
- `card_banner(banner_handle(27))`
- `star_separator()` entre sections

### `src/ui/health.rs`
- `card_banner(banner_handle(12))`
- `star_separator()` entre catégories (Installation / Agent GPG / Sécurité)

---

## Phase 6 — Vues clefs (key_list.rs, key_detail.rs)

### `src/ui/key_list.rs`

**Bannière en tête du panel liste :**
```rust
// En haut de list_col, avant le scrollable
Image::new(banner_handle(18)).width(Length::Fill)  // Mes Clefs
Image::new(banner_handle(23)).width(Length::Fill)  // Public Keys
```

**Badges statut (remplacer icônes texte par SVG circulaires) :**
```rust
// Keyserver publié → badge_keyserver SVG (étoile rouge sur cercle rouge)
Svg::new(badge_keyserver()).width(18).height(18)
// Sur YubiKey → badge_yubikey SVG (☭ sur cercle noir)
Svg::new(badge_yubikey()).width(18).height(18)
// Trust (public keys) → badge_trust_full/marginal/undef
Svg::new(badge_trust_full()).width(18).height(18)
```

### `src/ui/key_detail.rs`

**Bannière en pied du panel détail :**
```rust
// En bas de detail_col, après les actions et le certificat de révocation
Image::new(banner_handle(25)).width(Length::Fill)  // Mes Clefs
Image::new(banner_handle(17)).width(Length::Fill)  // Public Keys
```

**Séparateurs étoile** entre les sections du détail (remplacer `rule::horizontal`) :
```rust
star_separator()
```

---

## Phase 7 — Chat (src/ui/chat.rs)

- Ajouter bannière `26` en tête du panel rooms (sous le header "Transmissions")
- Vérifier que le header rooms affiche bien "+ Nouveau" et "Rejoindre"
- **Pas de changement fonctionnel sur la messagerie**

---

## Phase 8 — Documentation (subagent dédié)

Après validation visuelle de toutes les pages, lancer `voltagent-dev-exp:documentation-engineer` pour :

- `book/src/11-settings.md` — refléter le nouveau settings UI (règle, aperçus thème, boutons langue)
- `book/src/3-key-management.md` — nouveaux badges statut, bannières, séparateurs
- `book/src/5-file-operations.md` — nouveau layout Encrypt (chips recipients, colonne unique)
- `CLAUDE.md` section `### UI theme` — documenter les nouvelles fonctions `star_separator()`, `card_banner()`, `ussr_assets`, badge functions
- `CLAUDE.md` section `## Roadmap` — marquer items concernés comme ✅

---

## Fichiers critiques à modifier

| Fichier | Type de changement |
|---------|-------------------|
| `src/ui/ussr_assets.rs` | NOUVEAU — bundling assets |
| `src/ui/common.rs` | card widths, star_separator, card_banner |
| `src/ui/theme.rs` | ajustements palette USSR |
| `src/ui/settings.rs` | MAJEUR — aperçus thème, règle scale, boutons langue |
| `src/ui/encrypt.rs` | MAJEUR — chips recipients, layout colonne unique |
| `src/ui/key_list.rs` | bannière liste, badges SVG |
| `src/ui/key_detail.rs` | bannière pied détail, séparateurs |
| `src/ui/sign.rs` | séparateurs + bannière |
| `src/ui/verify.rs` | séparateurs + bannière |
| `src/ui/health.rs` | séparateurs + bannière |
| `src/ui/import.rs` | séparateurs + bannière |
| `src/ui/create_key.rs` | labels inline + bannière |
| `src/ui/chat.rs` | bannière rooms panel |
| `assets/banners/*.png` | 13 fichiers copiés |
| `assets/*.svg` | 6 fichiers copiés |
| `Cargo.toml` | vérifier `image` feature d'iced activée |

---

## Phase 9 — Revue qualité & sécurité (sous-agents dédiés)

**Règle générale** : pour chaque plan (même purement cosmétique), les sous-agents
`voltagent-qa-sec:code-reviewer` et `voltagent-qa-sec:security-auditor` doivent être
interrogés. Leur rôle ici est de **confirmer l'absence de régression** ou, si rien n'est
impacté, de le noter explicitement.

### Code reviewer (`voltagent-qa-sec:code-reviewer`)
- Vérifier l'usage de `include_bytes!()` pour les assets binaires (taille binaire, pas de fuite de données)
- Valider que `widget::image` et `widget::svg` sont utilisés sans risque (pas d'accès réseau, pas de parsing non contrôlé)
- S'assurer que les nouvelles fonctions dans `common.rs` et `ussr_assets.rs` respectent les conventions du projet
- Confirmer : pas de régression sur les tests existants

### Security auditor (`voltagent-qa-sec:security-auditor`)
- Confirmer que les assets PNG/SVG bundlés ne contiennent pas de contenu actif
- Vérifier que `svg::Handle::from_memory()` dans iced ne parse pas de SVG avec `<script>` ou liens externes
- Confirmer : aucune surface d'attaque nouvelle introduite par ce plan de theming

**Résultat attendu** : même si rien n'est impacté, la revue est tracée dans le plan.

---

## Vérification

```bash
# Dans nix develop
cargo build                    # doit compiler sans erreur
cargo clippy -- -D warnings    # zéro warning
cargo test --lib               # tests unitaires
cargo test -- --ignored        # tests GPG intégration
# Lancer l'app et vérifier chaque vue visuellement contre les mockups
cargo run
```

**Checklist visuelle :**
- [ ] Sidebar : étoiles rouges devant chaque section, icônes Nerd Font correctes
- [ ] Toutes les cards à ~700px
- [ ] Bannière propagandiste en bas de chaque card
- [ ] Séparateurs étoile entre sections
- [ ] Settings : aperçu thème, règle scale, boutons langue
- [ ] Encrypt : layout vertical, recipients en liste compacte
- [ ] Key list : bannières en tête des panels liste et détail
- [ ] Key detail : bannière en pied
- [ ] Chat : bannière en tête du panel rooms
- [ ] Revue code-reviewer : ✅ / ❌
- [ ] Revue security-auditor : ✅ / ❌
