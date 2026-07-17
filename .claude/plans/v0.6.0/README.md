# Plan v0.6.0 — pgpilot

## Vue d'ensemble

Deux grandes familles, dix axes. Les axes i18n et utilitaires (1, 9, 10) sont réalisables immédiatement. Le chat (axes 2–8) démarre après validation et commit de l'axe 1.

### Famille A — i18n + utilitaires + UI harmonisation (avant commit intermédiaire)

- [Axe 0 — Harmonisation UI card layout](axe0-ui-harmonisation.md) — audit, helpers `page_layout/card_*`, migration des vues
- [Axe 1 — Corrections i18n](axe1-i18n.md) — audit, implémentation, tests, revue
- [Axe 9 — Expiry audit + Revocation manager](axe9-utilitaires.md) — deux petites features S indépendantes

### Famille B — Chat chiffré PGP (après commit A)

- [Axe 2 — Architecture chat](axe2-chat-architecture.md) — spec technique complète avant tout code
- [Axe 3 — Transport MQTT](axe3-chat-transport.md) — couche réseau, connexion, reconnexion
- [Axe 4 — Core : rooms & messages](axe4-chat-core.md) — modèle de données, rooms.yaml, chiffrement
- [Axe 5 — UI chat](axe5-chat-ui.md) — vues iced, room list, bulles, barre de saisie
- [Axe 6 — Présence & ACK](axe6-chat-presence-ack.md) — indicateurs ●/○, accusés de réception
- [Axe 7 — Sécurité & robustesse](axe7-chat-securite.md) — TLS, validation signatures, hardening
- [Axe 8 — Tests chat](axe8-chat-tests.md) — unit + intégration pour tous les modules chat

### Transversal

- [Axe 10 — Documentation](axe10-doc.md) — mise à jour CLAUDE.md, mdbook, i18n strings chat

---

## Séquence d'exécution

```
┌─────────────────────────────────────┐
│  FAMILLE A (immédiatement)          │
│                                     │
│  Axe 0 (UI)    ──┐                 │
│  Axe 1 (i18n)  ──┤                 │
│  Axe 9 (utils) ──┤                 │
│                   ▼                 │
│         Commit A + Push             │
│      Validation utilisateur         │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│  FAMILLE B (chat)                   │
│                                     │
│  Axe 2 (architecture)               │
│       │                             │
│       ▼                             │
│  Axe 3 ──► Axe 4 ──► Axe 5         │
│                 │         │         │
│                 └──► Axe 6          │
│                       │             │
│                       ▼             │
│                    Axe 7            │
│                       │             │
│                 ┌─────┴──────┐      │
│              Axe 8        Axe 10    │
│                 │             │     │
│                 └──────┬──────┘     │
│                        ▼            │
│              Commit B + Push        │
│           Validation utilisateur    │
└─────────────────────────────────────┘
```

---

## Dépendances détaillées

```
T0.1 (audit UI — ui-designer)
  └─► T0.2 (spec cards — ui-designer)
        └─► T0.3 (impl helpers + migration — rust-engineer)
              └─► T0.4 (revue visuelle — ui-designer)

T1.1 (audit Explore)  ← parallèle avec T0.x
  └─► T1.2 (impl rust-engineer)
        ├─► T1.3 (tests test-automator)
        └─► T1.4 (revue code-reviewer)
              └─► T1.5 (validation utilisateur) → COMMIT A

T9.1 (expiry audit) ──┐  parallélisables avec Axe 0 et Axe 1
T9.2 (revocation)   ──┘  intégrés au même commit A

T2.1 (arch — architect-reviewer)
T2.2 (API design — api-designer)     ← parallèle avec T2.1
  └─► spec chat validée
        ├─► T3.x (transport MQTT — rust-engineer)
        ├─► T4.x (core rooms/messages — rust-engineer)  ← après T3
        ├─► T5.x (UI — rust-engineer + ui-designer)     ← après T4
        ├─► T6.x (présence + ACK — rust-engineer)       ← après T5
        └─► T7.x (sécurité — security-engineer)         ← après T6
              ├─► T8.x (tests — test-automator)
              └─► T10.x (doc — technical-writer)
                    └─► COMMIT B
```

---

## Agents impliqués

| Agent | Axes | Rôle |
|-------|------|------|
| `Explore` | 1 | Audit exhaustif des strings FR hardcodées |
| `voltagent-lang:rust-engineer` | 0, 1, 3, 4, 5, 6, 9 | Implémentation Rust principale |
| `voltagent-qa-sec:test-automator` | 1, 8 | Écriture des tests unitaires et intégration |
| `voltagent-qa-sec:code-reviewer` | 1, 7 | Revue de code qualité |
| `voltagent-qa-sec:architect-reviewer` | 2 | Architecture modules chat, interfaces |
| `voltagent-core-dev:api-designer` | 2 | Design API interne Message/Task |
| `voltagent-core-dev:ui-designer` | **0**, 5 | **Harmonisation card layout (Famille A)**, UI chat |
| `voltagent-infra:security-engineer` | 7 | Hardening TLS, validation crypto, review THREAT_MODEL.md |
| `voltagent-dev-exp:refactoring-specialist` | 7 | Refactorisation modules chat |
| `voltagent-biz:technical-writer` | 10 | CLAUDE.md, mdbook guide chat + chapitre sécurité |

---

## Critères d'acceptation globaux

### Famille A
- [ ] `cargo build` ✓
- [ ] `cargo clippy -- -D warnings` ✓
- [ ] `cargo fmt -- --config tab_spaces=2` ✓
- [ ] `cargo test` ✓ (incluant tests i18n)
- [ ] Test manuel : Settings → English → zéro texte français visible
- [ ] Test manuel : Settings → French → aucune régression
- [ ] Toutes les vues (sauf MyKeys/PublicKeys) utilisent `page_layout(card_*)` — largeur cohérente
- [ ] Bannière expiry visible si sous-clef expire < 90j
- [ ] Section revocation certificate dans le detail panel

### Famille B
- [ ] `cargo build` ✓
- [ ] `cargo clippy -- -D warnings` ✓
- [ ] `cargo test` ✓ (incluant tests chat)
- [ ] `cargo test -- --ignored` ✓
- [ ] Connexion MQTT établie sur `test.mosquitto.org:8883`
- [ ] Message chiffré envoyé et reçu entre deux instances PGPilot
- [ ] Présence ●/○ correcte en temps réel
- [ ] ACK reçu après déchiffrement réussi
- [ ] `rooms.yaml` persisté et rechargé correctement
- [ ] Fermeture de PGPilot → messages RAM effacés, rooms.yaml intact
- [ ] Identité sélectionnée avant d'entrer dans une room (si plusieurs clefs privées)
- [ ] Bouton "Leave room" fonctionnel avec confirmation
- [ ] `THREAT_MODEL.md` mis à jour avec section chat
- [ ] `book/src/9-security.md` contient la section chat threat model
- [ ] CLAUDE.md à jour (module chat, rooms.yaml, MQTT)
