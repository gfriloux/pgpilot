# Détail des phases — pgpilot v0.8.0

---

## Phase 1 : Dérisking Tauri (sem 1–2)

**Lead :** PM + rust-engineer  
**Sous-agents :** fullstack-developer (observation)  
**Dépendances :** Aucune

### Objectif
Valider qu'une migration Tauri est techniquement faisable et ne pose pas de risques bloquants. Démontrer une app Tauri v2 minimale avec IPC fonctionnelle.

### Tâches

**T1.1** — Scaffold Tauri v2 + TypeScript (2j)
- `npm create tauri-app@latest` avec template React minimal
- Vérifier que Tauri dépend bien de `webview2` (Windows), `wkwebview` (macOS), `webkit2gtk` (Linux)
- Compiler sur Linux (Wayland + X11)
- DoD : App démarrée, "Hello Tauri" affiché

**T1.2** — IPC backend → frontend simple (3j)
- Ajouter commande Rust : `#[tauri::command] fn get_version() -> String { "0.8.0-pre".to_string() }`
- Frontend TypeScript : appel `invoke("get_version")` au démarrage
- Afficher version en bas de fenêtre
- DoD : Appel IPC fonctionne bidirectionnellement, latency < 5ms

**T1.3** — Audit sécurité PoC (2j)
- CSP headers : `default-src 'self'`, `script-src 'self'` seulement
- WebView sandbox : vérifier `webview` config avec `allowlist: { all: false }`
- Checklist OWASP : injection, XSS, CSRF basiques
- DoD : Rapport 1-page, liste des mitigations à ajouter Phase 8

**T1.4** — Performance baseline (2j)
- Mesurer démarrage app : temps jusqu'à "ready"
- Mesurer IPC round-trip : 10ms target
- Mesurer mémoire : baseline vide
- DoD : Spreadsheet : démarrage, IPC latency, RAM, comparé à iced v0.7.0

**T1.5** — Décision framework frontend (1j)
- Réunion ui-designer + PM : présenter 3 options (React, Vue, Svelte)
- Critères : DX, perf, composants design, bundle size, communauté
- Voter
- DoD : Framework choisi, documenté dans tech-choices.md

### Critères de complétion
- [ ] App Tauri compiles et run
- [ ] IPC fonctionne sur X11 + Wayland
- [ ] Perf baseline établie
- [ ] Framework frontend choisi
- [ ] Aucun blocker critique identifié

---

## Phase 2 : Design system & tokens (sem 3)

**Lead :** ui-designer + frontend-dev  
**Sous-agents :** PM (validation)  
**Dépendances :** Phase 1 ✅

### Objectif
Créer l'ADN visuel du projet : palette couleurs, typo, espacements, composants de base.

### Tâches

**T2.1** — Tokens JSON (2j)
- Fichier `design-tokens.json` (ou `tokens.config.json` si Figma Tokens sync)
- Sections : colors (Catppuccin 26 + USSR 26), typography (fonts, sizes, weights), spacing (8px scale), shadows, radius, gaps
- Export : CSS variables template, Tailwind config, TypeScript types
- DoD : Fichier valide JSON Schema, import dans 3 formats

**T2.2** — Storybook setup (1j)
- Installer Storybook pour React (ou équivalent)
- Ajouter Chromatic (optionnel : test visuel)
- Configurer pour utiliser tokens.json
- DoD : Storybook local : `npm run storybook` → http://localhost:6006

**T2.3** — 10 composants de base (2j)
- Button (4 variantes : primary, secondary, ghost, destructive)
- Input (text, email, password)
- Card
- Badge
- Modal
- Tooltip
- Dropdown
- Radio/Checkbox
- Tabs
- Alert
- Chaque composant : 3 states (default, hover, disabled), Catppuccin + USSR

### Critères de complétion
- [ ] tokens.json complet, validé
- [ ] Storybook accessible en local
- [ ] 10 composants storybooked (Catppuccin + USSR)
- [ ] Palette couleurs visuellement comparée à iced v0.7.0 (reproduction fidèle)

---

## Phase 3 : Frontend architecture (sem 4–5)

**Lead :** frontend-dev + fullstack-dev  
**Sous-agents :** rust-engineer (observation)  
**Dépendances :** Phase 2 ✅, Phase 1 dérisking ✅

### Objectif
Poser l'architecture frontend : routing, state management, folder structure, TypeScript strictement configuré.

### Tâches

**T3.1** — Vite + React 18 + TypeScript (2j)
- Upgrader template Tauri pour Vite (pas Create React App — lourd)
- `tsconfig.json` : `strict: true`, `noImplicitAny: true`, `exactOptionalPropertyTypes: true`
- Eslint + prettier configuré
- Husky pre-commit hooks : format + lint
- DoD : `npm run build` produit bundle sans warnings

**T3.2** — Routing (React Router v6) (1j)
- Installer `react-router-dom`
- 8 routes principales : `/`, `/keys`, `/keys/:id`, `/create-key`, `/import`, `/encrypt`, `/sign`, `/verify`, `/chat`, `/settings`, `/health`
- Outlets et nested routes
- DoD : Navigation fluide, state persist via URL params

**T3.3** — State management (Redux Toolkit ou Zustand) (1j)
- Choisir : Redux Toolkit (scalable) ou Zustand (minimaliste, recommandé < 20 states)
- Slices pour : ui (view, pending_op, status), keys (list, selected, loading), config (theme, language, scale)
- Hydration depuis localStorage au démarrage
- DoD : States centralisés, aucun useState global

**T3.4** — Folder structure (1j)
```
src/
├── pages/          (route components)
├── components/     (reusable UI)
├── hooks/          (custom hooks, IPC wrappers)
├── store/          (redux/zustand)
├── types/          (shared TS types)
├── utils/          (helpers, formatters)
├── styles/         (global CSS, tokens)
└── ipc/            (generated Tauri bindings)
```
- DoD : Codebase naviguable, imports clairs

**T3.5** — IPC hooks (1j)
- `useKeys()` → `invoke("list_keys")` + loading/error states
- `useCreateKey()` → async wrapper
- Pattern : `useAsync(command, deps)` générique
- Types générées depuis Rust (tauri-specta ou manuel)
- DoD : 5 hooks testés, zéro boilerplate répété

### Critères de complétion
- [ ] Vite build zero warnings
- [ ] React Router 8 routes naviguables
- [ ] State management centralisé (Redux ou Zustand)
- [ ] Folder structure documentée
- [ ] IPC hooks réutilisables

---

## Phase 4 : Thème Catppuccin (web) (sem 6–7)

**Lead :** frontend-dev  
**Sous-agents :** ui-designer (review), PM (validation)  
**Dépendances :** Phase 3 ✅, Phase 2 design tokens

### Objectif
Implémenter le thème Catppuccin Frappé en CSS, matching le rendu iced v0.7.0.

### Tâches

**T4.1** — CSS variables (1j)
- Créer `src/styles/theme-catppuccin.css` avec 26 variables (sidebar_bg, detail_bg, text_strong, accent, etc.)
- Utiliser sRGB pour vérifier exactitude vs iced
- `:root.theme-catppuccin { --sidebar-bg: #303446; ... }`
- DoD : Variabilité complète, pas de couleurs hardcodées

**T4.2** — Composants Catppuccin (2j)
- Updater 10 composants Storybook (Phase 2) pour utiliser CSS variables
- Button : background `var(--button-bg)`, hover `var(--button-hover-bg)`, text `var(--text-on-button)`
- Card : `var(--card-bg)`, border `var(--border)`
- Badge : variants success / error / warning / neutral
- DoD : Visuellement identique à screenshot iced v0.7.0

**T4.3** — Global layout (1j)
- Sidebar layout : 180px left, scrollable, themed
- Detail panel : flexible width
- Master-detail responsive (stacking mobile, mais layout principal mobile-last)
- DoD : Layout stacks mobile, details visible desktop

**T4.4** — Preview & validation (1j)
- Créer page `/settings/theme-preview` : display 2 panels (Catppuccin + USSR inline)
- Color swatch grid : aperçu de chaque couleur + hex
- Component showcase : buttons, cards, modals
- DoD : Designer valide couleurs visuellement

### Critères de complétion
- [ ] Toutes 26 variables CSS Catppuccin définies
- [ ] Composants appliquent variables (zéro hardcoding)
- [ ] Layout master-detail responsif
- [ ] Preview page fonctionnelle

---

## Phase 5 : Thème USSR (web) (sem 8–9)

**Lead :** frontend-dev + ui-designer  
**Sous-agents :** PM (validation)  
**Dépendances :** Phase 4 ✅, Phase 2 assets (bannières PNG + badges SVG)

### Objectif
Implémenter le thème USSR v2 en web : bannières propagandistes, badges SVG, palette soviétique, fontes Bebas Neue / Russo One.

### Tâches

**T5.1** — CSS variables USSR (1j)
- Créer `src/styles/theme-ussr.css`
- Palette : cream (#F0EBD8), near-black sidebar (#0F0D09), Soviet red (#D82C20)
- 26 variables mapping (sidebar_bg = #0F0D09, text_strong = #1A1815, accent = #D82C20, etc.)
- `:root.theme-ussr { ... }`
- DoD : CSS complet, color-accurate per v0.7.0 design

**T5.2** — Assets PNG/SVG (1j)
- Copier 12 bannières PNG de v0.7.0 (`src/assets/banners/*.png`)
- Intégrer 5 badges SVG (keyserver, yubikey, trust-full, trust-marginal, trust-undefined)
- Vérifier transparence coins PNG (radius 13px)
- DoD : Assets intégrés, chargements sans 404, webp fallback

**T5.3** — Composants USSR (1.5j)
- Updater 10 composants + 5 nouveaux USSR-spécifiques
- Button : cream bg, red accent
- Sidebar : gradient (red → dark)
- Card : cream bg (#F0EBD8), subtle borders
- Badges : SVG inline (React component wrapper)
- Bannières : composant `<Banner viewName="mykeys" />` auto-img
- DoD : Visuellement identique v0.7.0 USSR

**T5.4** — Fontes Bebas Neue / Russo One (0.5j)
- `@import` depuis Google Fonts (ou self-hosted)
- CSS classes : `.bebas` (nav labels), `.russo` (headings)
- Fallback : système sans web fonts
- DoD : Fontes chargées, zéro layout shift (font-display: swap)

**T5.5** — Thème switcher (1j)
- Page Settings : radio buttons (Catppuccin / USSR)
- Dispatch Redux action : `setTheme('ussr')`
- Changer classe root : `document.root.classList.toggle('theme-ussr')`
- Persistance localStorage
- DoD : Switching instantané, état persist refresh

### Critères de complétion
- [ ] CSS variables USSR complet
- [ ] 12 bannières PNG + 5 badges SVG intégrés
- [ ] Composants styled USSR
- [ ] Fontes chargées sans layout shift
- [ ] Thème switcher fonctionnel

---

## Phase 6 : IPC Tauri (backend → frontend) (sem 10–11)

**Lead :** fullstack-dev + rust-engineer  
**Sous-agents :** test-automator (intégration tests)  
**Dépendances :** Phase 3 architecture ✅, Phase 5 UI ✅

### Objectif
Créer l'IPC type-safe entre Rust (backend GPG/MQTT/crypto) et frontend TypeScript. Refactoriser le backend iced en commandes Tauri.

### Tâches

**T6.1** — Commandes Tauri Rust (2j)
- Créer `src-tauri/src/commands/` avec modules :
  - `keyring.rs` : list_keys, get_key, create_key, import_key, delete_key, export_key, etc.
  - `config.rs` : get_config, set_theme, set_language, set_scale
  - `gpg.rs` : encrypt_files, sign_file, verify_signature
  - `chat.rs` : create_room, join_room, send_message, get_messages
  - `health.rs` : run_diagnostics
- Signature uniforme : `#[tauri::command] async fn list_keys(handle: tauri::AppHandle) -> Result<Vec<KeyInfo>, String> { ... }`
- DoD : 15+ commandes compilées, aucun lint warning

**T6.2** — Type-safe IPC avec tauri-specta (1j)
- Installer `tauri-specta` pour auto-générer bindings TypeScript depuis Rust
- Traits : `#[specta::specta]` sur tous les types de commandes
- Génération : `cargo run --bin specta-codegen`
- TypeScript : importer depuis `src/ipc/bindings.ts` (généré)
- DoD : Commandes Rust = TS types automatiquement synced

**T6.3** — Error handling type-safe (1j)
- Types Rust côté backend : `CustomError(String)`, `GpgError(String)`, `ConfigError(String)`
- Enum Rust : `#[derive(serde::Serialize)] enum ErrorKind { ... }`
- Frontend : `try { await invoke(...) } catch (e: TauriError) { handle e.kind }`
- DoD : Erreurs structurées, messages utilisateur clairs

**T6.4** — IPC hooks frontend (1j)
- Wrappers React : `useKeys()`, `useCreateKey()`, `useTheme()`, `useConfig()`
- Loading/error states centralisés (Zustand)
- Retry logic avec backoff exponentiel
- DoD : 10+ hooks, zéro boilerplate

**T6.5** — Integration tests (1j)
- Tests Rust : `#[tokio::test] async fn test_list_keys_empty() { ... }`
- Tests Tauri : Playwright E2E (voir Phase 11)
- Coverage : 80% des commandes
- DoD : 20+ tests passants

### Critères de complétion
- [ ] 15+ commandes Rust compilées
- [ ] TypeScript bindings auto-générés (tauri-specta)
- [ ] Error handling type-safe
- [ ] 10 IPC hooks frontend
- [ ] 20+ tests passants

---

## Phase 7 : Porte-clés (master feature) (sem 12–13)

**Lead :** frontend-dev + test-automator  
**Sous-agents :** rust-engineer (support), ui-designer (review)  
**Dépendances :** Phase 6 IPC ✅, Phase 4–5 UI ✅

### Objectif
Implémenter le feature central : gestion des clés PGP. Interface complète CRUD.

### Tâches

**T7.1** — List view (1j)
- Master panel (320px, scrollable)
- Rows : nom + email (2 lignes), expiry badge, trust icon
- Sélection : highlight bg + detail panel auto-update
- Filtering : chercher par nom/email/fingerprint
- DoD : 20+ clés affichées, scroll fluide, search fonctionnel

**T7.2** — Detail panel (2j)
- Left column : key info (fingerprint, created, expiry, owner trust picker)
- Right column : subkeys table (Type | Algo | Expiry | Actions)
- Trust picker : radio buttons (Undefined → Marginal → Full)
- Action buttons : Export | Backup | Migrate YubiKey | Publish | Delete
- DoD : Toutes les actions cliquables, modals d'confirmation affichés

**T7.3** — Create key modal (1.5j)
- Form : Name + Email
- Keysize : 2048 / 4096 / Ed25519 (radio)
- Expiry : None / 1Y / 5Y / Custom date
- Submit : dispatch `create_key()` IPC, progress bar
- DoD : Clé créée en < 10s (test local), erreur gérée

**T7.4** — Import workflows (1.5j)
- 4 sources : File | URL | Keyserver | Paste armored
- File : drag & drop + file picker (rfd async)
- URL : input + fetch (safe_get wrapper)
- Keyserver : fingerprint/email input, search via HKP
- Paste : textarea, validation `-----BEGIN PGP`
- DoD : Tous les workflows terminés, erreurs claires

**T7.5** — Subkey management (1.5j)
- Renew expiry : modal date picker
- Rotate (replace) : auto-revoke old, create new
- Add new : modal (Type radio)
- Delete (public key only) : confirmation modal
- DoD : Modals fonctionnels, backend calls OK

**T7.6** — YubiKey migration (1j)
- Detect card : `card_status()` IPC call
- Migration workflow : key picker → slot selector (SIG/ENC/AUT) → confirm
- Progress : "Moving to card..."
- Post-migration : subkey shows "YubiKey" badge
- DoD : Badge displays, workflow non-blocking

**T7.7** — E2E tests (2j)
- Playwright : 25+ scenarios
  - Create, list, select, detail view (4)
  - Import file, URL, keyserver, paste (4)
  - Export public / secret (2)
  - Delete with confirmation (1)
  - Subkey renew / rotate / add (3)
  - YubiKey migration (1)
  - Trust level changes (1)
  - Performance : load 50 keys (1)
- Coverage : 80%+
- DoD : Tous les tests passants, < 100ms per action

### Critères de complétion
- [ ] List + Detail views complets
- [ ] Create + Import workflows fonctionnels
- [ ] Subkey management opérationnel
- [ ] YubiKey migration affichée
- [ ] 25+ E2E tests passants
- [ ] No critical UI bugs

---

## Phase 8 : Audit sécurité Tauri (sem 14)

**Lead :** security-auditor + rust-engineer  
**Sous-agents :** PM (triage), code-reviewer (fixes)  
**Dépendances :** Phase 7 ✅

### Objectif
Audit complet de la surface d'attaque Tauri/WebView post-Phase 7. Identifier et corriger tous les vulnérabilités CRITICAL.

### Tâches

**T8.1** — Audit surface IPC (2j)
- Checklist : injection command, path traversal, symlink follow, race conditions
- Chaque commande Rust : vérifier validation inputs
- Fuzzing : random inputs sur 5 commandes principales
- Résultat : rapport avec CVSS scores
- DoD : Zéro CRITICAL, MAX 3 HIGH

**T8.2** — Audit WebView (1j)
- CSP headers : vérifier strict (no `unsafe-inline`, no `*` wildcard)
- Frame options : `X-Frame-Options: DENY`
- Cookie security : `HttpOnly`, `Secure`, `SameSite=Strict`
- Insecure resource loading : zéro http:// dans prod
- DoD : Tous les headers validés

**T8.3** — Audit permissions Tauri (1j)
- `tauri.conf.json` : `allowlist` = whitelist strict
- Examine : fs, http, shell, notification, window
- Deny : `{ all: false }` par défaut
- DoD : Chaque permission documentée et justifiée

**T8.4** — Fixes CRITICAL (1j)
- Appliquer correctifs immédiats pour tous les CRITICAL
- Re-test avec audit tools (semgrep, cargo-audit)
- DoD : Zéro CRITICAL restants

**T8.5** — Rapport et documentation (1j)
- Rapport d'audit 5–10 pages : méthodologie, findings, fixes
- Sauvegarder dans `.claude/plans/v0.8.0/audit-report.md`
- Communiquer au chef de projet + utilisateur
- DoD : Rapport finalisé, approbation utilisateur obtenue

### Critères de complétion
- [ ] Audit IPC terminé
- [ ] Audit WebView terminé
- [ ] Audit permissions Tauri terminé
- [ ] Zéro CRITICAL vulnérabilités
- [ ] MAX 3 HIGH non-fixés (justifiés)
- [ ] Rapport finalisé

---

## Phase 9 : Opérations (encrypt/sign/verify) (sem 15–16)

**Lead :** frontend-dev + rust-engineer  
**Sous-agents :** test-automator (E2E), ui-designer (review)  
**Dépendances :** Phase 7 ✅, Phase 8 audit ✅

### Objectif
Implémenter 3 workflows critiques : chiffrement fichiers, signature, vérification signature.

### Tâches

**T9.1** — Encrypt view (1.5j)
- File picker + drag & drop
- Recipient selector : master-detail (list gauche, picker droite)
- Recipients chips : 2-per-row grid, removable
- Trust warning modal : "1 recipients untrusted — force?"
- Format toggle : .gpg (binary) vs .asc (armored)
- Progress bar + status
- DoD : Fichiers chiffrés, output next to original

**T9.2** — Sign view (1j)
- File picker + drag & drop
- Key picker : filtered signing-capable keys only (Sign algo)
- Output : `<file>.sig` (detached signature)
- Status : success / error
- DoD : .sig généré, vérifiable

**T9.3** — Verify view (1.5j)
- File picker + drag & drop
- Signature file picker (detect `<file>.sig` auto)
- 5-state outcome display :
  - Valid + Full trust → ✅ Green badge + signer name + date
  - Valid + Marginal trust → ⚠️ Orange badge + signer name
  - Valid + Undefined trust → ⚠️ Yellow badge
  - BadSig → ❌ Red badge
  - UnknownKey / ExpiredKey / RevokedKey → ❌ Red + specific message
- Fingerprint display + copy button
- DoD : Tous les états affichés correctement

**T9.4** — Drag & drop integration (0.5j)
- `window.addEventListener('drop', ...)` → `useDropZone()` hook
- Files captured → IPC `handle_dropped_files()`
- Route to appropriate view (Encrypt vs Sign vs Verify)
- DoD : Drag & drop fonctionne X11 + Wayland

**T9.5** — E2E tests (1j)
- 15 scenarios : encrypt (multi-recipient, trust warn), sign, verify (5 states), drag-drop
- Performance : chiffrer 100MB file < 5s
- DoD : Tous tests passants

### Critères de complétion
- [ ] Encrypt view complet (drag-drop, recipients, trust warning)
- [ ] Sign view avec output .sig
- [ ] Verify view : 5 states corrects
- [ ] Drag & drop fonctionnel (X11 + Wayland)
- [ ] 15+ E2E tests passants

---

## Phase 10 : Chat chiffré (migration UI) (sem 17)

**Lead :** frontend-dev + rust-engineer  
**Sous-agents :** test-automator (E2E)  
**Dépendances :** Phase 9 ✅

### Objectif
Migrer le chat PGP chiffré depuis l'UI iced vers web Tauri. Keepingbackend MQTT/crypto unchanged.

### Tâches

**T10.1** — Room list view (0.5j)
- Sidebar : 280px left, scrollable
- Room cards : name + last message snippet + timestamp + unread count
- Click → detail view
- New room button → modal
- DoD : List scrolls, click navigates

**T10.2** — Room detail view (1j)
- Header : room name + online status badge
- Message list : bubbles (sender name + timestamp + message text + trust badge)
- Input : textarea + send button + attach file (optional)
- Auto-scroll new messages
- DoD : Messages affichées, send functional

**T10.3** — New room form (0.5j)
- Input : room name
- Participant picker : multi-select (fingerprints)
- Relay URL : text input (pre-filled HiveMQ)
- Create button → join_room IPC
- DoD : Room créée, liste updated

**T10.4** — Join room form (0.5j)
- Input : join code (encoded 20-char string)
- Decode + verify signature (backend IPC call)
- Show room details before join
- Join button
- DoD : Joindre room fonctionne

**T10.5** — Presence & ACK (0.5j)
- Online badge dans header
- ACK checkmarks sous les messages (1 check = sent, 2 checks = delivered)
- DoD : Badges affichés, comportement correct

**T10.6** — E2E tests (1j)
- 10 scenarios : create room, join, send message, receive, presence, ACK
- Performance : send 10 messages < 1s (latency MQTT)
- DoD : Tous tests passants

### Critères de complétion
- [ ] Room list + detail views
- [ ] New room + join room forms
- [ ] Presence & ACK affichés
- [ ] 10+ E2E tests passants
- [ ] No MQTT connection issues

---

## Phase 11 : QA & tests E2E (sem 18–19)

**Lead :** test-automator + accessibility-tester  
**Sous-agents :** frontend-dev (support), PM (coordination)  
**Dépendances :** Phase 10 ✅

### Objectif
Validation complète : couverture tests > 80%, performance, accessibilité WCAG AA.

### Tâches

**T11.1** — Coverage analysis (1j)
- Playwright : E2E coverage (Phases 7–10 = 50+ scenarios)
- Unit tests Rust : GPG/MQTT layers (30+ tests)
- Code coverage : `cargo tarpaulin` + `nyc` (JS)
- Goal : > 80% coverage, identify gaps
- DoD : Rapport coverage, plan pour 20% restants

**T11.2** — Performance audit (1j)
- Startup time : < 2s (target)
- IPC latency : < 10ms (target)
- UI transitions : 60fps (lighthouse)
- Bundle size : < 100MB (app size)
- RAM baseline : < 200MB idle
- DoD : Spreadsheet benchmark, comparison iced v0.7.0

**T11.3** — Accessibility audit (1j)
- axe DevTools scan : toutes les pages
- WCAG AA level : min 85 / 100 score
- Screen reader test (NVDA / VoiceOver)
- Keyboard navigation : all features accessible
- Color contrast : 4.5:1 min (WCAG AA)
- DoD : Audit report, fixes pour blockers

**T11.4** — Cross-platform testing (1j)
- Linux X11 + Wayland : smoke tests (5 clés, chat, encrypt)
- Possible : Windows VM (WebView2), macOS (VM rental)
- Focus : UI consistency, no platform-specific bugs
- DoD : Test matrix passed

**T11.5** — Smoke tests (user journeys) (1j)
- Happy path : import clé → list → detail → subkey renew → export
- Chat : create room → send message → receive → ACK
- Encrypt : select file → recipients → trust warn → encrypt
- Sign : select file → sign → generate .sig
- Verify : select .sig → verify → show outcome
- DoD : Tous les chemins valides fonctionnels

### Critères de complétion
- [ ] Coverage > 80%
- [ ] Startup time < 2s
- [ ] IPC latency < 10ms
- [ ] WCAG AA score > 85
- [ ] Smoke tests 100% pass rate

---

## Phase 12 : Validation utilisateur (beta) (sem 20–22)

**Lead :** ui-ux-tester  
**Sous-agents :** PM (feedback triage), frontend-dev (support)  
**Dépendances :** Phase 11 ✅

### Objectif
10+ utilisateurs testent l'app pendant 2 semaines. Collecter feedback structuré sur UX, graphique, perf, bugs.

### Tâches

**T12.1** — Formulaire feedback (0.5j)
- Google Form / Typeform :
  - Sections : UX (5-point), graphique design (5-point), performance (5-point), bugs (free text), suggestions
  - Feedback quantitatif + qualitatif
  - Données CSVs
- DoD : Formulaire live, shareable link

**T12.2** — Recruitment & setup (1j)
- Email 10–15 candidates (diverse usage : débutants, experts, designers)
- Envoyer release candidate binaire
- Envoyer lien formulaire + guide rapide
- Slack/Discord channel pour questions
- DoD : Participants confirmés, binaires distribués

**T12.3** — Monitoring & support (2 sem)
- Logs structurés : bug reports, crash dumps (opt-in telemetry)
- PM : daily triage feedback → label (UX / Design / Perf / Critical / Trivial)
- Weekly sync avec participants : questions, blockers
- DoD : 10+ responses, tous les feedbacks collectés

**T12.4** — Analysis & triage (1j)
- Spreadsheet : feedback aggregé par catégorie
- Scoring : criticality, frequency
- Decision meeting : quoi inclure 0.8.0 vs v0.9.0 ?
- DoD : Décision items documentée dans `decisions.md`

### Critères de complétion
- [ ] 10+ beta participants
- [ ] Feedback form responses > 50%
- [ ] All feedback triaged et scoré
- [ ] Decision : inclusion v0.8.0 vs defer

---

## Phase 13 : Corrections post-beta (sem 23–24)

**Lead :** rust-engineer + frontend-dev  
**Sous-agents :** PM (prioritization), test-automator (regression)  
**Dépendances :** Phase 12 ✅, Phase 8 audit ✅

### Objectif
Implémenter hotfixes critiques du feedback beta + perf tuning final.

### Tâches

**T13.1** — Hotfixes critiques (1j)
- Stack items décidés en Phase 12 (CRITICAL + HIGH)
- Bugs : crashes, UX blockages, security issues
- Perf : si startup > 2.5s ou IPC > 15ms, optimize
- DoD : Tous les CRITICAL items fixés

**T13.2** — Perf tuning (1j)
- Profile startup avec flamegraph
- IPC : lazy-load keys, pagination
- Frontend : code splitting, lazy routes
- Caching : localStorage, IndexedDB pour messages
- DoD : Startup < 2s confirmed, IPC < 10ms

**T13.3** — Regression testing (1j)
- Re-run Phase 11 smoke tests
- Verify fixes don't break existing workflows
- Coverage remains > 80%
- DoD : No new regressions

**T13.4** — Release candidate build (1j)
- Tauri build : `cargo tauri build --release`
- Sign binaries : cosign or equivalent
- Create release notes : features, bugs fixed, known limitations
- DoD : Binary release candidate ready

### Critères de complétion
- [ ] All CRITICAL hotfixes merged
- [ ] Perf tuning validated
- [ ] Regression tests 100% pass
- [ ] Release candidate binary signed

---

## Phase 14 : Astro + Starlight setup (sem 25)

**Lead :** technical-writer  
**Sous-agents :** PM (validation)  
**Dépendances :** Phase 1 ✅ (can start in parallel)

### Objectif
Scaffold documentation project : Astro + Starlight, structure i18n EN/FR, local build OK.

### Tâches

**T14.1** — Astro scaffold (1j)
- `npm create astro@latest` with Starlight template
- `npm run build` → static site
- `npm run dev` → local server
- DoD : Local build works, `http://localhost:3000` loads

**T14.2** — i18n structure (1j)
- Starlight i18n : root config `astro.config.mjs` with `defaultLocale: 'en'`, `locales: { en: {...}, fr: {...} }`
- Folder structure : `docs/src/en/` + `docs/src/fr/`
- Navigation sidebar : translated per locale
- DoD : `/en/` + `/fr/` routes active, content loads

**T14.3** — Theme customization (0.5j)
- CSS variables : match pgpilot tokens (Catppuccin + USSR)
- Dark mode : native Starlight support
- Logo : pgpilot icon (HUD Lock design)
- DoD : Theme loads, dark mode toggles

### Critères de complétion
- [ ] Astro project created + runs locally
- [ ] i18n EN + FR routes active
- [ ] Starlight customized (theme, logo)

---

## Phase 15 : Migration doc mdbook → Astro (sem 26–27)

**Lead :** technical-writer  
**Sous-agents :** PM (review)  
**Dépendances :** Phase 14 ✅

### Objectif
Migrer contenu mdBook vers Astro + Starlight. Toutes les pages EN + FR re-rendues.

### Tâches

**T15.1** — Content migration (1.5j)
- Copier tous les .md files depuis `book/src/` → `docs/src/en/`
- Migrer structure : `SUMMARY.md` → Starlight `sidebar` config
- Frontmatter : ajouter `title`, `description`, `i18nReady: true`
- Fix image paths : `![](../../images/...)` → `![](/img/...)`
- DoD : Tous les fichiers migré, sans 404s

**T15.2** — French translation (1.5j)
- Traduire tous les .md files en français → `docs/src/fr/`
- Utiliser termes glossary (pgpilot, clé, chiffrement, signature, etc.)
- Proofreading : vérifier cohérence texte
- DoD : 100% EN + FR content present

**T15.3** — Fix markdownlinks & syntax (1j)
- Starlight table syntax : check `| col |` formatting
- Fix code blocks : ` ```rust` + language identifier
- Links : `[text](../path)` → Starlight astro imports if needed
- Admonitions : convert Starlight callouts syntax
- DoD : Build zero warnings

### Critères de complétion
- [ ] Tous les .md files migrés
- [ ] EN + FR content identical structure
- [ ] Liens + images OK
- [ ] Build sans warnings

---

## Phase 16 : Intégration screenshots & démo (sem 28)

**Lead :** technical-writer + ui-ux-tester  
**Sous-agents :** PM (validation)  
**Dépendances :** Phase 13 release candidate ✅, Phase 15 content ✅

### Objectif
Ajouter 15+ screenshots dans la doc, démo interactive, getting started guide.

### Tâches

**T16.1** — Screenshots (1j)
- Capture 15 screens : mykeys list, key detail, create key, import, encrypt, sign, verify, chat, settings
- Thème Catppuccin + USSR (2 per major view)
- Resize 800x600 (consistent)
- Annotate si besoin (arrows, highlights)
- Embed dans Astro markdown
- DoD : Tous les screenshots intégrées, chargement OK

**T16.2** — Getting started guide (1j)
- Installation : download binaire, verify signature
- First run : create key ou import
- Basic workflow : list keys, export public
- Chat setup : invite friend via join code
- Help links : where to report bugs, docs URL
- DoD : Guide complet EN + FR

**T16.3** — Interactive demo (optional, 0.5j)
- Storybook-style : embedded UI components demo
- Click to explore buttons, inputs, modals
- Theme switcher dans la page
- DoD : Optional, but nice-to-have for user engagement

### Critères de complétion
- [ ] 15+ screenshots intégrées
- [ ] Getting started guide complet
- [ ] Links + images vérifiées

---

## Phase 17 : Déploiement (GitHub Pages) (sem 29)

**Lead :** technical-writer (+ infra if needed)  
**Sous-agents :** PM (validation)  
**Dépendances :** Phase 16 ✅

### Objectif
Déployer doc site sur GitHub Pages. Domaine custom, auto-deploy CI.

### Tâches

**T17.1** — GitHub Actions CI (1j)
- `.github/workflows/docs.yml` :
  - Trigger : push to `main` (docs folder changes)
  - `npm install`, `npm run build`
  - Deploy artifact : `dist/` folder
  - GitHub Pages : auto-deploy to `gh-pages` branch
- DoD : Action workflows fonctionne, artifact deployed

**T17.2** — GitHub Pages settings (0.5j)
- Repo settings → Pages → Source = `gh-pages` branch
- Custom domain : configure CNAME si applicable (ou skip)
- HTTPS : enable automatic
- DoD : Site accessible via https://username.github.io/pgpilot (or custom domain)

**T17.3** — Site metadata (0.5j)
- sitemap.xml : auto-generated par Astro
- robots.txt : allow all, sitemap link
- Open Graph : title, description, image per page
- DoD : SEO basics configured

### Critères de complétion
- [ ] CI/CD pipeline fonctionnel
- [ ] Site accessible GitHub Pages
- [ ] HTTPS actif
- [ ] sitemap + robots.txt présents

---

## Phase 18 : Release & communication (sem 30)

**Lead :** PM  
**Sous-agents :** rust-engineer (binary), technical-writer (notes)  
**Dépendances :** Phase 13 RC ✅, Phase 17 doc deployed ✅

### Objectif
Lancer v0.8.0 publiquement. Binaires signés, annonce, notes de version.

### Tâches

**T18.1** — Create GitHub release (0.5j)
- Git tag : `v0.8.0`
- Binary uploads : `pgpilot-0.8.0-x64-linux.deb`, `.tar.gz`, `.AppImage` (si applicable)
- Sign binaries : cosign ou GPG signature
- RELEASE_NOTES.md : auto-generate depuis changelog (cliff)
- DoD : Release publié GitHub

**T18.2** — Announce (0.5j)
- Email announcement : users list (if applicable)
- Mastodon post : features, thème USSR, chat
- Twitter/X post : link to release
- Blog post (optional) : technical deep-dive Tauri migration
- DoD : Announcements live

**T18.3** — Checkups post-release (0.5j)
- Monitor : issues reported après release
- Prepare hotfix commits si CRITICAL bug found
- Feedback survey : "What did you think of v0.8.0?"
- DoD : Hotfix plan communiqué si needed

### Critères de complétion
- [ ] Release tag v0.8.0 créé
- [ ] Binaires signés
- [ ] GitHub Release publié
- [ ] Announcement sent

---

## Dépendances inter-phases

```
Phase 1 (Derisking)
├─ Phase 2 (Design tokens) ✅
├─ Phase 3 (Frontend arch) ✅
│  ├─ Phase 4 (Catppuccin CSS) ✅
│  │  └─ Phase 5 (USSR CSS) ✅
│  │     └─ Phase 6 (IPC) ✅
│  │        └─ Phase 7 (Master feature) ✅
│  │           └─ Phase 8 (Audit Tauri) ✅
│  │              └─ Phase 9 (Encrypt/Sign/Verify) ✅
│  │                 └─ Phase 10 (Chat) ✅
│  │                    └─ Phase 11 (QA/Tests) ✅
│  │                       ├─ Phase 12 (Beta) ✅
│  │                       │  └─ Phase 13 (Hotfixes) ✅
│  │                       │     ├─ Phase 16 (Screenshots) ✅
│  │                       │     │  └─ Phase 17 (Deploy docs) ✅
│  │                       │     │     └─ Phase 18 (Release)
│  │                       │     └─ Phase 18 (Release)
│  │
│  └─ Phase 14 (Astro setup) ✅ (can start week 4 in parallel)
│     └─ Phase 15 (Doc migration) ✅
│        └─ Phase 16 (Screenshots) ✅
│           └─ Phase 17 (Deploy) ✅
│              └─ Phase 18 (Release)
```

**Chemins critiques :**
1. **Tauri app (Phase 1–13) :** 24 semaines séquentielles (peut être parallélisé partiellement)
2. **Documentation (Phase 14–17) :** 5 semaines, peut démarrer Sem 4

**Parallélisation possible :**
- Phases 2 + 3 : en parallèle (respectivement design tokens + frontend scaff)
- Phases 4–5 : séquentielles (CSS themes, dépendent architecture Phase 3)
- Phases 7 + 8 : séquentielles (Phase 8 audit Phase 7 code)
- Phases 9–10 + 11 : peuvent chevaucher (QA lors de 9–10 completion)
- Phase 14 : démarrable dès Sem 4 (indépendant de phases UI)

---

## Notes & conventions

- Tous les commits incluent l'issue numéro : `#T7.1` (Phase 7, Task 1)
- PRs : code review minimum 1 agent (pas l'auteur)
- DoD = Definition of Done, vérifiée avant "Phase complète"
- Risques : réévaluées toutes les 2 semaines
- Blockers : communiqués au PM dans les 24h
