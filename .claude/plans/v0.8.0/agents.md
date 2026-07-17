# Allocation des agents — pgpilot v0.8.0

---

## Agents disponibles et rôles

| Agent | Sigle | Domaine | Compétences clés |
|-------|-------|--------|-----------------|
| **ui-designer** | DES | Design / UX/UI | Maquettes, design tokens, composants visuels |
| **frontend-developer** | FE | Frontend / React | React, TypeScript, CSS, composants web |
| **fullstack-developer** | FULL | Backend + Frontend | IPC, intégration Rust ↔ JS |
| **rust-engineer** | RS | Rust / Backend | Refactoring Tauri cmds, sécurité |
| **security-auditor** | SEC-A | Sécurité (audit) | Vulnérabilités, IPC, WebView, threat model |
| **penetration-tester** | SEC-P | Sécurité (attaque) | Fuzzing, injection, exploits |
| **test-automator** | QA-AUTO | Tests automatisés | Playwright, CI/CD, coverage |
| **code-reviewer** | REVIEW | Qualité code | PR review, standards, arch decisions |
| **accessibility-tester** | A11Y | Accessibilité | WCAG, screen readers, keyboard nav |
| **technical-writer** | DOCS | Documentation | mdBook → Astro, EN/FR, screenshots |
| **ui-ux-tester** | UX-TEST | Tests utilisateur | Beta feedback, UX validation |
| **project-manager** | PM | Coordination | Planning, risques, communication |

---

## Allocation par phase

### Phase 1 : Dérisking Tauri (sem 1–2)

**Lead :** PM + RS  
**FTE :** 2

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **PM** | T1.1–T1.5 oversight + decision gating | 2 sem | Status quo, risk log updated |
| **RS** | T1.1–T1.4 technical execution (Tauri PoC, IPC, perf) | 2 sem | PoC compiles, IPC latency < 5ms |
| **FULL** | Observation + suggestions | 1 sem | Notes on architecture |

**Standup :** 2x par semaine (Lun, Jeu)

---

### Phase 2 : Design system & tokens (sem 3)

**Lead :** DES + FE  
**FTE :** 2

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **DES** | T2.1 tokens.json creation (couleurs, typo, spacing) | 1.5 sem | tokens.json + design.md |
| **DES** | T2.3 component design (10 base components) | 1.5 sem | Figma / sketches |
| **FE** | T2.2 Storybook setup + styling | 1 sem | Storybook local build |
| **FE** | T2.3 implement 10 components in Storybook | 2 sem | 10 components storybooked |
| **PM** | Design review + validation | 0.5 sem | Approval gate |

---

### Phase 3 : Frontend architecture (sem 4–5)

**Lead :** FE + FULL  
**FTE :** 2.5

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **FE** | T3.1–T3.2 Vite + React Router setup | 1.5 sem | Project scaffold, no warnings |
| **FE** | T3.3–T3.4 State mgmt (Zustand) + folder structure | 1 sem | Stores, folder layout stable |
| **FULL** | T3.5 IPC hooks (useKeys, useCreateKey, etc.) | 1 sem | 5 hooks tested |
| **RS** | Observation, Rust types preview | 0.5 sem | Notes on Rust-side types |
| **PM** | Arch review + approval | 0.5 sem | Architecture freeze |

---

### Phase 4 : Thème Catppuccin (web) (sem 6–7)

**Lead :** FE  
**FTE :** 2

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **FE** | T4.1–T4.3 CSS variables + components + layout | 2 sem | Catppuccin theme complete |
| **DES** | T4.4 design validation (color swatches, preview page) | 0.5 sem | Sign-off on color accuracy |
| **PM** | Preview page test (user approval gate) | 0.5 sem | Color validation complete |

---

### Phase 5 : Thème USSR (web) (sem 8–9)

**Lead :** FE + DES  
**FTE :** 2.5

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **FE** | T5.1–T5.5 CSS + assets + components | 2.5 sem | USSR theme complete |
| **DES** | T5.2 asset integration review | 0.5 sem | Banner/badge assets validated |
| **FE** | T5.4–T5.5 fonts + theme switcher | 1 sem | Tema toggle functional |
| **PM** | Visual approval | 0.5 sem | Design sign-off |

---

### Phase 6 : IPC Tauri (backend → frontend) (sem 10–11)

**Lead :** FULL + RS  
**FTE :** 2.5

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **RS** | T6.1 refactor backend into Tauri commands (15+ commands) | 1.5 sem | Commands compiled, no lint |
| **FULL** | T6.2 tauri-specta setup (auto-gen TS bindings) | 0.5 sem | TS bindings generated |
| **FULL** | T6.3–T6.4 error handling + IPC hooks | 1 sem | Type-safe wrapper hooks |
| **QA-AUTO** | T6.5 integration tests (Rust + Tauri) | 0.5 sem | 20+ tests passing |
| **REVIEW** | Code review (IPC layer) | 0.5 sem | PR approved |

---

### Phase 7 : Porte-clés (master feature) (sem 12–13)

**Lead :** FE + QA-AUTO  
**FTE :** 3

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **FE** | T7.1–T7.6 views + workflows (list, detail, create, import, subkey, yubikey) | 2.5 sem | All views functional |
| **DES** | Design review (layout, spacing, visual hierarchy) | 0.5 sem | Design approval |
| **RS** | Support + troubleshooting | 0.5 sem | Command integration OK |
| **QA-AUTO** | T7.7 E2E tests (25+ Playwright scenarios) | 1 sem | 25+ tests passing |
| **REVIEW** | Code review (master detail, state mgmt) | 0.5 sem | PR approved |

---

### Phase 8 : Audit sécurité Tauri (sem 14)

**Lead :** SEC-A + RS  
**FTE :** 2

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **SEC-A** | T8.1 IPC audit + fuzzing | 1.5 sem | Audit report, findings |
| **SEC-A** | T8.2–T8.3 WebView + permissions audit | 1 sem | Detailed checklist |
| **RS** | T8.4 implement fixes (CRITICAL) | 0.5 sem | Fixes merged |
| **SEC-P** | Fuzzing assist (optional) | 0.5 sem | Fuzz test results |
| **SEC-A** | T8.5 report + sign-off | 0.5 sem | Final audit report |
| **PM** | Triage + communication | 0.5 sem | Risk acceptance |

---

### Phase 9 : Opérations (encrypt/sign/verify) (sem 15–16)

**Lead :** FE + RS  
**FTE :** 2.5

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **FE** | T9.1–T9.4 UI views (Encrypt, Sign, Verify) + drag-drop | 2 sem | All workflows interactive |
| **RS** | Support (backend logic review) | 0.5 sem | Command integration OK |
| **FE** | T9.4 drag-drop integration (X11 + Wayland) | 0.5 sem | Drag-drop functional |
| **QA-AUTO** | T9.5 E2E tests (15 scenarios) | 1 sem | 15+ tests passing |
| **REVIEW** | Code review | 0.5 sem | PR approved |

---

### Phase 10 : Chat chiffré (migration) (sem 17)

**Lead :** FE + RS  
**FTE :** 2

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **FE** | T10.1–T10.5 room list, detail, new room, join room, presence/ACK | 1.5 sem | Chat views functional |
| **RS** | Assist MQTT integration | 0.5 sem | Backend hooks ready |
| **QA-AUTO** | T10.6 E2E tests (10 scenarios) | 0.5 sem | 10+ tests passing |
| **REVIEW** | Code review | 0.5 sem | PR approved |

---

### Phase 11 : QA & tests E2E (sem 18–19)

**Lead :** QA-AUTO + A11Y  
**FTE :** 2.5

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **QA-AUTO** | T11.1 coverage analysis (code + E2E) | 1 sem | Coverage report, gaps identified |
| **QA-AUTO** | T11.4–T11.5 cross-platform + smoke tests | 1 sem | Test matrix passed |
| **A11Y** | T11.3 accessibility audit (WCAG AA) | 1 sem | axe scan, screen reader test |
| **FE** | Perf audit + optimization prep | 0.5 sem | Bottlenecks identified |
| **PM** | QA coordination | 0.5 sem | Status check |

---

### Phase 12 : Validation utilisateur (beta) (sem 20–22)

**Lead :** UX-TEST  
**FTE :** 0.5

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **UX-TEST** | T12.1–T12.2 feedback form + recruitment | 0.5 sem | Form live, participants confirmed |
| **UX-TEST** | T12.3 monitoring (daily triage) | 2 sem | Feedback aggregated, logs |
| **PM** | T12.4 analysis + decision meeting | 1 sem | Items triaged (CRITICAL/HIGH/defer) |
| **FE** (on-call) | Ad-hoc support for beta blockers | 2 sem | Urgent fixes deployed |

---

### Phase 13 : Corrections post-beta (sem 23–24)

**Lead :** RS + FE  
**FTE :** 2

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **FE** | T13.1 implement hotfixes (frontend) | 1 sem | Hotfixes merged |
| **RS** | T13.1 hotfixes (backend) | 1 sem | Hotfixes merged |
| **FE** | T13.2 perf tuning (code splitting, caching) | 0.5 sem | Perf targets met |
| **QA-AUTO** | T13.3 regression testing | 0.5 sem | No new bugs introduced |
| **RS** | T13.4 release binary build | 0.5 sem | Binary signed & ready |
| **PM** | Release readiness check | 0.5 sem | Go/No-go decision |

---

### Phase 14 : Astro + Starlight setup (sem 25)

**Lead :** DOCS  
**FTE :** 1

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **DOCS** | T14.1–T14.3 Astro scaffold + i18n + theme | 1 sem | Astro project compiles, routes OK |
| **PM** | Validation | 0.5 sem | Doc structure approved |

**Note :** Peut démarrer en parallèle Phase 4–5 (indépendant de UI)

---

### Phase 15 : Migration doc mdbook → Astro (sem 26–27)

**Lead :** DOCS  
**FTE :** 1

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **DOCS** | T15.1–T15.3 content migration + French translation | 2 sem | Tous les .md files migrés EN + FR |
| **PM** | QA content | 0.5 sem | Links + images verified |

---

### Phase 16 : Screenshots & démo (sem 28)

**Lead :** DOCS + UX-TEST  
**FTE :** 1.5

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **UX-TEST** | Capture screenshots (15+) | 0.5 sem | All screenshots ready |
| **DOCS** | T16.1–T16.3 integrate screenshots + getting started + demo | 1 sem | Doc fully illustrated |
| **PM** | Doc validation | 0.5 sem | Content approved |

---

### Phase 17 : Déploiement GitHub Pages (sem 29)

**Lead :** DOCS (+ infra si available)  
**FTE :** 1

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **DOCS** | T17.1–T17.3 CI/CD + GitHub Pages + metadata | 1 sem | Doc live on GitHub Pages |
| **PM** | Verification | 0.5 sem | URL accessible, HTTPS OK |

---

### Phase 18 : Release & communication (sem 30)

**Lead :** PM  
**FTE :** 1

| Agent | Task | Durée | Livrable |
|-------|------|-------|----------|
| **RS** | Release binary build (final) | 0.5 sem | Signed binaries |
| **PM** | T18.1–T18.3 GitHub Release + announcement | 1 sem | v0.8.0 released publicly |
| **DOCS** | Blog post (optional tech writeup) | 1 sem | Blog published |

---

## Dépendances inter-agents

```mermaid
PM ─→ RS (Phase 1, 13)
  ├─→ DES (Phase 2, 4, 5)
  ├─→ FE (Phase 2, 3, 4, 5, 7, 9, 10, 13)
  │  ├─→ FULL (Phase 3, 6)
  │  │  └─→ RS (Phase 6, 9, 13)
  │  │     └─→ SEC-A (Phase 8)
  │  │        └─→ SEC-P (Phase 8)
  │  ├─→ QA-AUTO (Phase 7, 9, 10, 11, 13)
  │  │  └─→ REVIEW (Phase 6, 7, 9, 10)
  │  ├─→ A11Y (Phase 11)
  │  └─→ UX-TEST (Phase 12, 16)
  │
  └─→ DOCS (Phase 14, 15, 16, 17, 18)
```

---

## Communication cadence

### Standups quotidiens (async)

- Chaque agent écrit status 30min avant standup (Slack thread)
- PM compile et partage avec stakeholders
- Blockers escaladés immédiatement

### Synchrone

- **Lundi 10h30** : Standup PM + Lead designers + Lead FE (45 min)
- **Jeudi 14h** : Backlog grooming + risk review (1h)
- **Fin de phase** : Review + décision phase suivante (variable)

### Reporting

- **Toutes les 2 semaines** : Status quo mail à utilisateur
  - Phase name + completion %
  - Top blockers
  - Risk updates
  
### Escalade

- **Blocker 24h+** : PM → utilisateur + lead agent
- **Critical CVE discovered** : SEC-A → PM → utilisateur (same day)
- **Phase delay 3+ days** : PM → utilisateur (forecast slip)

---

## Ressources notes

### Availability expectations

- **Phase 1–7** (sem 1–13) : Équipe stable, focus principal
- **Phase 8** (sem 14) : Audit critical, SEC-A full-time
- **Phase 9–11** (sem 15–19) : Équipe stable
- **Phase 12** (sem 20–22) : Low resource (UX-TEST + FE on-call)
- **Phase 13** : Full effort (hotfixes)
- **Phase 14–17** : DOCS lead (async with Phase 13 Tauri finalization)
- **Phase 18** : Ramp-down (release only)

### Multi-task restrictions

- ❌ RS cannot lead Phase 1 + Phase 6 simultaneously
- ❌ FE cannot lead Phase 4 + Phase 5 + Phase 7 simultaneously (too much)
  - Solution : Phase 4–5 concurrent, Phase 6 starts after Phase 5
- ❌ SEC-A unavailable Phase 8 week except audit
- ✅ DOCS can work on Phase 14–17 in parallel with Phase 7–13 (independent path)

### Contingency

**If agent unavailable :**
- FE unavailable Phase 7 → FULL takes frontend, hire contractor
- RS unavailable Phase 1 → PM + contractor Rust expert
- SEC-A unavailable Phase 8 → Hire external audit firm

---

## Success metrics (team)

| Metric | Target | Method |
|--------|--------|--------|
| **On-time delivery** | > 90% phases on-schedule | Burndown chart per phase |
| **Code quality** | < 2 PRs rejected per phase | Review metrics |
| **Test coverage** | > 80% by Phase 11 | Coverage tool |
| **Team satisfaction** | Retro feedback (end of phase) | Anon survey |
| **Stakeholder happiness** | Approval gates signed-off | PM sign-off |

---

## Contacts & escalation

| Role | Primary | Backup |
|------|---------|--------|
| **Project Manager** | @pm | @utilisateur (escalation) |
| **Lead Designer** | @des | @pm |
| **Lead Frontend** | @fe | @full |
| **Lead Rust** | @rs | @full |
| **Security Lead** | @sec-a | @pm |
| **QA Lead** | @qa-auto | @fe |
| **Tech Writer** | @docs | @pm |

---

## Phase transitions (decision gates)

Each phase ends with **DoD validation** :

1. ✅ All tasks completed (checklist in phases.md)
2. ✅ No critical blockers
3. ✅ Code reviewed (REVIEW agent sign-off)
4. ✅ Tests passing (QA-AUTO sign-off)
5. ✅ Design approved (DES sign-off if applicable)
6. ✅ PM approves → next phase starts

**Fail gate :** Phase blocked until DoD met. PM communicates delay to utilisateur.
