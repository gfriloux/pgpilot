# Plan de développement — pgpilot v0.8.0

**Version :** 0.8.0  
**Status :** Planification complète  
**Date :** 2026-05-12  
**Chef de projet :** @agent-voltagent-biz:project-manager

---

## Vue d'ensemble

La **v0.8.0** marque la plus grande refonte de pgpilot depuis ses débuts : **migration du framework iced vers Tauri** pour déplocker le potentiel créatif de l'interface utilisateur et atteindre un niveau de qualité attendu par les utilisateurs grand public.

### Raison centrale

iced (framework Elm-like basé sur OpenGL) limite fortement les capacités d'expression artistique et visuelle. Les sous-agents design et frontend se heurtent régulièrement aux contraintes de composition widget, d'animations, de typographie avancée.

**Tauri** (wrapper léger autour d'une WebView système + backend Rust) offre une liberté créative totale. Le frontend devient un véritable site web (HTML/CSS/JS modern), tandis que toute la logique métier (GPG, MQTT, crypto) reste en Rust via l'IPC Tauri.

### Trois piliers

| Pilier | Scope | Impact |
|--------|-------|--------|
| **Partie 1 : Migration Tauri** | Framework replacement, IPC, WebView sécurité | Nouvelle UI + backend refactorisé |
| **Partie 2 : Validation utilisateur** | Beta testing, feedback UX/graphique | Corrections itératives, polishing |
| **Partie 3 : Doc Astro/Starlight** | Remigration mdbook → Astro + Starlight | Doc bilingue, SEO, dark mode natif |

---

## Découpage en phases (18 semaines)

### Tableau résumé

| Phase | Nom | Durée | Pilier | Dépendances | DoD |
|-------|-----|-------|--------|------------|-----|
| 1 | Dérisking Tauri | 2 sem | 1 | Aucune | PoC Tauri + 1 écran fonctionnel | ✅ DONE |
| 2 | Design system & tokens | 1 sem | 1 | Phase 1 ✅ | Design tokens JSON, composants web storybook | ✅ DONE |
| 3 | Frontend architecture | 2 sem | 1 | Phase 2 ✅ | Scaffold React + TypeScript, routing | ✅ DONE |
| 4 | Thème Catppuccin (web) | 2 sem | 1 | Phase 3 ✅ | CSS variables, live preview | ✅ DONE (fusionné 4+5) |
| 5 | Thème USSR (web) | 2 sem | 1 | Phase 4 ✅ | Bannières SVG/PNG intégrées, assets | ✅ DONE (fusionné 4+5) |
| 6 | IPC backend → frontend | 2 sem | 1 | Phase 5 ✅ | Type-safe IPC bindings, handlers | ✅ DONE |
| 7 | Porte clés (master feature) | 2 sem | 1 | Phase 6 ✅ | Key list, detail, CRUD | ✅ DONE |
| 8 | Audit sécurité Tauri | 1 sem | 1 | Phase 7 ✅ | Rapport audit, fixes critiques | ✅ DONE |
| 9 | Opérations (encrypt/sign/verify) | 2 sem | 1 | Phase 8 ✅ | File drag&drop, formats, workflows | ✅ DONE |
| 10 | Chat chiffré (migration) | 1 sem | 1 | Phase 9 ✅ | UI chat responsive, MQTT intégré | ✅ DONE |
| 11 | QA & tests E2E | 2 sem | 1,2 | Phase 10 ✅ | Coverage > 80%, smoke tests | ✅ DONE |
| 12 | Validation utilisateur (beta) | 3 sem | 2 | Phase 11 ✅ | Feedback logs, liste des fixes |
| 13 | Corrections post-beta | 2 sem | 2 | Phase 12 ✅ | Hotfixes, perf tuning |
| 14 | Astro + Starlight setup | 1 sem | 3 | Phase 1 ✅ | Projet scaffold, structure bilingue |
| 15 | Migration doc mdbook → Astro | 2 sem | 3 | Phase 14 ✅ | Contenu migré, re-rendered |
| 16 | Intégration screenshots & démo | 1 sem | 3 | Phase 13 ✅ | Doc avec illustrations |
| 17 | Déploiement (GitHub Pages) | 1 sem | 3 | Phase 16 ✅ | Domaine custom, auto-deploy CI |
| 18 | Release & communication | 1 sem | 1,2,3 | Phase 13 + 17 ✅ | Binaires signés, notes de version |

**Durée totale :** 18 semaines  
**Jalons critiques :**
- Fin Phase 7 (sem 11) : application fonctionnelle avec clés (MVP #1)
- Fin Phase 11 (sem 16) : application stable, tests passants
- Fin Phase 12 (sem 19) : beta feedback intégré
- Fin Phase 18 (sem 20) : release publique v0.8.0

---

## Critères de succès (DoD par phase)

### Phase 1 : Dérisking Tauri
- [ ] PoC Tauri v2 scaffold du projet (`npm create tauri-app@latest`)
- [ ] Écran d'accueil affiché en Tauri
- [ ] Commande IPC simple : `get_version()` → affichage version backend Rust
- [ ] Sécurité par défaut vérifiée : CSP headers, WebView sandbox, allowlist générique

### Phase 2 : Design system
- [ ] Fichier `tokens.json` (couleurs Catppuccin + USSR, typographie, espacements)
- [ ] Storybook local (isolé des commandes IPC) avec 10 composants de base
- [ ] Palette de couleurs exportable (CSS variables, Tailwind config)

### Phase 3 : Frontend architecture
- [ ] Vite + React 18 + TypeScript strictement configuré
- [ ] Routing via React Router (ou équivalent)
- [ ] Typage IPC : `tauri-specta` ou équivalent (auto-générération types Rust → TS)
- [ ] Structure dossiers stabilisée : `components/`, `pages/`, `hooks/`, `types/`

### Phase 4 : Catppuccin web
- [ ] Palette CSS complète (26 fonctions de couleur en TS)
- [ ] Tous les composants styled Catppuccin
- [ ] Preview temps-réel du thème sélectionné

### Phase 5 : USSR web
- [ ] Bannières PNG intégrées (12 assets)
- [ ] Badges SVG circulaires (keyserver, YubiKey, trust) générés dynamiquement
- [ ] Composants styled USSR complets
- [ ] Sélecteur thème fonctionnel

### Phase 6 : IPC Tauri
- [ ] Bindings TypeScript auto-générés depuis Rust via `tauri-specta`
- [ ] 5+ commandes IPC testées (list_keys, export_key, etc.)
- [ ] Gestion d'erreurs type-safe (Result<T, String>)
- [ ] Documentation API IPC interne

### Phase 7 : Porte-clés (master)
- [ ] Master-detail layout (list 320px, detail flexible)
- [ ] Affichage clés publiques + privées
- [ ] Opérations CRUD : create, import, delete, export
- [ ] Trust level picker
- [ ] Subkeys panel avec renew/rotate
- [ ] YubiKey migration workflow
- [ ] Tests E2E Playwright : 20+ scénarios

### Phase 8 : Audit sécurité Tauri
- [ ] Rapport d'audit complet (IPC, WebView, permissions)
- [ ] Fixes appliqués pour toutes les vulnérabilités CRITICAL
- [ ] Re-test validé par security-auditor

### Phase 9 : Opérations
- [ ] File encryption (multi-recipient, trust warning, format toggle)
- [ ] File signing (key picker, .sig output)
- [ ] Signature verification (5-state outcome display)
- [ ] Drag & drop file support
- [ ] Tests E2E : 15+ scénarios

### Phase 10 : Chat chiffré
- [ ] Migration UI chat (room list, room view, new room, join room)
- [ ] MQTT transport intégré (TLS, auto-reconnect)
- [ ] Encryption/decryption via backend Rust
- [ ] Présence & ACKs fonctionnels
- [ ] Tests E2E : 10+ scénarios

### Phase 11 : QA & tests E2E
- [ ] Coverage > 80% (unit + E2E Playwright)
- [ ] Tous les tests passants sur Linux (X11 + Wayland)
- [ ] Performance : démarrage < 2 s, transitions fluides
- [ ] Accessibility : audit WCAG AA, min 85% score

### Phase 12 : Validation utilisateur (beta)
- [ ] Formulaire feedback structuré (UX, graphique, perf, bugs)
- [ ] 10+ utilisateurs beta, 2 semaines de tests
- [ ] Logs de feedback triés par sévérité/catégorie
- [ ] Prise de décision : items à inclure en 0.8.0 vs v0.9.0

### Phase 13 : Corrections post-beta
- [ ] Hotfixes critiques implémentés
- [ ] Perf tuning validé (démarrage, transitions, IPC latency)
- [ ] Binary release candidate signée

### Phase 14 : Astro + Starlight setup
- [ ] Projet Astro scaffold avec Starlight preset
- [ ] Structure i18n (EN/FR) fonctionnelle
- [ ] Local build et preview OK

### Phase 15 : Migration doc mdbook
- [ ] Contenu 100% migré (book/src → docs/src)
- [ ] Markdown converti pour Starlight (frontmatter, images)
- [ ] Build static sans erreurs

### Phase 16 : Screenshots & démo
- [ ] 15+ screenshots intégrées dans la doc
- [ ] Liens vers repo, release, bugtracker
- [ ] Page "Getting Started" interactive

### Phase 17 : Déploiement
- [ ] GitHub Pages configure via Actions
- [ ] Domaine custom CNAME pointant vers doc
- [ ] Auto-deploy main → pages sur commit
- [ ] HTTPS, sitemap, SEO basics

### Phase 18 : Release
- [ ] Binaires Tauri compilés (x64, possiblement ARM)
- [ ] Signatures cryptographiques (cosign ou équivalent)
- [ ] RELEASE_NOTES.md généré
- [ ] GitHub Release publié
- [ ] Blog post / annonce réseaux sociaux

---

## Risques et mitigations

### Risques techniques

| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|-----------|
| **WebView rendering instabilité** | Moyenne | Critique | Phase 1 : test PoC cross-platform ; fallback : iced remains |
| **IPC perf overhead** | Basse | Moyen | Phase 6 : benchmark 100x round-trips ; optimize si > 10ms |
| **Frontend > 50 dépendances npm** | Moyenne | Moyen | Phase 3 : audit dépendances ; deny list CVE-tracker |
| **Tauri v2 breaking changes** | Basse | Critique | Phase 1 : pin version ; subscribe GitHub releases |
| **Compilations croisées (ARM) problématiques** | Moyenne | Moyen | Phase 18 : test sur RPi / ARM VM ; skip ARM si trop coûteux |

### Risques ressources

| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|-----------|
| **Agents frontend pas assez disponibles** | Basse | Critique | Phase 1 : pré-valider ressources agents ; plan B : 1 agent frontend senior |
| **Refonte design prend 2× prévu** | Moyenne | Moyen | Phase 2–5 : réunion d'alignment design semaine 2 ; réduire scope USSR si nécessaire |
| **Beta feedback massif (50+ items)** | Basse | Moyen | Phase 12 : appel de décision collectif ; rediriger non-critiques vers v0.9.0 |

### Risques organisationnels

| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|-----------|
| **Scope creep (nouvelles fonctionnalités)** | Moyenne | Critique | Verrouiller scope Phase 1 ; utiliser innovation.md pour v0.9.0 |
| **Timeline glisse 2+ semaines** | Moyenne | Moyen | Réunions bihebd ; buffer 2 sem en Phase 13 |
| **Utilisateur non satisfait du résultat graphique** | Basse | Critique | Phase 2 : design review avec utilisateur ; mockups approuvés avant code |

---

## Communication & gouvernance

### Réunions régulières

- **Lundi 10h30 (45 min)** : Standup chef de projet + lead designer + lead frontend
- **Jeudi 14h (1h)** : Backlog grooming + risk assessment
- **Fin de phase (variable)** : Review + décision phase suivante

### Rapports

- **Status quo :** toutes les 2 semaines (mail à utilisateur)
- **Risques actualisés :** à chaque phase
- **Decisions log :** privé, archivé dans `.claude/plans/v0.8.0/decisions.md`

### Escalade

- **Phase bloquée 3+ jours :** PM → utilisateur + lead technique
- **Audit sécurité critique (Phase 8) :** security-auditor → PM → utilisateur

---

## Budget & ressources

### FTE estimé par phase

| Phase | Agents principaux | FTE | Durée |
|-------|-------------------|-----|-------|
| 1 | PM + rust-engineer | 2 | 2 sem |
| 2 | ui-designer + frontend-dev | 2 | 1 sem |
| 3 | frontend-dev + fullstack-dev | 2.5 | 2 sem |
| 4–5 | frontend-dev | 2 | 4 sem |
| 6 | fullstack-dev + rust-engineer | 2.5 | 2 sem |
| 7 | frontend-dev + test-automator | 3 | 2 sem |
| 8 | security-auditor + rust-engineer | 2 | 1 sem |
| 9–10 | frontend-dev + rust-engineer | 2.5 | 3 sem |
| 11 | test-automator + accessibility-tester | 2.5 | 2 sem |
| 12 | ui-ux-tester | 0.5 | 3 sem |
| 13 | rust-engineer + frontend-dev | 2 | 2 sem |
| 14–17 | technical-writer | 1 | 5 sem |
| 18 | PM + release-engineer | 1 | 1 sem |

**Total estimé :** ~35 FTE / 18 semaines = 1.9 FTE moyennes

---

## Architecture décisions (confirmées)

Voir `.claude/plans/v0.8.0/tech-choices.md` pour les justifications.

Toutes les décisions sont confirmées par le projet **sshive** (`../sshive`), app Tauri v2 + Svelte 5 fonctionnelle sur le même environnement NixOS/Wayland.

| Sujet | Décision |
|-------|----------|
| Framework frontend | **React 18 + TypeScript strict** |
| Bundler | **Vite 6**, port 1421 |
| CSS themes | **Variables CSS dans `theme.css`** (pas de Tailwind) |
| IPC | **`invoke` direct** (pas de tauri-specta) |
| State | **Zustand** |
| Routing | **React Router v6** |
| Tests | **Playwright + `VITE_MOCK=true`** |
| Nix deps | `webkitgtk_4_1 + gtk3 + libsoup_3 + dbus` (de sshive) |
| Doc interactive | **Build mock React embarqué dans Astro** |

---

## Livérables

### Par phase

- **Phase 1–11 (app Tauri)** : Dépôt GitHub branche `dev/tauri`, CI GitHub Actions, release candidate binaire
- **Phase 12 (beta feedback)** : Logs structurés, décision items, plan post-beta
- **Phase 13 (corrections)** : Hotfixes commitées, binaire release finale
- **Phase 14–17 (doc Astro)** : Site doc déployé en GitHub Pages, CNAME configuré
- **Phase 18 (release)** : v0.8.0 tag, GitHub Release, binaires signés, notes de version publiques

### En dehors du plan (post-release)

- Blog post technique (archi Tauri)
- Vidéo démo 3 min (thèmes, workflows clés)
- Annonce réseaux sociaux (Mastodon, Twitter, etc.)

---

## Next steps

1. **Valider cette planification :** utilisateur + PM (cette semaine)
2. **Lancer Phase 1 (dérisking Tauri) :** semaine prochaine, agents PM + rust-engineer
3. **Créer le repository branché :** dépôt séparé ou branche dev/tauri du dépôt principal ?
4. **Confirmer le framework frontend :** Phase 1, réunion discovery design

---

## Annexes

- `.claude/plans/v0.8.0/phases.md` — Détail exécution par phase
- `.claude/plans/v0.8.0/tech-choices.md` — Justifications tech
- `.claude/plans/v0.8.0/security.md` — Audit Tauri + WebView
- `.claude/plans/v0.8.0/agents.md` — Allocation agents détaillée
- `.claude/plans/v0.8.0/innovations.md` — Opportunités créatives
