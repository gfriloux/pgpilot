# axe5-ui-spec.md — Spec visuelle UI Chat

> Livrable de T5.1. Ce document est la spec complète et non ambigue que le rust-engineer
> consomme pour implémenter les vues iced du chat. Toute decision visuelle est tranchee ici.
> Le rust-engineer ne doit pas inventer de patterns — il suit ce document.

---

## 1. Layout general

### 1.1 Structure des trois colonnes

```
┌──────────────┬──────────────────┬───────────────────────────────────────┐
│ SIDEBAR      │ ROOM LIST        │ CHAT PANEL                            │
│ 180px fixe   │ 280px fixe       │ fill (Length::Fill)                   │
│              │                  │                                       │
│ (existant)   │ room_list_panel  │ chat_panel / empty_chat_state         │
└──────────────┴──────────────────┴───────────────────────────────────────┘
```

### 1.2 Structure iced de haut niveau

Le point d'entree est `ui::chat::view(app)` qui remplace le `chat_placeholder` existant dans
`ui/mod.rs` pour les variantes `View::ChatList` et `View::ChatRoom(_)`.

```rust
// ui/chat.rs — fonction publique principale
pub fn view(app: &App) -> Element<'_, Message> {
  let room_list = room_list_panel(app);
  let sep = rule::vertical(1).style(|_: &iced::Theme| rule::Style {
    color: theme::border(),
    radius: 0.0.into(),
    fill_mode: rule::FillMode::Full,
    snap: false,
  });
  let chat = match &app.active_room {
    Some(room_id) => {
      let room = app.rooms.iter().find(|r| &r.id == room_id);
      let msgs = app.chat_messages.get(room_id.as_str());
      match room {
        Some(r) => chat_panel(r, msgs.map(|d| d.as_slices().0).unwrap_or(&[]), app),
        None => empty_chat_state(app.strings),
      }
    }
    None => empty_chat_state(app.strings),
  };

  row![room_list, sep, chat]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
```

Les vues `View::ChatNewRoom` et `View::ChatJoinRoom` utilisent le pattern `page_layout(card_medium(...))`
exactement comme `create_key.rs` et `import.rs` — elles NE passent PAS par `chat.rs`.

### 1.3 Correspondance avec key_list.rs

| Element              | key_list.rs              | chat.rs                  |
|----------------------|--------------------------|--------------------------|
| Panneau gauche fixe  | `width(320)`             | `width(280)`             |
| Separateur vertical  | `rule::vertical(1)`      | identique                |
| Panneau droit fill   | `width(Length::Fill)`    | identique                |
| Fond global          | `detail_bg()`            | `detail_bg()`            |
| Fond panneau gauche  | `header_bg()` (header)   | voir §3                  |
| Scrollable interne   | `scroll_style`           | `common::scroll_style`   |

---

## 2. Sidebar — cluster CHAT

### 2.1 Position dans la sidebar

Inserer le cluster CHAT entre le cluster OPERATIONS et l'espaceur `Space::new().height(Length::Fill)`,
c'est-a-dire avant le cluster OUTILS qui est en bas.

Structure finale de la sidebar apres modification :

```
CLEFS
  - Mes clefs         View::MyKeys
  - Clefs publiques   View::PublicKeys
─────────────────────
OPERATIONS
  - Chiffrer          View::Encrypt
  - Dechiffrer        View::Decrypt
  - Signer            View::Sign
  - Verifier          View::Verify
─────────────────────
CHAT                  ← nouveau cluster
  - Salons            View::ChatList
─────────────────────
[espaceur fill]
─────────────────────
OUTILS
  - Importer          View::Import
  - Creer une clef    View::CreateKey
  - Diagnostic        View::Health
  - Parametres        View::Settings
```

### 2.2 Code d'insertion dans `sidebar()` de `ui/mod.rs`

```rust
sep(),
column![
  section_label(s.nav_section_chat()),
  nav_btn("\u{f0e5}", s.nav_chat_rooms(), View::ChatList),
]
.spacing(2),
sep(),
Space::new().height(Length::Fill),
// ... cluster OUTILS existant
```

L'icone `\u{f0e5}` est FA4 "comment" (bulle de dialogue). Voir §7 pour le tableau complet des icones.

### 2.3 Comportement de l'item "Salons"

- Actif (`background: accent()`) quand `app.view == View::ChatList || matches!(app.view, View::ChatRoom(_))`
- `Message::NavChanged(View::ChatList)` au clic
- Pas de badge de comptage sur l'item sidebar (trop etroit, risque de confusion)

---

## 3. Room list panel (280px)

### 3.1 Structure generale du panneau

```rust
// ui/chat.rs — fonction privee
fn room_list_panel(app: &App) -> Element<'_, Message> {
  let s = app.strings;

  // Header fixe en haut
  let header = room_list_header(app);      // contient le titre + boutons +/Join

  // Corps : liste scrollable ou etat vide
  let body = if app.rooms.is_empty() {
    empty_rooms_state(s)
  } else {
    let rows: Vec<Element<Message>> = app.rooms.iter()
      .map(|room| room_row(room, app))
      .collect();
    scrollable(Column::with_children(rows).spacing(2).padding([4, 8]))
      .style(common::scroll_style)
      .height(Length::Fill)
      .into()
  };

  // Pied de page : badge etat MQTT
  let footer = mqtt_status_bar(&app.mqtt_state, s);

  column![header, body, footer]
    .width(Length::Fixed(280.0))
    .height(Length::Fill)
    .into()
}
```

### 3.2 Header du panneau room list

```rust
fn room_list_header(app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let bold = Font { weight: font::Weight::Bold, ..theme::heading_font() };

  // Titre "Salons" ou flavor USSR "TRANSMISSIONS"
  let title = text(theme::flavor(s.nav_chat_rooms(), s.nav_chat_rooms_ussr()))
    .size(13)
    .font(theme::flavor_title_font());

  // Boutons "+ New" et "Join"
  let btn_new = button(
    row![text("\u{f067}").font(theme::ICONS).size(11), text(s.chat_create_room()).size(12)]
      .spacing(4)
      .align_y(Alignment::Center),
  )
  .on_press(Message::NavChanged(View::ChatNewRoom))
  .padding([4, 8])
  .style(button_styles::ghost_accent());

  let btn_join = button(text(s.chat_join_room()).size(12))
    .on_press(Message::NavChanged(View::ChatJoinRoom))
    .padding([4, 8])
    .style(button_styles::ghost_neutral());

  container(
    column![
      row![title, Space::new().width(Length::Fill), btn_new, btn_join]
        .spacing(4)
        .align_y(Alignment::Center),
    ]
    .spacing(4)
    .padding([8, 12]),
  )
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::header_bg())),
    ..Default::default()
  })
  .into()
}
```

### 3.3 Ligne de salon (`room_row`)

Structure visuelle d'une ligne :

```
┌─ room_row ───────────────────────────────┐
│ Nom du salon              ● ● ○  14:32   │
│ Dernier message tronque...               │
└──────────────────────────────────────────┘
```

Specs :
- Hauteur approximative : 52px (deux lignes de texte + padding 8px vertical)
- Nom : `text(room.name).size(13).font(theme::heading_font())`
- Pastilles de presence : voir §3.4
- Timestamp du dernier message : `text_muted()`, taille 11, aligne a droite
- Apercu du dernier message : `text_secondary()`, taille 11, tronque a 40 caracteres

```rust
fn room_row(room: &Room, app: &App) -> Element<'_, Message> {
  let selected = app.active_room.as_deref() == Some(room.id.as_str());
  let s = app.strings;

  // Ligne 1 : nom + pastilles + timestamp
  let dots = presence_dots(&room.participants, &app.presence, &room.my_fp);

  // Timestamp fictif du dernier message (None si aucun message)
  let ts_label = app.chat_messages
    .get(&room.id)
    .and_then(|msgs| msgs.back())
    .map(|m| format_time_short(m.ts))
    .unwrap_or_default();

  let line1 = row![
    text(room.name.as_str()).size(13).font(theme::heading_font()).width(Length::Fill),
    dots,
    text(ts_label.as_str()).size(11).style(|_: &iced::Theme| iced::widget::text::Style {
      color: Some(theme::text_muted()),
    }),
  ]
  .spacing(6)
  .align_y(Alignment::Center);

  // Ligne 2 : apercu du dernier message
  let preview = app.chat_messages
    .get(&room.id)
    .and_then(|msgs| msgs.back())
    .map(|m| truncate_preview(&m.text, 40))
    .unwrap_or_default();

  let line2 = text(preview.as_str()).size(11).style(|_: &iced::Theme| iced::widget::text::Style {
    color: Some(theme::text_secondary()),
  });

  let content = column![line1, line2].spacing(2).width(Length::Fill);

  let styled = container(content)
    .padding([7, 12])
    .width(Length::Fill)
    .style(move |_: &iced::Theme| {
      if selected {
        container::Style {
          background: Some(Background::Color(theme::accent_subtle())),
          border: Border {
            color: theme::accent_border(),
            width: 1.0,
            radius: 6.0.into(),
          },
          ..Default::default()
        }
      } else {
        container::Style::default()
      }
    });

  mouse_area(styled)
    .on_press(Message::ChatRoomSelected(room.id.clone()))
    .into()
}
```

### 3.4 Pastilles de presence (`presence_dots`)

Les pastilles sont des caracteres Unicode directs (pas d'icone Nerd Font) pour eviter les
problemes de rendu.

- Online : `"●"` couleur `theme::success()`
- Offline : `"○"` couleur `theme::text_muted()`
- L'utilisateur local (`room.my_fp`) est toujours exclu des pastilles affiches
- Maximum 4 pastilles affichees ; si N > 4, afficher "+N" en `text_muted()` taille 10

```rust
fn presence_dots(
  participants: &[RoomParticipant],
  tracker: &PresenceTracker,
  my_fp: &str,
) -> Element<'_, Message> {
  let others: Vec<_> = participants.iter()
    .filter(|p| p.fp != my_fp)
    .collect();

  let displayed = &others[..others.len().min(4)];
  let overflow = others.len().saturating_sub(4);

  let mut dot_elems: Vec<Element<Message>> = displayed.iter().map(|p| {
    let online = tracker.get(&p.fp)
      .map(|s| *s == PresenceStatus::Online)
      .unwrap_or(false);
    let (symbol, color) = if online {
      ("●", theme::success())
    } else {
      ("○", theme::text_muted())
    };
    text(symbol).size(11).color(color).into()
  }).collect();

  if overflow > 0 {
    dot_elems.push(
      text(format!("+{overflow}")).size(10)
        .style(|_: &iced::Theme| iced::widget::text::Style {
          color: Some(theme::text_muted()),
        })
        .into()
    );
  }

  row(dot_elems).spacing(2).into()
}
```

### 3.5 Etat vide (aucune room)

```rust
fn empty_rooms_state(s: &'static dyn Strings) -> Element<'_, Message> {
  container(
    column![
      text(s.chat_no_rooms()).size(13).style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::text_muted()),
      }),
      button(text(s.chat_create_room()).size(12))
        .on_press(Message::NavChanged(View::ChatNewRoom))
        .padding([6, 12])
        .style(button_styles::ghost_accent()),
      button(text(s.chat_join_room()).size(12))
        .on_press(Message::NavChanged(View::ChatJoinRoom))
        .padding([6, 12])
        .style(button_styles::ghost_neutral()),
    ]
    .spacing(12)
    .align_x(Alignment::Center),
  )
  .center_x(Length::Fill)
  .padding([24, 16])
  .height(Length::Fill)
  .into()
}
```

La string `chat_no_rooms` a une variante USSR via `flavor()` :
- Normal : "No conversations yet."
- USSR : "No comrades yet. Establish secure communications."

### 3.6 Badge etat MQTT (pied de page)

```rust
fn mqtt_status_bar(state: &MqttState, s: &'static dyn Strings) -> Element<'_, Message> {
  let (dot, label, color) = match state {
    MqttState::Connected => ("●", s.chat_mqtt_connected(), theme::success()),
    MqttState::Connecting => ("◌", s.chat_mqtt_connecting(), theme::peach()),
    MqttState::Reconnecting { .. } => ("◌", s.chat_mqtt_reconnecting(), theme::peach()),
    MqttState::Disconnected => ("✗", s.chat_mqtt_disconnected(), theme::error()),
    MqttState::Failed(_) => ("✗", s.chat_mqtt_failed(), theme::error()),
  };

  container(
    row![
      text(dot).size(10).color(color),
      text(label).size(11).color(color),
    ]
    .spacing(6)
    .align_y(Alignment::Center),
  )
  .padding([6, 12])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    border: Border {
      color: theme::border(),
      width: 0.0,
      radius: 0.0.into(),
    },
    ..Default::default()
  })
  .into()
}
```

---

## 4. Chat panel (fill width)

### 4.1 Structure generale

```rust
// ui/chat_room.rs — fonction publique
pub fn view(room: &Room, messages: &[ChatMessage], app: &App) -> Element<'_, Message> {
  // Banniere MQTT si non connecte (optionnelle, en haut)
  let maybe_banner = if app.mqtt_state != MqttState::Connected {
    Some(mqtt_warning_banner(app.strings))
  } else {
    None
  };

  let header = chat_header(room, app);
  let sep_h = rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
    color: theme::border(),
    radius: 0.0.into(),
    fill_mode: rule::FillMode::Full,
    snap: true,
  });
  let msg_area = message_list(messages, app);
  let sep_compose = rule::horizontal(1).style(/* idem */);
  let compose = compose_bar(&room.id, app);

  let mut col = column![];
  if let Some(banner) = maybe_banner {
    col = col.push(banner);
  }
  col = col.push(header).push(sep_h).push(msg_area).push(sep_compose).push(compose);

  container(col)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::detail_bg())),
      ..Default::default()
    })
    .into()
}
```

### 4.2 Header de conversation

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Nom du salon       ● Alice  ○ Bob  ● Carol       [Copy invite] [Leave] │
└─────────────────────────────────────────────────────────────────────────┘
```

Specs :
- Fond : `header_bg()`
- Padding : `[10, 16]`
- Hauteur approximative : 44px

```rust
fn chat_header(room: &Room, app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let bold = Font { weight: font::Weight::Bold, ..theme::heading_font() };

  let title = text(room.name.as_str())
    .size(15)
    .font(bold);

  // Pastilles avec nom court (fp[:8]) — pas de tooltip dans iced 0.14
  let participant_badges: Vec<Element<Message>> = room.participants.iter()
    .filter(|p| p.fp != room.my_fp)
    .map(|p| {
      let online = app.presence.get(&p.fp)
        .map(|s| *s == PresenceStatus::Online)
        .unwrap_or(false);
      let (dot, color) = if online {
        ("●", theme::success())
      } else {
        ("○", theme::text_muted())
      };
      // Identifier le participant par un nom court : 8 premiers chars du fp
      let short = p.fp.get(..8).unwrap_or(&p.fp);
      row![
        text(dot).size(11).color(color),
        text(short).size(10).font(theme::MONO).style(|_: &iced::Theme| {
          iced::widget::text::Style { color: Some(theme::text_muted()) }
        }),
      ]
      .spacing(3)
      .align_y(Alignment::Center)
      .into()
    })
    .collect();

  let participants_row = row(participant_badges).spacing(10).align_y(Alignment::Center);

  let btn_copy = button(
    row![
      text("\u{f0c5}").font(theme::ICONS).size(11),
      text(s.chat_copy_invite()).size(12),
    ]
    .spacing(4)
    .align_y(Alignment::Center),
  )
  .on_press(Message::ChatJoinCodeCopy(room.id.clone()))
  .padding([4, 8])
  .style(button_styles::ghost_neutral());

  let btn_leave = button(text(s.chat_leave_room()).size(12))
    .on_press(Message::ChatRoomLeave(room.id.clone()))
    .padding([4, 8])
    .style(button_styles::ghost_destructive());

  container(
    row![
      title,
      Space::new().width(8),
      participants_row,
      Space::new().width(Length::Fill),
      btn_copy,
      btn_leave,
    ]
    .spacing(8)
    .align_y(Alignment::Center),
  )
  .padding([10, 16])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::header_bg())),
    ..Default::default()
  })
  .into()
}
```

### 4.3 Banniere MQTT deconnecte

```rust
fn mqtt_warning_banner(s: &'static dyn Strings) -> Element<'_, Message> {
  container(
    row![
      text("\u{f071}").font(theme::ICONS).size(12).color(theme::peach()),
      text(s.chat_mqtt_disconnected_banner()).size(12).color(theme::peach()),
    ]
    .spacing(8)
    .align_y(Alignment::Center),
  )
  .padding([6, 16])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::warning_bg())),
    ..Default::default()
  })
  .into()
}
```

### 4.4 Zone messages (`message_list`)

```rust
fn message_list<'a>(messages: &'a [ChatMessage], app: &'a App) -> Element<'a, Message> {
  let local_fp = app.rooms.iter()
    .find(|r| Some(r.id.as_str()) == app.active_room.as_deref())
    .map(|r| r.my_fp.as_str())
    .unwrap_or("");

  let bubbles: Vec<Element<Message>> = messages.iter()
    .map(|msg| message_bubble(msg, msg.sender_fp == local_fp, app))
    .collect();

  // Auto-scroll vers le bas : utiliser un Id stable par room_id
  let scroll_id = iced::widget::scrollable::Id::new(
    format!("chat-{}", app.active_room.as_deref().unwrap_or("none"))
  );

  scrollable(
    Column::with_children(bubbles)
      .spacing(8)
      .padding([12, 16]),
  )
  .id(scroll_id)
  .style(common::scroll_style)
  .height(Length::Fill)
  .width(Length::Fill)
  .into()
}
```

Auto-scroll : apres chaque `ChatReceived` ou `ChatSent`, emettre
`scrollable::snap_to(scroll_id, scrollable::RelativeOffset::END)` via `Task::done`.

### 4.5 Bulle de message (`message_bubble`)

```
Bulle recue (alignee a gauche) :
┌──────────────────────────────────────────────────┐
│ [AB] Alice · 14:32                               │
│      Texte du message sur plusieurs lignes       │
└──────────────────────────────────────────────────┘

Bulle envoyee (alignee a droite) :
┌──────────────────────────────────────────────────┐
│                           Moi · 14:33 [AB]       │
│ Texte du message                                 │
│                                ✓ alice_fp  ⏳ bob │
└──────────────────────────────────────────────────┘
```

Specs detaillees :

**Avatar** : cercle de 28px de diametre, fond `accent_subtle()`, lettre initiale en `accent()`,
police `heading_font()`, taille 13. L'initiale est `sender_fp[..1].to_uppercase()` (premiere
lettre du fingerprint, donc toujours hexadecimal 0-9 A-F). Implementer avec un `container`
carre de `28x28` avec `border.radius = 14.0.into()`.

**Auteur** : `text_muted()`, taille 11. Afficher les 8 premiers chars du fingerprint si l'on ne
connait pas le nom (l'associacion fp → nom requiert de chercher dans `app.keys`).

**Horodatage** : `text_muted()`, taille 10, format `HH:MM` (heure locale).

**Fond des bulles** :
- Recue : `card_bg()` avec bordure gauche de 2px `accent_border()`, `border.radius = 8.0.into()`
- Envoyee : `accent_subtle()` avec `border.radius = 8.0.into()`, pas de bordure laterale

**Largeur maximale des bulles** : `max_width(480)` pour eviter les lignes trop longues.

**Message de decryptage echoue** : bulle grisee, texte `s.chat_decrypt_failed()` en
`text_muted()`, icone `\u{f071}` (warning), fond `disabled_bg()` a 30% d'opacite.

```rust
fn message_bubble<'a>(msg: &'a ChatMessage, is_own: bool, app: &'a App) -> Element<'a, Message> {
  let s = app.strings;
  let ts = format_time_short(msg.ts);

  // Resolution du nom depuis le keyring (meilleur effort)
  let sender_name: String = app.keys.iter()
    .find(|k| k.fingerprint == msg.sender_fp)
    .map(|k| k.name.clone())
    .unwrap_or_else(|| msg.sender_fp.get(..8).unwrap_or(&msg.sender_fp).to_string());

  // Initiale pour l'avatar
  let initial = msg.sender_fp.get(..1).unwrap_or("?").to_uppercase();

  let avatar = container(
    text(initial.as_str()).size(13).font(theme::heading_font()).color(theme::accent()),
  )
  .width(28)
  .height(28)
  .center_x(Length::Fill)
  .center_y(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::accent_subtle())),
    border: Border { radius: 14.0.into(), ..Default::default() },
    ..Default::default()
  });

  let meta = row![
    text(sender_name.as_str()).size(11).style(|_: &iced::Theme| iced::widget::text::Style {
      color: Some(theme::text_muted()),
    }),
    text(" · ").size(10).style(/* text_muted */),
    text(ts.as_str()).size(10).style(/* text_muted */),
  ]
  .spacing(0)
  .align_y(Alignment::Center);

  let body_text = text(msg.text.as_str()).size(13).style(|_: &iced::Theme| {
    iced::widget::text::Style { color: Some(theme::text_strong()) }
  });

  let bubble_content = if is_own {
    // Bulle envoyee : meta + corps + ack indicators
    let acks = ack_indicators(&msg.acks, app);
    column![meta, body_text, acks].spacing(4)
  } else {
    column![meta, body_text].spacing(4)
  };

  let bubble_style = move |_: &iced::Theme| {
    if is_own {
      container::Style {
        background: Some(Background::Color(theme::accent_subtle())),
        border: Border { radius: 8.0.into(), ..Default::default() },
        ..Default::default()
      }
    } else {
      container::Style {
        background: Some(Background::Color(theme::card_bg())),
        border: Border {
          color: theme::accent_border(),
          width: 2.0,
          radius: 8.0.into(),
        },
        ..Default::default()
      }
    }
  };

  let bubble = container(bubble_content)
    .padding([8, 12])
    .max_width(480)
    .style(bubble_style);

  if is_own {
    // Aligne a droite : avatar a droite du texte
    row![
      Space::new().width(Length::Fill),
      bubble,
      avatar,
    ]
    .spacing(8)
    .align_y(Alignment::Top)
    .into()
  } else {
    // Aligne a gauche : avatar a gauche du texte
    row![
      avatar,
      bubble,
      Space::new().width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Top)
    .into()
  }
}
```

### 4.6 Indicateurs ACK (`ack_indicators`)

Affichee uniquement sous les bulles envoyees. Un indicateur par participant (hors soi-meme).

- ACK recu : `"✓"` (U+2713) + fp court (8 chars) en `success()`, taille 10
- ACK en attente : `"⏳"` (U+23F3) + fp court en `text_muted()`, taille 10

```rust
fn ack_indicators<'a>(
  acks: &'a HashMap<String, AckStatus>,
  app: &'a App,
) -> Element<'a, Message> {
  let room = app.rooms.iter()
    .find(|r| Some(r.id.as_str()) == app.active_room.as_deref());
  let my_fp = room.map(|r| r.my_fp.as_str()).unwrap_or("");

  let indicators: Vec<Element<Message>> = acks.iter()
    .filter(|(fp, _)| fp.as_str() != my_fp)
    .map(|(fp, status)| {
      let short = fp.get(..8).unwrap_or(fp.as_str());
      let (symbol, color) = match status {
        AckStatus::Received => ("✓", theme::success()),
        AckStatus::Pending => ("⏳", theme::text_muted()),
      };
      row![
        text(symbol).size(10).color(color),
        text(short).size(10).font(theme::MONO).color(color),
      ]
      .spacing(2)
      .align_y(Alignment::Center)
      .into()
    })
    .collect();

  row(indicators).spacing(8).into()
}
```

### 4.7 Barre de saisie (`compose_bar`)

```
┌──────────────────────────────────────────────────────────────┐
│  [ Type a message...                                    ] [►] │
└──────────────────────────────────────────────────────────────┘
```

Fond : `card_bg()`. Padding : `[8, 12]`. Hauteur approximative : 48px.

```rust
fn compose_bar(room_id: &str, app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let can_send = !app.chat_input.is_empty() && app.mqtt_state == MqttState::Connected;

  let input = text_input(s.chat_type_message(), &app.chat_input)
    .on_input(Message::ChatInputChanged)
    .on_submit(Message::ChatSend)
    .padding([8, 12])
    .size(13)
    .width(Length::Fill)
    .style(|_: &iced::Theme, _status| text_input::Style {
      background: Background::Color(theme::detail_bg()),
      border: Border {
        color: theme::border(),
        width: 1.0,
        radius: 6.0.into(),
      },
      placeholder: theme::text_muted(),
      value: theme::text_strong(),
      selection: theme::accent_subtle(),
      icon: theme::text_muted(),
    });

  let send_btn = button(
    text("\u{f1d8}").font(theme::ICONS).size(14).color(theme::text_on_accent()),
  )
  .padding([8, 12])
  .style(button_styles::primary_toggle(can_send));
  let send_btn = if can_send {
    send_btn.on_press(Message::ChatSend)
  } else {
    send_btn
  };

  container(
    row![input, send_btn]
      .spacing(8)
      .align_y(Alignment::Center),
  )
  .padding([8, 12])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::card_bg())),
    ..Default::default()
  })
  .into()
}
```

Note : `\u{f1d8}` est FA4 "send" (avion en papier, code point 0xf1d8 = 61912). Valide dans
la plage FA4 `\u{f000}`–`\u{f2e0}`.

### 4.8 Etat vide (aucune room selectionnee)

```rust
fn empty_chat_state(s: &'static dyn Strings) -> Element<'_, Message> {
  container(
    text(s.chat_select_room()).size(13).style(|_: &iced::Theme| iced::widget::text::Style {
      color: Some(theme::text_muted()),
    }),
  )
  .center_x(Length::Fill)
  .center_y(Length::Fill)   // ici center_y est acceptable car c'est un splash vide
  .width(Length::Fill)
  .height(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::detail_bg())),
    ..Default::default()
  })
  .into()
}
```

---

## 5. Modals

### 5.1 Creer un salon (`View::ChatNewRoom`)

Utilise le pattern `page_layout(card_medium(...))` identique a `create_key.rs`.
Implementer dans `ui/chat_modals.rs`.

```rust
// ui/chat_modals.rs
pub fn create_room_view(app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let form = &app.chat_new_form;

  let can_submit = !form.name.trim().is_empty()
    && !form.relay.trim().is_empty()
    && !form.participants_input.trim().is_empty();

  let content = column![
    // Titre de page
    text(theme::flavor(s.chat_create_room_title(), s.chat_create_room_title_ussr()))
      .size(22)
      .font(theme::flavor_title_font()),

    // Champ nom
    column![
      text(s.chat_room_name_label()).size(12).style(/* text_muted */),
      text_input(s.chat_room_name_placeholder(), &form.name)
        .on_input(Message::ChatRoomNameChanged)
        .padding([8, 12])
        .size(13),
    ].spacing(4),

    // Champ relay MQTT
    column![
      text(s.chat_relay_label()).size(12).style(/* text_muted */),
      text_input(s.chat_relay_placeholder(), &form.relay)
        .on_input(Message::ChatRoomRelayChanged)
        .padding([8, 12])
        .size(13),
      text(s.chat_relay_hint()).size(11).style(/* text_muted */),
    ].spacing(4),

    // Textarea participants (fingerprints, un par ligne)
    column![
      text(s.chat_participants_label()).size(12).style(/* text_muted */),
      text_editor(&app.chat_new_form_participants_editor)  // text_editor::Content
        .on_action(|a| Message::ChatRoomParticipantsChanged(/* ... */))
        .height(120)
        .padding([8, 12]),
      text(s.chat_participants_hint()).size(11).style(/* text_muted */),
    ].spacing(4),

    // Boutons
    row![
      button(text(s.btn_cancel()).size(13))
        .on_press(Message::NavBack)
        .padding([8, 16])
        .style(button_styles::ghost_neutral()),
      Space::new().width(Length::Fill),
      button(text(s.chat_create_room_btn()).size(13))
        .on_press_maybe(can_submit.then_some(Message::ChatRoomCreate))
        .padding([8, 16])
        .style(button_styles::primary_toggle(can_submit)),
    ]
    .align_y(Alignment::Center),
  ]
  .spacing(20);

  common::page_layout(common::card_medium(content))
}
```

Note sur le textarea participants : `ChatRoomParticipantsChanged` prend un `String` (la valeur
brute du textarea). Le parsing (split sur newlines, validation 40-hex) est fait dans le handler.
Cependant iced 0.14 utilise `text_editor::Content` (pas `text_input`) pour les zones multi-lignes.
Le champ `chat_new_form.participants_input: String` est la valeur serialisee. Le `Content` doit
etre tenu en `App` ou reconstruit depuis `participants_input` a chaque render.
Recommandation : ajouter `pub participants_editor: text_editor::Content` dans `ChatNewForm`.

### 5.2 Rejoindre un salon (`View::ChatJoinRoom`)

```rust
pub fn join_room_view(app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let form = &app.chat_new_form;
  let can_submit = !form.join_code.trim().is_empty();

  let content = column![
    text(s.chat_join_room_title()).size(22).font(theme::flavor_title_font()),

    column![
      text(s.chat_join_code_label()).size(12).style(/* text_muted */),
      text_input(s.chat_join_code_placeholder(), &form.join_code)
        .on_input(Message::ChatJoinCodeChanged)
        .padding([8, 12])
        .size(13),
      text(s.chat_join_code_hint()).size(11).style(/* text_muted */),
    ].spacing(4),

    row![
      button(text(s.btn_cancel()).size(13))
        .on_press(Message::NavBack)
        .padding([8, 16])
        .style(button_styles::ghost_neutral()),
      Space::new().width(Length::Fill),
      button(text(s.chat_join_btn()).size(13))
        .on_press_maybe(can_submit.then_some(Message::ChatRoomJoin))
        .padding([8, 16])
        .style(button_styles::primary_toggle(can_submit)),
    ]
    .align_y(Alignment::Center),
  ]
  .spacing(20);

  common::page_layout(common::card_medium(content))
}
```

### 5.3 Selection d'identite (modal overlay)

Affichee quand l'utilisateur clique sur une room alors qu'il a plusieurs clefs privees et
que `room.my_fp` n'est pas encore fixe. Elle s'affiche EN OVERLAY sur `View::ChatList`
(pas en view distincte) via `PendingOp::IdentitySelection { room_id, selected_fp }`.

Le rendu d'overlay suit le meme pattern que `migration_modal` et `delete_modal` dans
`key_detail.rs` : on encapsule le contenu existant dans un `stack![]` (si iced 0.14 le
supporte) ou on remplace le contenu du panneau droit par le modal. En iced 0.14, il n'y a
pas de primitif `overlay` — le pattern utilise est de remplacer le panneau de droite.

Comportement : quand `PendingOp::IdentitySelection` est present et `app.view == View::ChatList`,
`chat_panel()` retourne le modal de selection au lieu du panel vide.

```
┌─ Choisir votre identite ───────────────────────────────────┐
│ Vous avez plusieurs clefs privees.                         │
│ Selectionnez celle a utiliser dans ce salon :              │
│                                                            │
│  ○  Alice Pro <alice@work.example>                         │
│     ABCD 1234 EFGH 5678 ...                                │
│                                                            │
│  ○  Alice Perso <alice@home.example>                       │
│     AAAA BBBB CCCC DDDD ...                                │
│                                                            │
│                         [Annuler]  [Entrer dans le salon]  │
└────────────────────────────────────────────────────────────┘
```

```rust
fn identity_selection_modal<'a>(
  room_id: &str,
  selected_fp: &Option<String>,
  app: &'a App,
) -> Element<'a, Message> {
  let s = app.strings;
  let private_keys: Vec<_> = app.keys.iter().filter(|k| k.has_secret).collect();

  let radio_items: Vec<Element<Message>> = private_keys.iter().map(|key| {
    let is_selected = selected_fp.as_deref() == Some(key.fingerprint.as_str());
    column![
      radio(
        format!("{} <{}>", key.name, key.email),
        key.fingerprint.clone(),
        selected_fp.clone(),
        |fp| Message::ChatIdentitySelected(fp),
      )
      .style(common::radio_style)
      .text_size(13),
      text(key.fingerprint.as_str())
        .size(10)
        .font(theme::MONO)
        .style(/* text_muted */),
    ]
    .spacing(2)
    .padding([4, 24])  // indent sous le radio
    .into()
  }).collect();

  let can_enter = selected_fp.is_some();

  let content = column![
    text(s.chat_choose_identity_title()).size(16).font(theme::heading_font()),
    text(s.chat_choose_identity_hint()).size(13).style(/* text_secondary */),
    Column::with_children(radio_items).spacing(8),
    row![
      button(text(s.btn_cancel()).size(13))
        .on_press(Message::NavBack)
        .padding([8, 16])
        .style(button_styles::ghost_neutral()),
      Space::new().width(Length::Fill),
      button(text(s.chat_enter_room_btn()).size(13))
        .on_press_maybe(can_enter.then_some(
          Message::ChatRoomSelected(room_id.to_string())
        ))
        .padding([8, 16])
        .style(button_styles::primary_toggle(can_enter)),
    ]
    .align_y(Alignment::Center),
  ]
  .spacing(16)
  .padding(32);

  container(content)
    .max_width(480)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::card_bg())),
      border: Border { radius: 12.0.into(), ..Default::default() },
      ..Default::default()
    })
    .into()
}
```

Le `Message::ChatIdentitySelected(fp)` est un nouveau message a ajouter. Il met a jour
`PendingOp::IdentitySelection.selected_fp`.

### 5.4 Quitter un salon (modal overlay)

Meme principe : `PendingOp::LeaveRoom(room_id)`. Affichee dans `chat_panel` si ce pending
est present.

```
┌─ Quitter le salon ? ───────────────────────────────────────┐
│ Vous ne recevrez plus les messages de "nom-du-salon".      │
│ Cette action est irreversible. Pour rejoindre a nouveau,   │
│ vous aurez besoin d'une nouvelle invitation.               │
│                                                            │
│                     [Annuler]  [Quitter le salon]          │
└────────────────────────────────────────────────────────────┘
```

```rust
fn leave_room_modal<'a>(room: &'a Room, s: &'static dyn Strings) -> Element<'a, Message> {
  let content = column![
    text(s.chat_leave_confirm_title()).size(16).font(theme::heading_font()),
    text(s.chat_leave_confirm_body_with_name(room.name.as_str())).size(13)
      .style(/* text_secondary */),
    row![
      button(text(s.btn_cancel()).size(13))
        .on_press(Message::MoveToCardCancel)  // reutilise NavBack ou PendingOp clear
        .padding([8, 16])
        .style(button_styles::ghost_neutral()),
      Space::new().width(Length::Fill),
      button(text(s.chat_leave_room_btn()).size(13))
        .on_press(Message::ChatRoomLeave(room.id.clone()))
        .padding([8, 16])
        .style(button_styles::ghost_destructive()),
    ]
    .align_y(Alignment::Center),
  ]
  .spacing(16)
  .padding(32);

  container(content)
    .max_width(480)
    .style(/* card_bg + radius 12 */)
    .into()
}
```

Pour `chat_leave_confirm_body_with_name` : c'est une methode avec parametre String (comme
`key_list_error`). Retourne `String`, pas `&'static str`.

---

## 6. i18n — liste exhaustive des nouvelles methodes Strings

Le rust-engineer ajoute exactement ces methodes dans le trait `Strings`, puis les implemente
dans `english.rs` et `french.rs`. Les methodes retournant `String` (parametrees) sont indiquees
par le type dans la colonne "signature".

### 6.1 Navigation / sidebar

| methode | signature | EN | FR |
|---|---|---|---|
| `nav_section_chat` | `&'static str` | "CHAT" | "CHAT" |
| `nav_chat_rooms` | `&'static str` | "Rooms" | "Salons" |
| `nav_chat_rooms_ussr` | `&'static str` | "TRANSMISSIONS" | "TRANSMISSIONS" |

### 6.2 Room list

| methode | signature | EN | FR |
|---|---|---|---|
| `chat_no_rooms` | `&'static str` | "No conversations yet." | "Aucune conversation." |
| `chat_no_rooms_ussr` | `&'static str` | "No comrades yet. Establish secure communications." | "Pas encore de camarades. Etablissez des communications securisees." |
| `chat_create_room` | `&'static str` | "+ New" | "+ Nouveau" |
| `chat_join_room` | `&'static str` | "Join" | "Rejoindre" |

### 6.3 Etat MQTT

| methode | signature | EN | FR |
|---|---|---|---|
| `chat_mqtt_connected` | `&'static str` | "Connected" | "Connecte" |
| `chat_mqtt_connecting` | `&'static str` | "Connecting..." | "Connexion..." |
| `chat_mqtt_reconnecting` | `&'static str` | "Reconnecting..." | "Reconnexion..." |
| `chat_mqtt_disconnected` | `&'static str` | "Disconnected" | "Deconnecte" |
| `chat_mqtt_failed` | `&'static str` | "Connection failed" | "Echec de connexion" |
| `chat_mqtt_disconnected_banner` | `&'static str` | "Disconnected — reconnecting..." | "Deconnecte — reconnexion en cours..." |

### 6.4 Header de conversation

| methode | signature | EN | FR |
|---|---|---|---|
| `chat_copy_invite` | `&'static str` | "Copy invite" | "Copier l'invitation" |
| `chat_leave_room` | `&'static str` | "Leave" | "Quitter" |

### 6.5 Zone messages

| methode | signature | EN | FR |
|---|---|---|---|
| `chat_decrypt_failed` | `&'static str` | "Could not decrypt this message" | "Impossible de dechiffrer ce message" |
| `chat_type_message` | `&'static str` | "Type a message..." | "Ecrire un message..." |
| `chat_select_room` | `&'static str` | "Select a room to start chatting." | "Selectionnez un salon pour discuter." |

### 6.6 Barre de saisie

| methode | signature | EN | FR |
|---|---|---|---|
| `chat_send` | `&'static str` | "Send" | "Envoyer" |

### 6.7 Modal creer un salon

| methode | signature | EN | FR |
|---|---|---|---|
| `chat_create_room_title` | `&'static str` | "Create a room" | "Creer un salon" |
| `chat_create_room_title_ussr` | `&'static str` | "ESTABLISH SECURE CHANNEL" | "ETABLIR UN CANAL SECURISE" |
| `chat_room_name_label` | `&'static str` | "Room name" | "Nom du salon" |
| `chat_room_name_placeholder` | `&'static str` | "e.g. Ops Team" | "ex. Equipe ops" |
| `chat_relay_label` | `&'static str` | "MQTT relay" | "Relais MQTT" |
| `chat_relay_placeholder` | `&'static str` | "mqtts://host:8883" | "mqtts://host:8883" |
| `chat_relay_hint` | `&'static str` | "TLS required. Use your own broker for maximum privacy." | "TLS requis. Utilisez votre propre broker pour une confidentialite maximale." |
| `chat_participants_label` | `&'static str` | "Participants (one fingerprint per line)" | "Participants (un fingerprint par ligne)" |
| `chat_participants_hint` | `&'static str` | "Add participants' PGP fingerprints, one per line." | "Ajoutez les fingerprints PGP des participants, un par ligne." |
| `chat_create_room_btn` | `&'static str` | "Create room" | "Creer le salon" |

### 6.8 Modal rejoindre un salon

| methode | signature | EN | FR |
|---|---|---|---|
| `chat_join_room_title` | `&'static str` | "Join a room" | "Rejoindre un salon" |
| `chat_join_code_label` | `&'static str` | "Join code" | "Code d'invitation" |
| `chat_join_code_placeholder` | `&'static str` | "pgpilot:join:..." | "pgpilot:join:..." |
| `chat_join_code_hint` | `&'static str` | "Paste the invite code you received." | "Collez le code d'invitation que vous avez recu." |
| `chat_join_btn` | `&'static str` | "Join room" | "Rejoindre le salon" |

### 6.9 Modal selection d'identite

| methode | signature | EN | FR |
|---|---|---|---|
| `chat_choose_identity_title` | `&'static str` | "Choose your identity" | "Choisissez votre identite" |
| `chat_choose_identity_hint` | `&'static str` | "You have multiple private keys. Select the one to use in this room:" | "Vous avez plusieurs clefs privees. Selectionnez celle a utiliser dans ce salon :" |
| `chat_enter_room_btn` | `&'static str` | "Enter room" | "Entrer dans le salon" |

### 6.10 Modal quitter un salon

| methode | signature | EN | FR |
|---|---|---|---|
| `chat_leave_confirm_title` | `&'static str` | "Leave room?" | "Quitter le salon ?" |
| `chat_leave_confirm_body_with_name` | `fn(&self, name: &str) -> String` | "You will no longer receive messages from \"{name}\". This cannot be undone — you would need a new invite to rejoin." | "Vous ne recevrez plus les messages de \"{name}\". Cette action est irreversible — vous auriez besoin d'une nouvelle invitation pour rejoindre." |
| `chat_leave_room_btn` | `&'static str` | "Leave room" | "Quitter le salon" |

### 6.11 Status messages chat

| methode | signature | EN | FR |
|---|---|---|---|
| `status_chat_room_created` | `&'static str` | "Room created." | "Salon cree." |
| `status_chat_room_joined` | `&'static str` | "Room joined." | "Salon rejoint." |
| `status_chat_room_left` | `&'static str` | "You have left the room." | "Vous avez quitte le salon." |
| `status_chat_invite_copied` | `&'static str` | "Invite code copied." | "Code d'invitation copie." |
| `status_chat_message_sent` | `&'static str` | "Message sent." | "Message envoye." |

### 6.12 Erreurs chat

| methode | signature | EN | FR |
|---|---|---|---|
| `err_chat_room_create_failed` | `&'static str` | "Failed to create room." | "Echec de creation du salon." |
| `err_chat_room_join_failed` | `&'static str` | "Failed to join room." | "Echec de rejoindre le salon." |
| `err_chat_room_leave_failed` | `&'static str` | "Failed to leave room." | "Echec de quitter le salon." |
| `err_chat_send_failed` | `&'static str` | "Failed to send message." | "Echec d'envoi du message." |
| `err_chat_invite_copy_failed` | `&'static str` | "Failed to copy invite code." | "Echec de copie du code d'invitation." |

---

## 7. Icones FA4 (plage \u{f000}–\u{f2e0} uniquement)

Toutes les icones ci-dessous sont validees dans la plage FA4 du Symbols Nerd Font Mono.
Ne jamais utiliser de codepoints au-dela de `\u{f2e0}`.

| Usage | Codepoint | Nom FA4 | Caractere |
|---|---|---|---|
| Salon / nav chat | `\u{f0e5}` | fa-comment | icone bulle dialogue |
| Envoyer message | `\u{f1d8}` | fa-paper-plane | avion en papier |
| Copier (clipboard) | `\u{f0c5}` | fa-copy | double feuilles |
| Lien / invite | `\u{f0c1}` | fa-link | maillon de chaine |
| Ajouter (+ new) | `\u{f067}` | fa-plus | signe plus |
| Connexion MQTT ok | utiliser `"●"` Unicode direct, pas d'icone Nerd Font | | |
| Deconnecte | utiliser `"✗"` Unicode direct | | |
| Reconnexion | utiliser `"◌"` Unicode direct | | |
| Warning banniere | `\u{f071}` | fa-exclamation-triangle | triangle avertissement |
| Quitter | pas d'icone — texte seul "Leave" / "Quitter" | | |
| Presence online | `"●"` U+25CF Unicode direct, couleur `success()` | | |
| Presence offline | `"○"` U+25CB Unicode direct, couleur `text_muted()` | | |

**Rappel important** : les pastilles de presence (`●`/`○`) et les indicateurs de connexion MQTT
(`●`/`◌`/`✗`) sont des caracteres Unicode standards, pas des codepoints Nerd Font. Ils
s'affichent avec la police DEFAULT, pas ICONS.

---

## 8. Fichiers a creer et a modifier

### 8.1 Fichiers crees

| Fichier | Contenu |
|---|---|
| `src/ui/chat.rs` | `pub fn view(app)`, `room_list_panel`, `room_row`, `presence_dots`, `mqtt_status_bar`, `empty_rooms_state`, `chat_panel`, `message_list`, `message_bubble`, `ack_indicators`, `compose_bar`, `empty_chat_state`, `chat_header`, `mqtt_warning_banner` |
| `src/ui/chat_modals.rs` | `pub fn create_room_view(app)`, `pub fn join_room_view(app)`, `identity_selection_modal`, `leave_room_modal` |

### 8.2 Fichiers modifies

| Fichier | Modification |
|---|---|
| `src/ui/mod.rs` | 1. Ajouter `pub mod chat;` et `pub mod chat_modals;`. 2. Dans `root()`, router `View::ChatList` et `View::ChatRoom(_)` vers `chat::view(app)`, `View::ChatNewRoom` vers `chat_modals::create_room_view(app)`, `View::ChatJoinRoom` vers `chat_modals::join_room_view(app)`. 3. Dans `sidebar()`, inserer le cluster CHAT. |
| `src/app/mod.rs` | Ajouter variants `PendingOp::IdentitySelection { room_id: String, selected_fp: Option<String> }` et `PendingOp::LeaveRoom(String)`. Ajouter `Message::ChatIdentitySelected(String)`. Ajouter `pub participants_editor: text_editor::Content` dans `ChatNewForm`. |
| `src/i18n/mod.rs` | Ajouter les 42 methodes de la §6 dans le trait `Strings`. |
| `src/i18n/english.rs` | Implementer les 42 methodes en anglais. |
| `src/i18n/french.rs` | Implementer les 42 methodes en francais. |

### 8.3 PendingOp a ajouter dans `app/mod.rs`

```rust
pub enum PendingOp {
  Migration(String),
  Delete(String),
  Renewal(PendingRenewal),
  ExportPubMenu(String),
  Publish(Keyserver),
  // --- v0.6.0 Chat ---
  IdentitySelection { room_id: String, selected_fp: Option<String> },
  LeaveRoom(String),
}
```

---

## 9. Comportements et cas limites

### 9.1 Auto-scroll

A chaque message entrant (`ChatReceived`) ou message envoye (`ChatSent`), retourner depuis
le handler :

```rust
Task::batch([
  self.set_status(...),
  scrollable::snap_to(
    scrollable::Id::new(format!("chat-{}", room_id)),
    scrollable::RelativeOffset::END,
  ),
])
```

### 9.2 Changement de room actif

Quand `ChatRoomSelected(id)` est traite :
1. `self.active_room = Some(id.clone())`
2. `self.chat_input = String::new()` (vider la saisie)
3. `self.view = View::ChatRoom(id.clone())`
4. Si l'utilisateur a plusieurs clefs privees : emettre `PendingOp::IdentitySelection`

### 9.3 Indicateur "Reconnecting" — anti-spam visuel

Le `MqttState::Reconnecting { attempt }` est affiche sans le numero de tentative dans l'UI.
La string `chat_mqtt_reconnecting` est toujours "Reconnecting..." independamment du champ
`attempt`. Le champ `attempt` est reserve aux logs.

### 9.4 Participants — affichage du nom

Dans les bulles et les pastilles du header, le nom prefere est :
1. `app.keys.iter().find(|k| k.fingerprint == fp).map(|k| k.name.as_str())`
2. Fallback : `&fp[..8]` (8 premiers chars du fingerprint en MONO)

Ne jamais afficher le fingerprint complet dans l'UI conversationnelle — trop verbeux.

### 9.5 Message de decryptage echoue

Si le `ChatMessage.text` commence par le prefixe interne `"[DECRYPT_FAILED]"` (convention
etablie par `app/chat.rs`), afficher une bulle grisee avec `s.chat_decrypt_failed()`.
Ce prefixe est une convention interne — le rust-engineer de l'axe 5 coordonne avec
celui de l'axe 4 pour confirmer cette convention ou en choisir une autre.

### 9.6 Sidebar : item actif pour toutes les vues chat

L'item "Salons" doit etre en etat actif (fond `accent()`) pour toutes les vues suivantes :
```rust
let chat_active = matches!(
  app.view,
  View::ChatList | View::ChatRoom(_) | View::ChatNewRoom | View::ChatJoinRoom
);
```
Adapter la closure `nav_btn` en consequence ou creer une variante `nav_btn_active`.

---

## 10. Helpers locaux (non publics)

Ces fonctions utilitaires sont privees dans `ui/chat.rs` :

```rust
/// Formate un datetime UTC en "HH:MM" heure locale.
fn format_time_short(dt: chrono::DateTime<chrono::Utc>) -> String {
  use chrono::Local;
  dt.with_timezone(&Local).format("%H:%M").to_string()
}

/// Tronque un texte a max_chars caracteres avec "…" si necessaire.
fn truncate_preview(s: &str, max_chars: usize) -> String {
  let mut chars = s.chars();
  let truncated: String = chars.by_ref().take(max_chars).collect();
  if chars.next().is_some() {
    format!("{truncated}…")
  } else {
    truncated
  }
}
```

---

## 11. Checklist de validation visuelle (pour la revue)

Avant de merger l'axe 5, verifier visuellement :

- [ ] Cluster CHAT visible dans sidebar en theme Catppuccin ET USSR
- [ ] Theme USSR : titres "TRANSMISSIONS", "ETABLIR UN CANAL SECURISE" en Bebas Neue
- [ ] Theme USSR : texte "No comrades yet." dans l'etat vide
- [ ] Pastilles ●/○ correctement colorees (success/text_muted)
- [ ] Room selectionnee : fond `accent_subtle()` + bordure `accent_border()`
- [ ] Bulles recues : bordure gauche 2px `accent_border()`, fond `card_bg()`
- [ ] Bulles envoyees : fond `accent_subtle()`, alignees a droite
- [ ] Avatars : cercle 28px, initiale hexadecimale (0-9 A-F)
- [ ] Barre de saisie : bouton envoyer desactive si champ vide ou MQTT non connecte
- [ ] Auto-scroll vers le bas a chaque nouveau message
- [ ] Banniere warning MQTT si `mqtt_state != Connected`
- [ ] Badge MQTT en pied de panneau avec la bonne couleur
- [ ] Modal creer/rejoindre : `card_medium` centre avec `page_layout`
- [ ] Modal identite : radio picker, bouton "Enter room" desactive si rien selectionne
- [ ] Modal quitter : bouton "Leave room" en `ghost_destructive`
- [ ] Zero string hardcodee en francais dans `src/ui/chat*.rs`
- [ ] `cargo clippy -- -D warnings` passe sans warning
