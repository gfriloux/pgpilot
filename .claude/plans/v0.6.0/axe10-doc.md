# Axe 10 — Documentation

## Objectif

Mettre à jour toute la documentation pour refléter v0.6.0 : CLAUDE.md (architecture chat,
nouveaux modules, rooms.yaml), mdbook (guide utilisateur chat), et les chaînes i18n du chat.

---

## T10.1 — Mise à jour CLAUDE.md

**Complexité** : M
**Agent** : `voltagent-biz:technical-writer`
**Dépendances** : T7.4 (implémentation chat stabilisée)

### Sections à ajouter / mettre à jour dans `CLAUDE.md`

**1. Module layout** — ajouter `src/chat/` :
```
src/
├── chat/
│   ├── mod.rs      — re-exports, ChatMessage, ChatError
│   ├── rooms.rs    — Room struct, load/save rooms.yaml, join code
│   ├── mqtt.rs     — MqttClient, connexion TLS, subscribe/publish, LWT
│   ├── crypto.rs   — encrypt_for_room, decrypt_message, verify_sender
│   └── presence.rs — PresenceStatus, publish_online/offline, parse_presence_event
```

**2. Nouvelle section "Chat chiffré PGP"** couvrant :
- Architecture générale (transport MQTT, messages éphémères, rooms persistées)
- `rooms.yaml` : chemin, format, ce qui est stocké / pas stocké
- Format `WireMessage` (id, sender, ts, payload, signature)
- Topics MQTT : `pgpilot/chat/{SHA256(room_id)[0..16]}`, `pgpilot/presence/{fp[0..16]}`, `pgpilot/ack/{msg_id[0..16]}`
- Invariant sécurité : `verify_sender` appelé avant tout affichage
- Borne messages RAM : 500 messages max par room
- Taille max message : 512 KiB

**3. Nouveaux `View` variants** dans "State flow" :
```
View::ChatList          — liste des rooms
View::ChatRoom(String)  — room_id actif
```

**4. Handlers chat** dans "update() structure" :
```
src/app/chat.rs — on_chat_room_create, on_chat_room_join, on_chat_send,
                  on_chat_received, on_mqtt_event, on_presence_updated,
                  on_chat_ack_received
```

**5. Section "Presence & ACK"** :
- LWT automatique à la déconnexion
- Signature des payloads de présence et ACK
- Filtrage silencieux des signatures invalides

**6. Mise à jour "Config"** :
- `rooms.yaml` créé dans `~/.config/pgpilot/`
- Pas de messages loggés (éphémères RAM)

**7. Mise à jour Roadmap** :
- Marquer items 26–28 (dashboard, compression, backup) comme `⏳`
- Ajouter item v0.6.0 : i18n, chat PGP, expiry audit, revocation manager

**Commit** : `docs(claude): update CLAUDE.md with chat architecture, rooms.yaml, MQTT topics`

---

## T10.2 — Guide utilisateur chat (mdbook)

**Complexité** : M
**Agent** : `voltagent-biz:technical-writer`
**Dépendances** : T10.1

### Fichier : `book/src/7-chat.md`

Sections :
1. **Introduction** — pourquoi un chat PGP ? Modèle de sécurité en une phrase.
2. **Prérequis** — avoir la clé publique des contacts dans le keyring (import depuis keyserver ou fichier)
3. **Créer un salon** — étapes illustrées (nom local, sélection contacts, bouton Créer)
4. **Partager l'invitation** — copier le join code, le transmettre hors-bande (email, Signal, en main propre)
5. **Rejoindre un salon** — coller le join code
6. **Envoyer un message** — composer, Enter pour envoyer
7. **Indicateurs de présence** — ● en ligne / ○ hors ligne
8. **Accusés de réception** — ✓ reçu / ⏳ hors ligne
9. **Sécurité** :
   - Messages éphémères : fermer PGPilot = messages perdus (intentionnel)
   - Le serveur relay ne voit que des blobs chiffrés
   - L'identité est prouvée par signature GPG, pas par mot de passe
   - Changer de broker MQTT (paramétrable)
10. **Limitations** :
    - Pas de persistance offline (messages manqués si hors ligne)
    - Broker public `test.mosquitto.org` = pas de SLA, usage test uniquement
    - Prévoir auto-hébergement MQTT pour usage production

Mettre à jour `book/src/SUMMARY.md` pour inclure `7-chat.md`.

**Commit** : `docs(book): add PGP chat user guide`

---

## T10.3 — Chapitre sécurité mdbook (THREAT_MODEL)

**Complexité** : M
**Agent** : `voltagent-biz:technical-writer`
**Dépendances** : T7.5 (THREAT_MODEL.md mis à jour par le security-engineer)

### Contexte

Le livre mdbook contient déjà une page `book/src/9-security.md` qui référence `THREAT_MODEL.md`.
Elle doit être mise à jour pour inclure la section chat.

### Ce qui est à faire

1. Lire `THREAT_MODEL.md` mis à jour (T7.5)
2. Lire `book/src/9-security.md` existant
3. Ajouter une section **"PGP Chat — Security Model"** dans `9-security.md` :

```markdown
## PGP Chat — Security Model

### What the relay sees
The MQTT broker sees only:
- An opaque topic (`SHA256(room_id)` truncated to 16 chars)
- Encrypted blobs (OpenPGP — content unreadable without private key)
- Sender fingerprint and timestamp (metadata)
- Presence status (online/offline) per fingerprint

The broker cannot read message content, sender identity beyond fingerprint,
or room membership.

### Identity proof
Every message, presence announcement, and ACK is signed with the sender's
GPG private key. PGPilot verifies signatures before displaying any content.
Spoofing a fingerprint requires the victim's private key.

### Ephemeral messages
Messages exist only in RAM. Closing PGPilot deletes them permanently.
No message log is written to disk. Only `rooms.yaml` persists (room IDs
and participant fingerprints — no message content).

### Known limitations
- **No offline delivery**: messages sent while you are offline are lost.
  Plan: NATS JetStream in a future version.
- **Metadata visible**: the broker can observe which fingerprints are online
  and when, even if it cannot read messages.
- **Public broker (MVP)**: `test.mosquitto.org` is a public test broker
  with no SLA. For production use, self-host a private MQTT broker.
```

4. Mettre à jour les liens depuis `book/src/SUMMARY.md` si nécessaire

**Commit** : `docs(book): update security chapter with PGP chat threat model`

---

## T10.4 — Validation documentation

**Complexité** : S
**Agent** : `voltagent-biz:technical-writer`
**Dépendances** : T10.1, T10.2, T10.3

1. `mdbook build book` — vérifier zéro erreur
2. Vérifier tous les liens internes du nouveau chapitre
3. Cohérence terminologique : "room" / "salon" selon la langue, "join code" / "code d'invitation"
4. Vérifier que CLAUDE.md ne contient plus de références aux anciennes numéros de lignes
   (les numéros de lignes bougent — utiliser des noms de fonctions comme ancres)

**Commit** : `docs: fix mdbook links, terminology consistency in v0.6.0 docs`

---

## Fichiers modifiés

```
CLAUDE.md                       (sections chat, rooms.yaml, topics MQTT)
THREAT_MODEL.md                 (+ section chat — T7.5)
book/src/SUMMARY.md             (+ entrée 7-chat.md)
book/src/7-chat.md              (nouveau)
book/src/9-security.md          (+ section "PGP Chat — Security Model")
```

## Critères d'acceptation

- [ ] `mdbook build book` ✓ (zéro erreur ni warning)
- [ ] CLAUDE.md contient la section "Chat chiffré PGP" avec tous les invariants
- [ ] `book/src/7-chat.md` existe et couvre les 10 sections
- [ ] `book/src/9-security.md` contient la section chat threat model
- [ ] `THREAT_MODEL.md` mis à jour avec actifs, menaces et contre-mesures du chat
- [ ] Terminologie cohérente EN/FR dans le guide
- [ ] Roadmap CLAUDE.md à jour
