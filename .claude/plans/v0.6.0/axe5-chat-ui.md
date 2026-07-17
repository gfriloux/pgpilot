# Axe 5 — UI chat

## Objectif

Implémenter les vues iced du chat : liste des salons, conversation, modal de création/rejoindre,
en cohérence stricte avec les patterns visuels existants (thème Catppuccin + USSR, fonts, scrolls,
boutons). Aucune logique métier — uniquement l'UI.

**Référence** : `axe2-spec-finale.md`, `src/ui/key_list.rs` (pattern master-detail à réutiliser)

---

## T5.1 — Maquette et cohérence visuelle

**Complexité** : S
**Agent** : `voltagent-core-dev:ui-designer`
**Dépendances** : T2.3

### Ce qui est à produire

Un document `axe5-ui-spec.md` décrivant :

**1. Layout général**

```
SIDEBAR (180px) │ ROOM LIST (280px) │ CHAT PANEL (reste)
```

Cohérence avec `key_list.rs` : même structure master-detail, même séparateurs, même padding.

**2. Room list (panneau gauche)**

Chaque ligne de room :
- Nom du salon (heading_font en USSR)
- Pastilles de présence des participants : ● (online, accent()) / ○ (offline, text_muted())
- Badge nb messages non lus (accent_subtle bg, text fort)
- État de connexion MQTT en bas : "Connected" (success()) / "Reconnecting..." (peach())

**3. Chat panel (panneau droit)**

- **Header** : nom du salon + pastilles participants + room_id tronqué + bouton [📋 Copy invite]
- **Zone messages** (scrollable, fill height) :
  - Bulles reçues : gauche, fond card_bg(), texte text_strong()
  - Bulles envoyées : droite, fond accent_subtle(), texte text_strong()
  - Horodatage : text_muted(), petit
  - Indicateurs ACK sous les bulles envoyées : `✓ Bob` / `⏳ Carol`
  - Icône 🔒 cliquable sur chaque bulle → tooltip "Signed by {fp_short}"
- **Barre de saisie** :
  - `TextInput` fill width
  - Bouton [►] envoyer (primary button style)
  - Enter = envoyer, Shift+Enter = saut de ligne

**4. Modals**

- **Créer un salon** : champ nom + picker fingerprints depuis keyring + [Créer]
- **Rejoindre** : textarea join code + [Rejoindre]
- **Partager** : affiche le join code + [📋 Copier]

**5. États vides / erreurs**

- Aucune room → "No conversations yet. Create or join a room." (flavor USSR : "No comrades yet. Establish secure communications.")
- MQTT déconnecté → bannière en haut du panel : "⚠ Disconnected — reconnecting..."
- Décryption échouée → bulle grisée : "⚠ Could not decrypt this message"

**6. i18n**

Liste de toutes les nouvelles méthodes `Strings` nécessaires pour l'UI chat.

**Commit** : aucun — livrable = `axe5-ui-spec.md`

---

## T5.2 — Sidebar cluster CHAT

**Complexité** : S
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T5.1, T4.3

### Fichier modifié : `src/ui/mod.rs`

Ajouter dans le cluster `OUTILS` ou créer un nouveau cluster `CHAT` :

```rust
// Nouveau cluster dans la sidebar
sidebar_section(s.nav_section_chat(), vec![
    nav_item(s.nav_chat_rooms(), View::ChatList, &icons::CHAT, app),
])
```

Ajouter `View::ChatList` et `View::ChatRoom(String)` à l'enum `View`.

**Commit** : `feat(chat/ui): add CHAT cluster to sidebar`

---

## T5.3 — Vue `ChatList` (room list)

**Complexité** : M
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T5.2

### Fichier : `src/ui/chat.rs`

```rust
pub fn view(app: &App) -> Element<Message> {
    // Layout : room list panel (gauche 280px) + chat panel (droite)
    // Si active_room = None → placeholder "Select a room"
}

fn room_list_panel(app: &App) -> Element<Message>;
fn room_row(room: &Room, app: &App) -> Element<Message>;
fn presence_dots(participants: &[String], presence: &HashMap<String, PresenceStatus>) -> Element<Message>;
fn mqtt_status_bar(connected: bool) -> Element<Message>;
fn empty_state(s: &'static dyn Strings) -> Element<Message>;
```

**Commit** : `feat(chat/ui): room list view with presence indicators`

---

## T5.4 — Vue `ChatRoom` (conversation)

**Complexité** : L
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T5.3

### Fichier : `src/ui/chat_room.rs`

```rust
pub fn view(room: &Room, messages: &[ChatMessage], app: &App) -> Element<Message>;

fn chat_header(room: &Room, app: &App) -> Element<Message>;
fn message_list(messages: &[ChatMessage], app: &App) -> Element<Message>;
fn message_bubble(msg: &ChatMessage, is_own: bool, app: &App) -> Element<Message>;
fn ack_indicators(acks: &HashMap<String, AckStatus>, room: &Room) -> Element<Message>;
fn compose_bar(room_id: &str, app: &App) -> Element<Message>;
```

**Note importante** : le scroll de la liste de messages doit auto-scroll vers le bas à chaque
nouveau message. Utiliser `scrollable::snap_to(Id, RelativeOffset::END)` ou équivalent iced 0.14.

**Commit** : `feat(chat/ui): chat room view with bubbles, ACK indicators, compose bar`

---

## T5.5 — Modal sélection d'identité

**Complexité** : S
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T5.3, T4.4

### Comportement

Affiché quand `Message::ChatIdentityRequired(room_id)` est émis et que l'utilisateur possède
**plusieurs clefs privées**.

```
┌─ Choose your identity ─────────────────────┐
│ You have multiple private keys.            │
│ Select the one to use in this room:        │
│                                            │
│ ○  Alice Pro <alice@work.example>          │
│    FP: ABCD...1234                         │
│                                            │
│ ○  Alice Perso <alice@personal.example>    │
│    FP: EFGH...5678                         │
│                                            │
│              [Cancel]  [Enter room →]      │
└────────────────────────────────────────────┘
```

- Liste des clefs privées du keyring (filtrée sur `has_secret = true`)
- Radio picker — une seule sélection
- Bouton "Enter room" actif uniquement si une identité est sélectionnée
- Bouton "Cancel" → `Message::NavBack`
- i18n : `s.chat_choose_identity()`, `s.chat_enter_room()`, etc.

**`PendingOp`** : `PendingOp::IdentitySelection { room_id, selected_fp: Option<String> }`

**Commit** : `feat(chat/ui): identity selection modal before entering a room`

---

## T5.6 — Modal leave room + bouton Leave dans le header

**Complexité** : S
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T5.4, T4.4

### Bouton Leave dans le header de conversation

Dans `chat_header()`, ajouter à droite un bouton discret :
```
[⬅ Leave room]   (ghost_destructive style)
```

Clic → `Message::ChatLeaveRoom(room_id)` → déclenche `PendingOp::LeaveRoom`.

### Modal de confirmation

```
┌─ Leave room? ──────────────────────────────┐
│ You will no longer receive messages        │
│ from "salon-pgp". This cannot be undone    │
│ (you would need a new invite to rejoin).   │
│                                            │
│              [Cancel]  [Leave room]        │
└────────────────────────────────────────────┘
```

- Bouton "Leave room" → `Message::ChatLeaveRoomConfirmed(room_id)` (destructive style)
- i18n : `s.chat_leave_room()`, `s.chat_leave_confirm_title()`, `s.chat_leave_confirm_body()`, etc.

**Commit** : `feat(chat/ui): leave room button and confirmation modal`

---

## T5.7 — Modals création / rejoindre / partager

**Complexité** : M
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T5.3

### Fichier : `src/ui/chat_modals.rs`

```rust
pub fn create_room_modal(app: &App) -> Element<Message>;
pub fn join_room_modal(app: &App) -> Element<Message>;
pub fn share_room_modal(room: &Room, join_code: &str) -> Element<Message>;
```

**PendingOp** : ajouter variants si nécessaire :
```rust
PendingOp::CreateRoom { name_draft: String }
PendingOp::JoinRoom { code_draft: String }
PendingOp::ShareRoom { room_id: String, join_code: String }
```

**Commit** : `feat(chat/ui): create/join/share room modals`

---

## T5.8 — Strings i18n chat

**Complexité** : M
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T5.1 (liste des strings définie dans axe5-ui-spec.md)

### Méthodes à ajouter dans le trait `Strings`

Basé sur la liste produite par T5.1. Exemples :

```rust
fn nav_section_chat(&self) -> &'static str;     // "CHAT" / "COMMUNICATIONS"
fn nav_chat_rooms(&self) -> &'static str;        // "Rooms" / "Salons"
fn chat_no_rooms(&self) -> &'static str;         // "No conversations yet..."
fn chat_create_room(&self) -> &'static str;      // "Create room" / "Créer un salon"
fn chat_join_room(&self) -> &'static str;        // "Join room" / "Rejoindre"
fn chat_room_name(&self) -> &'static str;        // "Room name" / "Nom du salon"
fn chat_join_code(&self) -> &'static str;        // "Join code" / "Code d'invitation"
fn chat_copy_invite(&self) -> &'static str;      // "Copy invite" / "Copier l'invitation"
fn chat_send(&self) -> &'static str;             // "Send" / "Envoyer"
fn chat_type_message(&self) -> &'static str;     // "Type a message..." / "Écrire un message..."
fn chat_disconnected(&self) -> &'static str;     // "Disconnected — reconnecting..."
fn chat_decrypt_failed(&self) -> &'static str;   // "Could not decrypt this message"
fn chat_ack_received(&self) -> &'static str;     // "Received" / "Reçu"
fn chat_ack_pending(&self) -> &'static str;      // "Pending" / "En attente"
fn chat_leave_room(&self) -> &'static str;        // "Leave room" / "Quitter le salon"
fn chat_leave_confirm_title(&self) -> &'static str; // "Leave room?"
fn chat_leave_confirm_body(&self) -> &'static str;  // "You will no longer receive messages..."
fn chat_choose_identity(&self) -> &'static str;   // "Choose your identity"
fn chat_choose_identity_hint(&self) -> &'static str; // "You have multiple private keys..."
fn chat_enter_room(&self) -> &'static str;        // "Enter room" / "Rejoindre"
// + variantes USSR pour les textes "flavor"
```

**Commit** : `feat(i18n): add chat strings to Strings trait (EN + FR)`

---

## Fichiers créés / modifiés

```
src/ui/mod.rs            (+ cluster CHAT, + View::ChatList, View::ChatRoom)
src/ui/chat.rs           (nouveau)
src/ui/chat_room.rs      (nouveau)
src/ui/chat_modals.rs    (nouveau)
src/i18n/mod.rs          (+ méthodes chat)
src/i18n/english.rs      (+ implémentations EN)
src/i18n/french.rs       (+ implémentations FR)
src/app/mod.rs           (+ PendingOp variants si nécessaire)
.claude/plans/v0.6.0/axe5-ui-spec.md  (livrable T5.1)
```

## Critères d'acceptation

- [ ] `cargo build` ✓
- [ ] Cluster CHAT visible dans sidebar (Catppuccin + USSR)
- [ ] Room list s'affiche avec présence ●/○
- [ ] Bulles correctement alignées gauche/droite
- [ ] Barre de saisie envoie un message sur Enter
- [ ] Auto-scroll vers le bas à chaque nouveau message
- [ ] Modals créer/rejoindre/partager fonctionnels
- [ ] Zero string français hardcodé dans `src/ui/chat*.rs`
