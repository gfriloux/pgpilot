# Choix techniques — pgpilot v0.8.0

Les choix IPC, CSS, mock, Nix et architecture s'appuient sur **sshive** (`../sshive`),
une app Tauri v2 fonctionnelle sur le même environnement NixOS/Wayland.
Le framework frontend est **React 18** — meilleur écosystème design et plus de marge
de manœuvre pour les sous-agents UI/frontend.

---

## 1. Framework frontend : React 18 + TypeScript

**Décision : React 18 + TypeScript strict**

React gagne sur les critères importants pour ce projet :

- **Écosystème design** : Radix UI, Shadcn/ui, Storybook mature — les sous-agents UI ont plus de latitude créative qu'avec Svelte
- **Composants disponibles** : bibliothèques de composants testés (datepickers, modals, combobox accessibles) évitent de tout recoder
- **TypeScript intégration** : JSX + TS strict, outillage (ESLint, Prettier) très mature
- **Tauri** : documentation et exemples React + Tauri abondants

Svelte 5 est utilisé dans sshive et tourne bien, mais React offre plus de puissance
d'expression aux sous-agents design pour le niveau de polish grand public visé.

**TypeScript config stricte :**
```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitReturns": true,
    "noUncheckedIndexedAccess": true
  }
}
```

Zéro `any` en code applicatif.

---

## 2. Bundler : Vite

Confirmé par sshive. Vite est le template officiel Tauri, HMR instantané, config minimale.

Port dev : **1421** (repris de sshive).

```javascript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: { port: 1421, strictPort: true },
  build: { target: 'es2020' },
});
```

---

## 3. Système de thème CSS : variables dans un fichier global

Confirmé par sshive — un fichier `app.css` avec variables CSS, switch de thème via
classe `:root.theme-ussr`. Pas de Tailwind, pas de CSS-in-JS.

```css
/* src/styles/theme.css — Catppuccin Frappé (défaut) */
:root {
  --sidebar-bg:     #303446;
  --detail-bg:      #292c3c;
  --card-bg:        #363a4f;
  --border:         #49506a;
  --text-strong:    #e5e7eb;
  --text-secondary: #a8aac5;
  --text-muted:     #6c7086;
  --text-header:    #c6d0f5;
  --accent:         #ca9ee6;
  --accent-hover:   #d4a8e6;
  --success:        #a6e3a1;
  --error:          #e74c3c;
  --warning:        #f9ca24;

  --font-body:    system-ui, sans-serif;
  --font-mono:    "JetBrains Mono", monospace;
  --font-nav:     "Bebas Neue", sans-serif;
  --font-heading: "Russo One", sans-serif;

  --radius-sm: 6px;
  --radius-md: 10px;
  --transition-normal: 200ms ease;
}

:root.theme-ussr {
  --sidebar-bg:       #0f0d09;
  --detail-bg:        #f0ebd8;
  --card-bg:          #f0ebd8;
  --border:           #d9d3bc;
  --text-strong:      #1a1815;
  --text-secondary:   #4a4742;
  --text-muted:       #7a7370;
  --text-header:      #0f0d09;
  --accent:           #d82c20;
  --accent-hover:     #c2251a;
  --sidebar-text:     #f0ebd8;
  --sidebar-hover-bg: rgba(216, 44, 32, 0.1);
  --success:          #5a8a3a;
}
```

Switch thème = `document.documentElement.classList.toggle('theme-ussr')`. Instantané, zéro re-render React.

---

## 4. State management : Zustand

React n'a pas de réactivité native — Zustand est l'option la plus légère et suffisante
pour ~20 slices de state (ui, keys, config, chat).

```typescript
// src/store/keys.ts
import { create } from 'zustand';

interface KeysStore {
  keys: KeyInfo[];
  loading: boolean;
  selectedFp: string | null;
  setKeys: (keys: KeyInfo[]) => void;
  selectKey: (fp: string | null) => void;
}

export const useKeysStore = create<KeysStore>((set) => ({
  keys: [],
  loading: false,
  selectedFp: null,
  setKeys: (keys) => set({ keys, loading: false }),
  selectKey: (fp) => set({ selectedFp: fp }),
}));
```

Redux Toolkit serait overkill. Pas de Context API globale (performance).

---

## 5. IPC Tauri : invoke direct

Inspiré de sshive — types JSON définis à la main, suffisants pour la taille du projet.
Pas de tauri-specta (complexité de build inutile ici).

**Rust (backend) :**
```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct KeyInfo {
    pub fingerprint: String,
    pub name: String,
    pub email: String,
    pub created: String,        // "YYYY-MM-DD"
    pub expiry: Option<String>,
    pub has_secret: bool,
    pub trust: String,          // "undefined" | "marginal" | "full" | "ultimate"
}

#[tauri::command]
async fn list_keys() -> Result<Vec<KeyInfo>, String> {
    gpg::keyring::list_keys().map_err(|e| e.to_string())
}
```

**Frontend React :**
```typescript
// src/ipc/keys.ts
import { invoke } from '@tauri-apps/api/core';
import type { KeyInfo } from '../types';

export const listKeys = (): Promise<KeyInfo[]> =>
  invoke('list_keys');
```

Types TypeScript écrits à la main dans `src/types/` — source de vérité Rust, copie manuelle TS.
Validation d'entrée côté Rust en tête de chaque commande (validate_fp, validate_name…).

---

## 6. Mock Tauri pour dev / screenshots / doc interactive

Pattern repris de sshive. Alias Vite conditionnel remplace `@tauri-apps/api/core`
par un module mock local.

```javascript
// vite.config.ts
const isMock = process.env.VITE_MOCK === 'true';

export default defineConfig({
  resolve: {
    alias: isMock ? {
      '@tauri-apps/api/core': path.resolve('./src/lib/mock-tauri.ts'),
    } : {},
  },
});
```

```typescript
// src/lib/mock-tauri.ts
export async function invoke(cmd: string, _args = {}) {
  await new Promise(r => setTimeout(r, 60)); // simule latence IPC
  if (cmd === 'list_keys') return MOCK_KEYS;
  throw new Error(`mock: unknown command ${cmd}`);
}
```

**Usage :**
```bash
VITE_MOCK=true npm run dev    # frontend seul dans le navigateur
VITE_MOCK=true npm run build  # build statique pour la doc interactive
```

**Doc interactive :** ce build statique s'embarque dans la doc Astro — le visiteur
navigue dans pgpilot dans son navigateur sans installer quoi que ce soit.

---

## 7. Tests E2E : Playwright

Playwright supporte Tauri via un custom launcher, DX excellente.

Pour la CI et la génération de screenshots, on utilise le mode mock (comme sshive) :
Playwright + `VITE_MOCK=true` = pas de binaire Tauri requis dans la CI.

```typescript
// scripts/screenshots.ts
import { chromium } from 'playwright';
// Lance Vite en mode mock, prend 15+ screenshots, exporte dans docs/screenshots/
```

---

## 8. Routing : React Router v6

```typescript
// src/router.tsx
<Routes>
  <Route path="/" element={<Layout />}>
    <Route index element={<KeyList />} />
    <Route path="import" element={<Import />} />
    <Route path="encrypt" element={<Encrypt />} />
    <Route path="sign" element={<Sign />} />
    <Route path="verify" element={<Verify />} />
    <Route path="chat" element={<ChatList />} />
    <Route path="chat/:roomId" element={<ChatRoom />} />
    <Route path="settings" element={<Settings />} />
    <Route path="health" element={<Health />} />
  </Route>
</Routes>
```

---

## 9. Dépendances Nix (Tauri v2 sur NixOS/Wayland)

Directement repris de `sshive/shells/default/default.nix` — éprouvé.

```nix
buildInputs = with pkgs; [
  dbus openssl glib glib-networking
  gtk3 webkitgtk_4_1 libsoup_3
  cairo pango gdk-pixbuf atk librsvg
  xdotool
];

shellHook = ''
  export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
    dbus openssl glib gtk3 webkitgtk_4_1 libsoup_3
    cairo pango gdk-pixbuf atk librsvg
  ]}:$LD_LIBRARY_PATH
'';
```

---

## 10. Architecture Rust

Même split que sshive : logique métier dans `src/` (lib Rust existante), glue Tauri
dans `tauri-app/src-tauri/src/lib.rs`.

Les commandes Tauri sont de fines couches qui appellent les fonctions de `src/gpg/`,
`src/chat/`, `src/config/`. Aucun `#[tauri::command]` dans la logique métier.

---

## 11. npm — dépendances production

```json
{
  "react": "^18.3",
  "react-dom": "^18.3",
  "react-router-dom": "^6.24",
  "zustand": "^5.0",
  "@tauri-apps/api": "^2.0"
}
```

Pas de composants UI tiers en production — les composants sont écrits sur mesure
pour correspondre exactement aux deux thèmes. Radix UI primitives (accessibilité) en option.

---

## Tableau de décision final

| Sujet | Décision | Justification |
|-------|----------|---------------|
| Framework frontend | **React 18 + TypeScript** | Écosystème design, sous-agents UI |
| Bundler | **Vite 6** | Standard Tauri, éprouvé sshive |
| CSS themes | **Variables CSS globales** | Runtime switch, éprouvé sshive |
| IPC | **invoke direct** | Simple, suffisant, éprouvé sshive |
| State | **Zustand** | Minimal, performant pour ~20 slices |
| Routing | **React Router v6** | Standard React |
| Tests | **Playwright + VITE_MOCK** | CI sans binaire Tauri, éprouvé sshive |
| Nix deps | **webkitgtk_4_1 + gtk3 + libsoup_3** | Éprouvé sshive |
| Doc interactive | **Build mock embarqué dans Astro** | Pattern sshive adapté |
