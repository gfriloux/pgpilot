# Axe 7 — Sécurité & robustesse

## Objectif

Audit de sécurité ciblé sur les nouveaux modules chat, hardening de la couche TLS/MQTT,
validation des invariants cryptographiques, et revue de code finale avant les tests.

---

## T7.1 — Audit sécurité des modules chat

**Complexité** : M
**Agent** : `voltagent-infra:security-engineer`
**Dépendances** : T6.3 (tous les modules chat implémentés)

### Ce qui est à faire

Lire et auditer :
- `src/chat/mqtt.rs`
- `src/chat/crypto.rs`
- `src/chat/presence.rs`
- `src/chat/ack.rs`
- `src/chat/rooms.rs`
- `src/app/chat.rs`

Pour chaque fichier, vérifier :

**1. Validation des entrées**

- `from_join_code` : rejette les codes malformés, truncated, avec relay non-MQTT ?
- `parse_presence_event` : rejette fingerprints invalides (longueur, charset) ?
- `decrypt_message` : que se passe-t-il avec un payload PGP vide ou malformé ?
- `rooms.yaml` : que se passe-t-il si un fichier malveillant est placé à ce chemin ?
- Taille max des messages : un message de 100 MB est-il rejeté avant déchiffrement ?

**2. Invariants cryptographiques**

- `verify_sender` est-il appelé AVANT d'afficher un message déchiffré ?
- Peut-on recevoir un message "de" un fingerprint non membre de la room ? (filtrage)
- Les ACK sont-ils vérifiés avant mise à jour des indicateurs ?
- Les payloads de présence sont-ils vérifiés avant mise à jour `self.presence` ?
- La signature couvre-t-elle bien `{id}|{sender}|{ts}|{payload}` (pas juste le payload) ?

**3. Metadata leakage**

- Le fingerprint complet apparaît-il dans les topics MQTT ? (doit être tronqué)
- Le nom du salon apparaît-il dans les topics ? (doit être opaque — SHA256)
- Les destinataires sont-ils inclus dans le WireMessage ? (recommandation : non)

**4. Gestion mémoire**

- Les messages en RAM sont-ils bornés ? (ex : garder les 500 derniers par room max)
- Les clés GPG ne sont-elles pas mises en cache en mémoire longtemps ?
- Panic possible sur unwrap() dans les handlers async ?

**5. TLS**

- Le client MQTT valide-t-il le certificat du broker ?
- Les connexions non-TLS (port 1883) sont-elles rejetées en production ?
- Comment configurer un broker auto-hébergé avec certificat custom ?

**Output** : liste de vulnérabilités ou points de vigilance, classés Critical/High/Medium/Low

**Commit** : aucun — livrable = rapport d'audit

---

## T7.2 — Corrections sécurité

**Complexité** : M (dépend du rapport T7.1)
**Agent** : `voltagent-lang:rust-engineer`
**Dépendances** : T7.1

### Ce qui est à faire

Implémenter toutes les corrections identifiées Critical et High dans le rapport T7.1.

Points attendus à corriger a minima :

**Validation taille message** :
```rust
const MAX_WIRE_MSG_BYTES: usize = 512 * 1024; // 512 KiB

fn parse_wire_message(payload: &[u8]) -> Result<WireMessage, ChatError> {
    if payload.len() > MAX_WIRE_MSG_BYTES {
        return Err(ChatError::MessageTooLarge);
    }
    // ...
}
```

**Borne des messages RAM** :
```rust
const MAX_MESSAGES_PER_ROOM: usize = 500;

fn store_message(&mut self, room_id: &str, msg: ChatMessage) {
    let msgs = self.chat_messages.entry(room_id.to_string()).or_default();
    if msgs.len() >= MAX_MESSAGES_PER_ROOM {
        msgs.remove(0); // ou VecDeque
    }
    msgs.push(msg);
}
```

**Filtrage sender non-membre** :
```rust
fn on_chat_received(&mut self, room_id: String, ...) -> Task<Message> {
    let Some(room) = self.rooms.iter().find(|r| r.id == room_id) else { return Task::none() };
    if !room.participants.contains(&sender_fp) {
        return Task::none(); // ignore silencieusement
    }
    // ...
}
```

**Commit** : `security(chat): message size limit, RAM bound, sender membership check`

---

## T7.3 — Revue de code finale

**Complexité** : M
**Agent** : `voltagent-qa-sec:code-reviewer`
**Dépendances** : T7.2

### Ce qui est à vérifier

1. Tous les `unwrap()` dans les handlers async → remplacés par `?` ou `match`
2. Pas de `Command::new("gpg")` dans `src/chat/` (utiliser `gpg_command`)
3. Pas de paths absolus hardcodés
4. Cohérence des noms : `room_id`, `fp`, `msg_id` utilisés de façon cohérente
5. Pas de string de statut inférée (`"ok"`, `"error"`) → utiliser `ChatError` enum
6. `cargo clippy -- -D warnings` serait vert
7. Longueur des fonctions : aucun handler > 40 lignes (router uniquement dans `update()`)

**Output** : liste de points ou "✓ RAS"

**Commit** : corrections mineures si nécessaire

---

## T7.5 — Review et mise à jour THREAT_MODEL.md

**Complexité** : M
**Agent** : `voltagent-infra:security-engineer`
**Dépendances** : T7.2 (implémentation sécurité stabilisée)

### Contexte

`THREAT_MODEL.md` existe à la racine du projet. Il a été rédigé avant le chat. L'ajout du
chat (MQTT, messages chiffrés, présence signée, rooms.yaml) introduit de nouveaux actifs,
nouvelles menaces, et nouvelles contre-mesures à documenter.

### Ce qui est à faire

1. Lire `THREAT_MODEL.md` dans son état actuel
2. Identifier les sections à mettre à jour ou créer pour couvrir le chat :

**Nouveaux actifs** :
- `rooms.yaml` (liste des rooms et participants — pas de contenu message)
- Messages en RAM (éphémères — disparaissent à la fermeture)
- Connexion MQTT (transport chiffré TLS)
- Identité dans une room (`my_identity` fingerprint)

**Nouvelles menaces à analyser** :
- Un attaquant connaît le room_id → peut-il lire les messages ? (non — chiffrement PGP)
- Un attaquant injecte un faux message avec un sender usurpé → contré par `verify_sender`
- Un attaquant publie une présence "online" pour un fingerprint tiers → contré par signature
- Broker MQTT compromis → voit les topics et timestamps, pas le contenu
- Broker MQTT down → messages perdus (pas de persistance offline — limitation connue, documentée)
- `rooms.yaml` lu par un attaquant → révèle les participants, pas les messages

**Nouvelles contre-mesures à documenter** :
- Chiffrement E2E OpenPGP (sequoia) — clés privées restent locales
- Signature de chaque message, présence, ACK — prouve l'identité
- `verify_sender` — anti-usurpation
- Taille max message 512 KiB — anti-DoS broker
- Messages éphémères RAM — pas de journal sur disque
- SHA256(room_id) dans les topics — pas d'exposition du room_id au broker

3. Rédiger les mises à jour directement dans `THREAT_MODEL.md`

**Commit** : `security: update THREAT_MODEL.md for PGP chat feature`

---

## T7.4 — Refactorisation

**Complexité** : S
**Agent** : `voltagent-dev-exp:refactoring-specialist`
**Dépendances** : T7.3

### Ce qui est à faire

Après revue, passer sur les modules chat avec un œil refactorisation :

1. Factoriser les patterns répétés entre `publish_online` / `publish_offline` / `publish_ack`
   (tous construisent une payload signée JSON → helper `sign_json_payload`)
2. Vérifier que `src/chat/crypto.rs` ne duplique pas de logique déjà dans `src/gpg/keyring.rs`
   (si oui, extraire en fonctions communes dans `src/gpg/mod.rs`)
3. `src/app/chat.rs` : si un handler dépasse 30 lignes, le découper en helpers privés
4. S'assurer que `src/chat/mod.rs` ne fait que des re-exports (pas de logique inline)

**Commit** : `refactor(chat): extract sign_json_payload helper, reduce handler size`

---

## Fichiers modifiés

```
src/chat/mqtt.rs         (corrections TLS, taille)
src/chat/crypto.rs       (corrections invariants)
src/chat/presence.rs     (corrections validation)
src/chat/ack.rs          (corrections validation)
src/app/chat.rs          (corrections handlers, refacto)
src/chat/mod.rs          (refacto exports)
```

## Critères d'acceptation

- [ ] Rapport d'audit T7.1 produit
- [ ] Toutes les vulnérabilités Critical et High corrigées
- [ ] Taille max message enforced (512 KiB)
- [ ] Messages RAM bornés à 500 par room
- [ ] Sender non-membre de la room → message ignoré silencieusement
- [ ] `cargo clippy -- -D warnings` ✓
- [ ] Aucun `unwrap()` non justifié dans les handlers async
