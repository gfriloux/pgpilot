# Axe 0 — Audit des layouts existants

Date : 2026-05-05
Scope : 7 vues pleine-page (`src/ui/`)

---

## Tableau de synthese

| Vue | Fichier | Largeur card | Padding interne | Border radius | Centrage X | Pattern scrollable | Deviation |
|-----|---------|-------------|----------------|--------------|-----------|-------------------|-----------|
| Import | `import.rs` | `Length::Fixed(520)` | `32` | `12.0` | Oui | Conforme | Aucune |
| CreateKey | `create_key.rs` | `Length::Fixed(520)` | `32` | `12.0` | Oui | Conforme | Aucune |
| Health | `health.rs` | `Length::Fixed(560)` | `32` | `12.0` | Oui | Conforme | Deviation D1 |
| Encrypt | `encrypt.rs` | `Length::Fixed(720)` | `32` | `12.0` | Oui | Conforme | Aucune |
| Sign | `sign.rs` | `Length::Fixed(640)` | `32` | `12.0` | Oui | Conforme | Aucune |
| Verify | `verify.rs` | `Length::Fixed(640)` | `32` | `12.0` | Oui | Conforme | Aucune |
| Settings | `settings.rs` | `Length::Fixed(400)` | `32` | `12.0` | Oui | Conforme | Aucune |

### Legende colonnes

- **Largeur card** : valeur passee a `.width()` sur le `container` interieur (la carte blanche).
- **Padding interne** : valeur du `.padding()` sur ce meme container.
- **Border radius** : valeur du champ `radius` dans le style de bordure de la carte.
- **Centrage X** : presence de `.center_x(Length::Fill)` sur le container intermediaire.
- **Pattern scrollable** : le triple emboitement `container > scrollable > container(card)` conforme a la reference CLAUDE.md, avec fond `sidebar_bg()` sur le container externe.

---

## Detail par vue

### import.rs (lignes 152-300)

```
card:
  .padding(32)
  .width(520)
  border radius: 12.0
  background: card_bg()
  border width: 1.0, color: border()

outer wrapper:
  container > scrollable > container(card)
    .center_x(Length::Fill)
    .padding([24, 0])
    .width(Length::Fill)
  scrollable:
    .height(Length::Fill).width(Length::Fill)
    .style(common::scroll_style)
  outer container:
    .height(Length::Fill).width(Length::Fill)
    background: sidebar_bg()
```

Conformite : pattern de reference respecte integralement.

---

### create_key.rs (lignes 133-300)

```
card:
  .padding(32)
  .width(520)
  border radius: 12.0
  background: card_bg()
  border width: 1.0, color: border()

outer wrapper: identique a import.rs
```

Conformite : pattern de reference respecte integralement.

---

### health.rs (lignes 178-210)

```
card (etat normal):
  .padding(32)
  .width(560)
  border radius: 12.0
  background: card_bg()
  border width: 1.0, color: border()

outer wrapper: identique au pattern de reference
```

**Etat loading (lignes 43-72) — Deviation D1** :
Le chemin de retour precoce `if loading { return scrollable(...).into(); }` construit un layout
incomplet : il manque le `container` externe avec fond `sidebar_bg()`. Le fond devient celui du
`scrollable` lui-meme (transparent/herite), ce qui expose le fond `detail_bg()` du parent
`main_el` dans `ui/mod.rs` (fond `detail_bg()`, pas `sidebar_bg()`).

Conformite etat normal : conforme. Etat loading : non conforme (fond incorrect).

---

### encrypt.rs (lignes 575-628)

```
card:
  .padding(32)
  .width(720)
  border radius: 12.0
  background: card_bg()
  border width: 1.0, color: border()

outer wrapper: conforme au pattern de reference
```

Note : la carte est la plus large du codebase (720 px). Le contenu justifie cette largeur — deux
colonnes cote-a-cote (`recipients_col` en `FillPortion(45)` + `files_col` en `FillPortion(55)`).

Conformite : conforme.

---

### sign.rs (lignes 198-286)

```
card:
  .padding(32)
  .width(640)
  border radius: 12.0
  background: card_bg()
  border width: 1.0, color: border()

outer wrapper: conforme au pattern de reference
```

Conformite : conforme.

---

### verify.rs (lignes 352-420)

```
card:
  .padding(32)
  .width(640)
  border radius: 12.0
  background: card_bg()
  border width: 1.0, color: border()

outer wrapper: conforme au pattern de reference
```

Conformite : conforme. Identique a sign.rs par construction (forme symetrique).

---

### settings.rs (lignes 103-160)

```
card:
  .padding(32)
  .width(400)
  border radius: 12.0
  background: card_bg()
  border width: 1.0, color: border()

outer wrapper: conforme au pattern de reference
  Note : utilise `iced::widget::scrollable` qualifie au lieu de l'import `scrollable`
  sans aliasing — fonctionnellement identique, style cosmétique uniquement.
```

Conformite : conforme.

---

## Deviations identifiees

### D1 — health.rs:43-72 — etat loading : wrapper externe manquant

**Fichier** : `src/ui/health.rs`, lignes 43-72

**Probleme** : Le chemin `if loading { return scrollable(...).into(); }` retourne directement un
`scrollable` sans l'envelopper dans le `container` externe qui porte `sidebar_bg()`. La vue
loading a donc un fond diffrent de la vue loaded.

**Impact visuel** : Sur le theme Catppuccin, `detail_bg()` ≈ `#303446` et `sidebar_bg()` ≈
`#292c3c` — difference perceptible. Sur USSR, les deux couleurs different aussi.

**Correction attendue** :

```rust
// Avant (incomplet) :
return scrollable( container(...).width(560) ... )
  .height(Length::Fill).width(Length::Fill)
  .style(common::scroll_style)
  .into();

// Apres (conforme) :
return container(
  scrollable(
    container(container(...).width(560) ...)
      .center_x(Length::Fill).padding([24, 0]).width(Length::Fill),
  )
  .height(Length::Fill).width(Length::Fill)
  .style(common::scroll_style),
)
.height(Length::Fill).width(Length::Fill)
.style(|_| container::Style {
  background: Some(Background::Color(theme::sidebar_bg())),
  ..Default::default()
})
.into();
```

---

## Observations complementaires (non-deviations)

### Boutons inline vs helpers de common.rs

Plusieurs vues (`import.rs`, `create_key.rs`, `encrypt.rs`) definissent leurs boutons d'action
inline avec leur propre closure de style, au lieu d'utiliser `common::action_btn`. Les vues plus
recentes (`sign.rs`, `verify.rs`) utilisent `common::action_btn`. Cela n'est pas une deviation
de layout mais une inconsistance de factorisation a adresser separement.

### Separateurs : `snap: false` vs `snap: true`

- `import.rs`, `create_key.rs`, `settings.rs`, `encrypt.rs` : `snap: false`
- `sign.rs`, `verify.rs` : `snap: true`

Difference negligeable visuellement a cette echelle, mais un choix unique serait preferable.

### Largeurs observees

| Vue | Largeur px |
|-----|-----------|
| Settings | 400 |
| Import | 520 |
| CreateKey | 520 |
| Health | 560 |
| Sign | 640 |
| Verify | 640 |
| Encrypt | 720 |

Cinq valeurs distinctes en l'etat. Le futur systeme de cards doit les rationaliser en 3 niveaux.
