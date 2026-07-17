# Index — Plan v0.8.0 pgpilot

**Navigation rapide des documents de planification.**

---

## Documents principaux

### 📋 README.md
**Vue d'ensemble, timeline, risques, gouvernance**
- Objectifs de v0.8.0 (3 piliers)
- Tableau résumé phases (18 semaines)
- Critères de succès (DoD) par phase
- Risques et mitigations
- Communication & gouvernance
- Budget & ressources

**Lire en premier pour comprendre le scope global.**

---

### 🔧 phases.md
**Détail exécution : tâches, dépendances, critères de complétion**
- 18 phases détaillées (Phase 1 Dérisking → Phase 18 Release)
- Pour chaque phase :
  - Lead + sous-agents
  - 4–8 tâches (T X.Y) avec durée
  - Critères de complétion (DoD checklist)
  - Dépendances inter-phases (diagram)

**Consulter pour exécution jour-à-jour. Référence pour les agents.**

---

### 🛠️ tech-choices.md
**Justifications techniques : framework, bundler, CSS, IPC, tests, accessibility**
- Framework frontend : React 18 ✅ (vs Vue, Svelte)
- Bundler : Vite ✅ (vs Webpack)
- CSS themes : CSS variables + Tailwind ✅
- Type-safe IPC : tauri-specta ✅
- E2E tests : Playwright ✅
- Accessibility : WCAG AA (axe + manual)
- State management : Zustand ✅
- Performance targets (startup < 2s, IPC < 10ms)

**Lire avant Phase 1 decision gate pour valider choix.**

---

### 🔒 security.md
**Analyse de menace, surface d'attaque, audit Phase 8**
- Contexte de menace (utilisateur grand public, matériel sensible)
- Analyse surface IPC, WebView, permissions, config
- Mitigations : input validation, CSP, permission scoping, secrets management
- Audit checklist Phase 8 (fuzzing, CSP validation, permissions audit)
- ANSSI guides applicabilité
- Known limitations v0.8.0

**Consulter pour Phase 1 (CSP baseline) et Phase 8 (full audit).**

---

### 👥 agents.md
**Allocation agents par phase, dépendances, communication**
- 12 agents disponibles (PM, RS, FE, FULL, DES, SEC-A, etc.)
- Allocation FTE par phase
- Tasks assignées avec durée + livrable
- Dépendances inter-agents (diagram)
- Cadence communication (standups, reporting, escalade)
- Contingency plans

**Consulter PM et leads agents pour planification ressources.**

---

### 💡 innovations.md
**Opportunités créatives (v0.8.0+ et post-release)**
- Notifications natives (Phase 9+)
- System tray icon + quick actions
- Deep links / URL handlers
- Interactive doc playground
- Video tutorials
- Advanced typography
- Post-quantum crypto (v0.9+)
- Cloud backup encryption
- QR code sharing
- Trust network visualization
- Telemetry opt-in
- Mobile companion (future)

**Lire pour inspirer Phase 2 design system et planifier v0.9.**

---

## Sections thématiques

### Par phase

| Phase | Document | Focus |
|-------|----------|-------|
| 1 | phases.md, tech-choices.md | Dérisking, framework choice |
| 2–5 | phases.md, tech-choices.md, innovations.md | Design tokens, themes |
| 6 | phases.md, tech-choices.md | IPC architecture, Rust refactor |
| 7–10 | phases.md, agents.md | Feature implementation |
| 8 | security.md, agents.md | Audit Tauri |
| 11–12 | phases.md | QA, beta testing |
| 13–18 | phases.md, agents.md | Finalization, release, doc |

### Par rôle

| Rôle | Documents |
|------|-----------|
| **PM** | README.md (vue d'ensemble), agents.md (ressources), phases.md (DoD) |
| **UI Designer** | tech-choices.md (design system), innovations.md (creativity), phases.md (2–5) |
| **Frontend Dev** | phases.md, tech-choices.md (React, Vite, CSS), agents.md (tasks) |
| **Rust Engineer** | phases.md, tech-choices.md (tauri-specta), security.md (IPC validation) |
| **Security Auditor** | security.md (audit plan), phases.md (Phase 8) |
| **QA/Test** | phases.md (test tasks), tech-choices.md (Playwright, accessibility) |
| **Tech Writer** | phases.md (14–17), innovations.md (interactive doc) |

---

## Timeline quick reference

```
Week 1–2   → Phase 1 : Dérisking Tauri (PM + RS)
Week 3     → Phase 2 : Design tokens (DES + FE)
Week 4–5   → Phase 3 : Frontend arch (FE + FULL)
Week 6–7   → Phase 4 : Catppuccin CSS (FE)
Week 8–9   → Phase 5 : USSR CSS (FE + DES)
Week 10–11 → Phase 6 : IPC backend (FULL + RS)
Week 12–13 → Phase 7 : Master feature (FE + QA)
Week 14    → Phase 8 : Security audit (SEC + RS)
Week 15–16 → Phase 9 : Encrypt/Sign/Verify (FE)
Week 17    → Phase 10 : Chat UI (FE)
Week 18–19 → Phase 11 : QA & tests (QA + A11Y)
Week 20–22 → Phase 12 : Beta (UX-TEST)
Week 23–24 → Phase 13 : Hotfixes (RS + FE)
---
Week 25    → Phase 14 : Astro setup (DOCS) [parallel from week 4]
Week 26–27 → Phase 15 : Doc migration (DOCS)
Week 28    → Phase 16 : Screenshots (DOCS + UX-TEST)
Week 29    → Phase 17 : Deploy (DOCS)
Week 30    → Phase 18 : Release (PM + RS)
```

---

## Decision gates

**Before each phase starts, confirm :**

1. ✅ **Technical readiness** : all dependencies from prior phase complete
2. ✅ **Resource availability** : all assigned agents confirmed
3. ✅ **Scope locked** : no scope creep authorized mid-phase
4. ✅ **DoD understood** : all tasks have clear completion criteria
5. ✅ **Risk mitigations** : current phase risks logged in README.md

**If ANY gate blocked :** PM escalates to user same day.

---

## Communication points

### For users
- **Weekly status** : Slack / Email (Monday EOD)
- **Phase reviews** : Scheduled calls when phase ends
- **Risk escalations** : ASAP (same day)

### For agents
- **Daily async standup** : Slack thread (before 11h)
- **Sync standups** : Lun 10h30, Jeu 14h (45 min / 1h)
- **Blockers** : Escalate to PM immediately (24h rule)

### Artifacts
- **This directory** : `.claude/plans/v0.8.0/`
  - README.md (this index file)
  - phases.md
  - tech-choices.md
  - security.md
  - agents.md
  - innovations.md
- **Git repo** : Issues + PRs tagged `#v0.8.0`
- **Archive** : decisions.md (TBD, Phase 1)

---

## How to use this plan

### First time reading

1. Start with **README.md** (15 min) — understand the 3 pillars + timeline
2. Skim **phases.md** intro (10 min) — see phase structure
3. Read **tech-choices.md** (20 min) — validate framework decisions
4. Check **agents.md** (10 min) — see if your agent is assigned
5. Refer back as needed during execution

### During Phase N

1. Open **phases.md** → find Phase N section
2. Check tasks (T N.1, T N.2, ...) + DoD checklist
3. Read **agents.md** → see task assignments for this phase
4. Consult **tech-choices.md** if making architectural decisions
5. Reference **security.md** if security concerns arise
6. Check **innovations.md** for optional enhancements

### When blocked

1. Check phase dependencies (phases.md dependency diagram)
2. Escalate blocker to PM (24h rule)
3. If security : consult security.md + PM
4. If design : consult innovations.md + DES
5. If timeline : PM updates README.md risks

---

## Key metrics (track in README.md)

| Metric | Target | Owner |
|--------|--------|-------|
| On-time delivery | > 90% phases on-schedule | PM |
| Budget variance | < 5% | PM |
| Test coverage | > 80% by Phase 11 | QA |
| Security audit | 0 CRITICAL, max 3 HIGH | SEC-A |
| Beta satisfaction | avg rating > 4/5 | UX-TEST |
| Doc completeness | 100% migration Phase 15 | DOCS |

---

## Document changelog

| Date | Change | Author |
|------|--------|--------|
| 2026-05-12 | Initial plan v0.8.0 created | PM |
| TBD | Decisions log started (Phase 1) | PM |
| TBD | Risk updates (weekly) | PM |

---

## Questions / FAQ

**Q: Can phases run in parallel?**  
A: Limited parallelization possible (see phases.md dependency diagram). Phase 14–17 (doc) can run parallel to Phase 7–13 (app).

**Q: What if v0.8.0 is delayed?**  
A: PM adjusts README.md risks, extends Phase 13 / 12 as needed. No scope creep without user approval.

**Q: Can we add features mid-v0.8.0?**  
A: No. New features → innovations.md → v0.9.0 roadmap. Scope locked Phase 1.

**Q: Who approves design?**  
A: DES (primary), PM (final), user (approval gate Phase 2–5).

**Q: Who handles urgent bugs?**  
A: Phase 12 : UX-TEST triage → PM decides include v0.8.0 vs defer. Phase 13 : RS + FE implement hotfixes.

---

## Links

- **Repository :** github.com/gfriloux/pgpilot
- **Issues :** github.com/gfriloux/pgpilot/issues?q=label%3Av0.8.0
- **Releases :** github.com/gfriloux/pgpilot/releases
- **Docs (planned)** : pgpilot.local or custom domain (Phase 17)

---

**Last updated :** 2026-05-12  
**Next review :** Phase 1 completion (2 weeks)  
**Maintained by :** @project-manager
