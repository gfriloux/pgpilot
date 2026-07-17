# Axe 0 — Specification du systeme de largeur de cartes

Date : 2026-05-05
Prerequis : `axe0-audit-ui.md`

---

## Contexte et contraintes

### Espace disponible pour les vues pleine-page

Les vues pleine-page (Import, CreateKey, Health, Encrypt, Sign, Verify, Settings, et les futures
vues Chat) remplacent **toute** la zone droite de la fenetre — elles ne cohabitent pas avec le
panneau liste. L'espace horizontal disponible est donc :

```
1000 px (min window)
- 180 px (sidebar, mesure dans ui/mod.rs ligne 232)
- ~1 px (separateur implicite entre sidebar et main)
= 819 px disponibles (arrondi a 820 px)
```

A la taille cible 1280 px, l'espace disponible est environ 1100 px.

### Contraintes fermes

- `center_y(Length::Fill)` est interdit — provoque un centrage vertical indesirable sur grandes fenetres.
- Le padding externe `[24, 0]` (vertical haut/bas, zero lateral) est fixe par le pattern de reference.
- Le fond externe doit toujours etre `sidebar_bg()`.
- Le `scrollable` doit toujours porter `common::scroll_style`.

---

## Decisions de conception

### Niveau Narrow — 480 px

**Valeur retenue** : 480 px (actuellement 400 px pour Settings).

**Justification** :
- Settings contient uniquement des radios et labels courts — aucune raison de depasser 480 px.
- 480 px laisse ~170 px de marge de chaque cote sur une fenetre de 820 px disponibles, soit un
  ratio de marge confortable (environ 21 % de chaque cote).
- 400 px actuel est trop etroit : les titres de section tronquent dans certaines locales et les
  labels des radios de scale factor ("175%", "200%") risquent de se couper si la police change.
- 480 px reste inferieur au niveau Medium (560 px), garantissant une distinction perceptible.

**Pages assignees** : Settings uniquement.

---

### Niveau Medium — 560 px

**Valeur retenue** : 560 px (identique a Health actuel, Import et CreateKey passent de 520 a 560).

**Justification** :
- Import et CreateKey sont actuellement a 520 px. Passer a 560 px les aligne sur Health et
  evite une valeur intermediaire inutile (520 px).
- 560 px sur 820 px disponibles = 130 px de marge de chaque cote (16 %). Confortable sans
  sembler "flottant".
- Le contenu de ces trois vues (formulaires, listes d'options, textes) tient dans 560 px.
- 560 px est une valeur "naturelle" : 7 * 80 px, facile a memoriser.

**Pages assignees** : Import, CreateKey, Health.

---

### Niveau Wide — `Length::Fill` avec max_width 760 px

**Valeur retenue** : `Length::Fill` + `.max_width(760)`.

**Justification** :
- Encrypt est actuellement a 720 px fixe. Sign et Verify sont a 640 px.
- Sign et Verify beneficieraient de plus d'espace sur grandes fenetres (le bloc de resultats
  est dense). Passer a Wide leur offre de la respiration.
- 720 px fixe pour Encrypt est trop rigide : sur 1280 px disponibles, la carte ne profite pas
  de l'espace. `Fill` avec `max_width(760)` permet a la carte de grandir jusqu'a une limite
  raisonnable.
- `max_width(760)` : reste en-dessous du seuil de lisibilite inconfortable (environ 800 px pour
  du texte courant), evite l'effet "spreadsheet".
- Sur la fenetre minimum (820 px disponibles), la carte occupe 760 px — 30 px de marge de chaque
  cote, ce qui est acceptable car ces vues sont des outils de travail, pas des pages
  "contemplatives".

**Implementation** : `container(card).max_width(760)` remplace `.width(N)` fixe. iced 0.14
supporte `.max_width()` sur `container`.

**Pages assignees** : Encrypt, Sign, Verify, et les futures vues Chat.

---

### Padding interne — identique pour tous les niveaux

**Valeur retenue** : `32` (uniforme, identique a l'existant).

**Justification** :
- Tous les 7 fichiers utilisent deja `.padding(32)`. C'est une constante de fait.
- Un padding variable par niveau de largeur n'apporterait pas de benefice visuel perceptible et
  compliquerait les helpers.
- 32 px offre un blanc interieur genereux qui fait respirer le contenu.

---

### Border radius — valeur unique

**Valeur retenue** : `12.0` (uniforme, identique a l'existant).

**Justification** :
- Tous les 7 fichiers utilisent `radius: 12.0`. La coherence est totale.
- 12 px correspond au token "radius-lg" dans la plupart des design systems modernes, adapte
  a des blocs de cette taille.
- Les widgets internes utilisent 6.0 (inputs, boutons) et 4.0 (badges inline) — la hierarchie
  est coherente : card (12) > composant interactif (6) > badge (4).

---

### Pages Chat a venir (phases 6-12 du plan v0.6.0)

**Niveau recommande** : Wide (760 px max).

**Justification** :
- Une interface de messagerie chiffree necessite de l'espace horizontal pour afficher
  simultanement la liste des conversations et le fil de messages.
- Avec `Length::Fill` + `max_width(760)`, la carte s'adapte a la taille de la fenetre,
  ce qui est le comportement attendu pour une interface conversationnelle.
- Si le chat adopte un layout deux colonnes (liste contact | fil de messages), 760 px minimum
  permet deux colonnes de ~350 px chacune — suffisant pour afficher un message PGP chiffre.

---

## Signatures des helpers

### Emplacement recommande

Tous les helpers sont a ajouter dans `src/ui/common.rs`, a la suite des helpers existants
(`scroll_style`, `radio_style`, `pick_btn`, `action_btn`).

### `page_layout`

```rust
/// Enveloppe externe standard pour toutes les vues pleine-page.
///
/// Produit : container (fond sidebar_bg, fill) > scrollable (fill, scroll_style)
///           > container (center_x, padding [24, 0], fill)
///
/// Le `card` passe en argument est le container interieur (la "carte blanche").
/// Sa largeur est definie par le helper de niveau utilise (card_narrow, card_medium, card_wide).
pub fn page_layout<'a>(card: Element<'a, Message>) -> Element<'a, Message> {
  container(
    scrollable(
      container(card)
        .center_x(Length::Fill)
        .padding([24, 0])
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .style(scroll_style),
  )
  .height(Length::Fill)
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::sidebar_bg())),
    ..Default::default()
  })
  .into()
}
```

Proprietes :
- Fond externe : `sidebar_bg()`
- Scrollable : `height(Fill)`, `width(Fill)`, `scroll_style`
- Container intermediaire : `center_x(Fill)`, `padding([24, 0])`, `width(Fill)`
- Aucun `center_y` — interdit par CLAUDE.md

---

### `card_narrow`

```rust
/// Carte etroite — Settings uniquement.
///
/// Largeur : 480 px fixe.
/// Padding interne : 32.
/// Border radius : 12.0.
/// Background : card_bg().
pub fn card_narrow<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
  container(content)
    .padding(32)
    .width(480)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::card_bg())),
      border: Border {
        color: theme::border(),
        width: 1.0,
        radius: 12.0.into(),
      },
      text_color: Some(theme::text_strong()),
      ..Default::default()
    })
    .into()
}
```

---

### `card_medium`

```rust
/// Carte moyenne — Import, CreateKey, Health.
///
/// Largeur : 560 px fixe.
/// Padding interne : 32.
/// Border radius : 12.0.
/// Background : card_bg().
pub fn card_medium<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
  container(content)
    .padding(32)
    .width(560)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::card_bg())),
      border: Border {
        color: theme::border(),
        width: 1.0,
        radius: 12.0.into(),
      },
      text_color: Some(theme::text_strong()),
      ..Default::default()
    })
    .into()
}
```

---

### `card_wide`

```rust
/// Carte large — Encrypt, Sign, Verify, Chat (futur).
///
/// Largeur : Fill avec max_width 760 px.
/// Padding interne : 32.
/// Border radius : 12.0.
/// Background : card_bg().
pub fn card_wide<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
  container(content)
    .padding(32)
    .max_width(760)
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::card_bg())),
      border: Border {
        color: theme::border(),
        width: 1.0,
        radius: 12.0.into(),
      },
      text_color: Some(theme::text_strong()),
      ..Default::default()
    })
    .into()
}
```

---

## Matrice de mapping vue / helper

| Vue | Helper card | Largeur effective | Migration depuis |
|-----|-------------|-------------------|-----------------|
| Settings | `card_narrow` | 480 px fixe | 400 px fixe (+80) |
| Import | `card_medium` | 560 px fixe | 520 px fixe (+40) |
| CreateKey | `card_medium` | 560 px fixe | 520 px fixe (+40) |
| Health | `card_medium` | 560 px fixe | 560 px fixe (=) |
| Sign | `card_wide` | jusqu'a 760 px | 640 px fixe |
| Verify | `card_wide` | jusqu'a 760 px | 640 px fixe |
| Encrypt | `card_wide` | jusqu'a 760 px | 720 px fixe |
| Chat (futur) | `card_wide` | jusqu'a 760 px | N/A |

---

## Pattern d'usage dans une vue

```rust
pub fn view<'a>(...) -> Element<'a, Message> {
  // 1. Construire le contenu de la carte
  let content = column![
    text(s.page_title()).size(22),
    // ... widgets
  ]
  .spacing(20);

  // 2. Encapsuler dans le helper de niveau
  let card = common::card_medium(content);   // ou card_narrow / card_wide

  // 3. Envelopper avec page_layout
  common::page_layout(card)
}
```

Ce pattern remplace les ~20 lignes de boilerplate actuellement dupliquees dans chaque vue.

---

## Refactorisation de health.rs — correction D1

La deviation D1 (etat loading sans container externe) est corrigee mecaniquement par l'adoption
de `page_layout` : puisque `page_layout` fournit toujours le container externe avec `sidebar_bg()`,
le chemin loading devient identique au chemin loaded.

```rust
// Avant (deviation D1) :
if loading {
  return scrollable(container(loading_content).width(560) ...)
    .height(Length::Fill)
    .into();
}

// Apres (conforme) :
if loading {
  let card = common::card_medium(loading_content);
  return common::page_layout(card);
}
```

---

## Tableau recapitulatif des valeurs

| Parametre | Narrow | Medium | Wide |
|-----------|--------|--------|------|
| Largeur | 480 px fixe | 560 px fixe | Fill + max 760 px |
| Padding interne | 32 | 32 | 32 |
| Border radius | 12.0 | 12.0 | 12.0 |
| Background | `card_bg()` | `card_bg()` | `card_bg()` |
| Border width | 1.0 | 1.0 | 1.0 |
| Border color | `border()` | `border()` | `border()` |

| Parametre `page_layout` | Valeur |
|------------------------|--------|
| Fond externe | `sidebar_bg()` |
| Padding externe | `[24, 0]` |
| `center_x` | `Length::Fill` |
| `center_y` | interdit |
| Scrollbar style | `common::scroll_style` |
