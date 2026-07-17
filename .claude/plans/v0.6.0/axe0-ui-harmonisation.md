# Axe 0 — Harmonisation UI (card layout)

## Objectif

Toutes les pages de PGPilot sauf `View::MyKeys` et `View::PublicKeys` affichent leur contenu
dans un bloc aux bords arrondis ("card"). Ce bloc n'a pas de largeur cohérente entre les pages :
certaines cartes sont étroites, d'autres larges, sans logique apparente. L'objectif est de
définir un système de largeur de carte cohérent et de l'appliquer uniformément.

**Pages concernées** : Import, CreateKey, Health, Encrypt, Sign, Verify, Settings, Chat (à venir).

---

## T0.1 — Audit visuel des cartes existantes

**Complexité** : S
**Agent** : `voltagent-core-dev:ui-designer`
**Dépendances** : aucune

### Ce qui est à faire

Lire les fichiers UI suivants et mesurer/noter pour chaque vue :
- `src/ui/import.rs`
- `src/ui/create_key.rs`
- `src/ui/health.rs`
- `src/ui/encrypt.rs`
- `src/ui/sign.rs`
- `src/ui/verify.rs`
- `src/ui/settings.rs`

Pour chaque fichier :
1. Quelle est la largeur du bloc card (valeur `Length::Fixed(x)`, `Length::Fill`, `max_width`, ou padding) ?
2. Quel est le padding interne ?
3. Quel est le border radius ?
4. Le contenu est-il centré horizontalement ?
5. Y a-t-il un pattern `scrollable(container(card).center_x(...))` — est-il suivi partout ?

**Référence attendue** dans CLAUDE.md (section "View layout pattern") :
```rust
container(
  scrollable(container(card).center_x(Length::Fill).padding([24, 0]).width(Length::Fill))
    .height(Length::Fill).width(Length::Fill),
)
.height(Length::Fill).width(Length::Fill)
.style(|_| container::Style { background: Some(Background::Color(theme::sidebar_bg())), ... })
```

Ce pattern est-il respecté partout ? Quelles dévations existent ?

**Output attendu** : tableau comparatif + liste des déviations au pattern de référence.

**Commit** : aucun — livrable = `axe0-audit-ui.md` dans `.claude/plans/v0.6.0/`

---

## T0.2 — Définition du système de largeur

**Complexité** : M
**Agent** : `voltagent-core-dev:ui-designer`
**Dépendances** : T0.1

### Ce qui est à définir

En s'appuyant sur le tableau de T0.1 et la contrainte de fenêtre minimum (1000px),
définir un système à **3 niveaux de largeur de carte** :

| Niveau | Largeur | Usage |
|--------|---------|-------|
| **Narrow** | 480px | Pages à formulaire simple (CreateKey, Settings, Sign, Verify) |
| **Medium** | 680px | Pages à formulaire étendu ou liste courte (Import, Health) |
| **Wide** | `Length::Fill` avec `max_width: 900px` | Pages complexes, multi-colonnes (Encrypt, Health détaillé) |

Questions à trancher par l'UI designer :
- Ces 3 niveaux sont-ils les bons ? Faut-il en avoir 2 ou 4 ?
- Quelle largeur pour les pages chat (à venir) ?
- Le padding interne de la card doit-il varier avec le niveau ?
- Border radius : valeur unique pour toutes les cartes ?

**Helpers à créer dans `src/ui/common.rs`** :
```rust
pub fn card_narrow<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message>;
pub fn card_medium<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message>;
pub fn card_wide<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message>;
```

Chaque helper encapsule : `container(content).padding(24).style(card_style).width(LEVEL_WIDTH)`.

**Commit** : aucun — livrable = `axe0-spec-cards.md` dans `.claude/plans/v0.6.0/`

---

## T0.3 — Implémentation helpers + migration des vues

**Complexité** : M
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T0.2 (spec validée)

### Ce qui est à faire

**1. Ajouter les helpers dans `src/ui/common.rs`**

Selon la spec de T0.2. Exemple :
```rust
pub fn card_narrow<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(24)
        .width(Length::Fixed(480.0))
        .style(|_| container::Style {
            background: Some(Background::Color(theme::card_bg())),
            border: Border { radius: 8.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}
// idem card_medium, card_wide
```

Et le wrapper page complet (scrollable + fond sidebar_bg) :
```rust
pub fn page_layout<'a>(card: Element<'a, Message>) -> Element<'a, Message> {
    container(
        scrollable(
            container(card)
                .center_x(Length::Fill)
                .padding([24, 0])
                .width(Length::Fill)
        )
        .height(Length::Fill)
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(theme::sidebar_bg())),
        ..Default::default()
    })
    .into()
}
```

**2. Migrer chaque vue concernée**

Pour chaque fichier identifié en T0.1 :
- Remplacer le layout manuel par `page_layout(card_narrow(...))` ou `card_medium` ou `card_wide`
- S'assurer que `center_y(Length::Fill)` n'est jamais utilisé (voir note CLAUDE.md)

Fichiers à migrer :
- `src/ui/import.rs`
- `src/ui/create_key.rs`
- `src/ui/health.rs`
- `src/ui/encrypt.rs`
- `src/ui/sign.rs`
- `src/ui/verify.rs`
- `src/ui/settings.rs`

**3. Documenter dans CLAUDE.md**

Remplacer la section "View layout pattern" par :
```markdown
### View layout pattern

Toutes les vues pleine page (sauf MyKeys et PublicKeys) utilisent les helpers de `ui/common.rs` :

```rust
// Choisir le niveau selon la complexité de la page
page_layout(card_narrow(content))   // formulaire simple
page_layout(card_medium(content))   // formulaire étendu
page_layout(card_wide(content))     // vue complexe, multi-colonnes
```

Ne jamais utiliser `center_y(Length::Fill)` — ça centre verticalement sur les grandes fenêtres.
Ne jamais dupliquer le pattern scrollable/container inline — utiliser `page_layout`.
```

**Commit** : `refactor(ui): harmonize card layout with page_layout/card_narrow/medium/wide helpers`

---

## T0.4 — Revue visuelle

**Complexité** : S
**Agent** : `voltagent-core-dev:ui-designer`
**Dépendances** : T0.3

### Ce qui est à faire

1. Lire toutes les vues migrées et vérifier la cohérence visuelle sur le papier :
   - Chaque page utilise le bon niveau de carte pour son contenu ?
   - Padding et border radius cohérents ?
   - Pas de layout cassé évident (contenu trop large pour la carte) ?
2. Vérifier que les nouvelles vues chat (axe 5) utilisent `page_layout` + le bon niveau
3. Signaler tout ajustement de niveau (ex: "Health devrait être `card_wide` plutôt que `card_medium`")

**Commit** : ajustements si nécessaire — `fix(ui): adjust card level for health and encrypt views`

---

## Livrables

```
.claude/plans/v0.6.0/axe0-audit-ui.md     (T0.1)
.claude/plans/v0.6.0/axe0-spec-cards.md   (T0.2)
src/ui/common.rs                           (+ page_layout, card_narrow, card_medium, card_wide)
src/ui/import.rs, create_key.rs, ...       (migrés)
CLAUDE.md                                  (section "View layout pattern" mise à jour)
```

## Critères d'acceptation

- [ ] `cargo build` ✓
- [ ] `cargo clippy -- -D warnings` ✓
- [ ] Toutes les vues concernées utilisent `page_layout(card_*)` — zéro layout inline dupliqué
- [ ] Largeur cohérente par niveau : Narrow = 480px, Medium = 680px, Wide = fill+max 900px
- [ ] `center_y(Length::Fill)` absent de toutes les vues migrées
- [ ] Section "View layout pattern" dans CLAUDE.md à jour
