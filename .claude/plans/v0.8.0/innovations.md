# Innovations créatives — pgpilot v0.8.0+

Ce document propose des opportunités au-delà du scope v0.8.0, exploitant la liberté créative de Tauri + web frontend.

---

## 1. Tauri-native capabilities (Phase 1–18)

### 1.1 Notifications natives

**Opportunité :** Tauri v2 supporte `tauri::notification` API. pgpilot peut utiliser le système de notification OS.

**Cas d'usage :**
- Message chat incoming : notif "Alice: Hello!"
- Key published to keyserver : "Key ABC123 published ✓"
- Error alerts : "Import failed: invalid format"
- Background tasks : subkey rotation completed

**Implémentation (Phase 9) :**

```rust
// src-tauri/src/commands/notifications.rs
#[tauri::command]
pub async fn send_notification(title: String, body: String) -> Result<(), String> {
    tauri::api::notification::Notification::new("com.pgpilot")
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| e.to_string())
}
```

Frontend :

```typescript
import { invoke } from '@tauri-apps/api';

async function notifyKeyPublished(fp: string) {
  await invoke('send_notification', {
    title: 'Key published',
    body: `Fingerprint ${fp.slice(0, 16)}... is now on the keyserver.`,
  });
}
```

**Design + UX impact :** Notifications visibles hors-app, workflow asynchrone plus fluide.

---

### 1.2 System tray icon + quick actions

**Opportunité :** Afficher pgpilot dans la system tray (panel Linux) avec menu contextuel.

**Cas d'usage :**
- Quick encrypt : right-click selected file → pgpilot → encrypt (copy to clipboard)
- Quick sign : right-click file → pgpilot → sign
- Show/hide main window
- Quick access to active chat room
- Status badge : # messages unread

**Implémentation (Phase 10+) :**

```rust
// src-tauri/src/main.rs
use tauri::SystemTray;
use tauri::SystemTrayMenu;

fn setup_tray() -> SystemTray {
    let show = tauri::CustomMenuItem::new("show", "Show pgpilot");
    let encrypt = tauri::CustomMenuItem::new("encrypt", "Encrypt file...");
    let sign = tauri::CustomMenuItem::new("sign", "Sign file...");
    let quit = tauri::CustomMenuItem::new("quit", "Quit");
    
    let menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(encrypt)
        .add_item(sign)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(quit);
    
    SystemTray::new().with_menu(menu)
}
```

**Design impact :** Petite icône HUD Lock dans la tray, badges de notification.

---

### 1.3 Deep links / URL protocol handler

**Opportunité :** Enregistrer pgpilot comme handler pour `pgp://` URLs.

**Cas d'usage :**
- `pgp://keys/ABC123DEF456.../` → ouvrir pgpilot, afficher la clé
- `pgp://import?url=https://...` → auto-import from URL
- `pgp://chat/roomid` → rejoindre room chat
- Email client : clic sur une URL pgp:// → ouvre pgpilot

**Implémentation (Phase 18+) :**

```toml
# src-tauri/tauri.conf.json
{
  "tauri": {
    "protocol": {
      "assetScope": {
        "scope": ["pgp://**"]
      }
    }
  }
}
```

Frontend routing :

```typescript
// Check URL at app startup
const urlParams = new URLSearchParams(window.location.search);
const action = urlParams.get('action');
const target = urlParams.get('target');

if (action === 'import_key') {
  navigate('/import', { state: { url: target } });
} else if (action === 'chat') {
  navigate(`/chat/${target}`);
}
```

**Design impact :** Seamless integration entre email/browser et pgpilot.

---

### 1.4 Drag & drop file association

**Opportunité :** Permettre drag & drop de clés `.asc` / `.gpg` files directement dans pgpilot.

**Déjà implémenté (Phase 9)** mais améliorable :
- Ajouter handlers système pour fichiers `.asc` → auto-import
- Ajouter handlers système pour fichiers `.gpg` → auto-decrypt
- Association dans le .desktop file (Linux)

```ini
# pgpilot.desktop
[Desktop Entry]
Type=Application
Name=PGPilot
Exec=pgpilot %F
MimeType=application/pgp-keys;application/pgp-signature;application/pgp-encrypted;
```

---

## 2. Documentation & education

### 2.1 Interactive playground in Astro doc (Phase 15+)

**Opportunité :** Ajouter un "bac à sable" interactif dans la documentation.

**Cas d'usage :**
- Demo key creation (no real key, simulated)
- Encryption flow : recipient selector → show encrypted output
- Signature verification : upload .sig → show verification result
- Chat room preview : message timeline animation
- Theme switcher : live dark/light toggle

**Implémenté Phase 16** (screenshots), mais extensible en :

```astro
<!-- docs/src/en/guides/encrypt.md -->
---
title: Encrypt a file
---

## How to encrypt

Follow these steps:

<EncryptPlayground />  <!-- Interactive component -->

Or use the full app:
...
```

Component (React embedded in Astro) :

```tsx
// docs/src/components/EncryptPlayground.tsx
export default function EncryptPlayground() {
  const [recipients, setRecipients] = useState<KeyInfo[]>([]);
  const [encrypted, setEncrypted] = useState('');
  
  return (
    <div className="playground">
      <h3>Try it</h3>
      <RecipientSelector onChange={setRecipients} />
      <button onClick={() => simulateEncrypt(recipients)}>
        Encrypt
      </button>
      {encrypted && <CodeBlock code={encrypted} />}
    </div>
  );
}
```

**UX impact :** Users learn by doing, reduce learning curve.

---

### 2.2 Embedded video tutorials

**Opportunité :** Courtes vidéos (3–5 min) intégrées dans la doc.

**Topics :**
- "First steps: create a key" (2 min)
- "Import a key from a keyserver" (2 min)
- "Encrypt a file" (3 min)
- "Sign and verify" (3 min)
- "Join a chat room" (2 min)
- "YubiKey setup" (4 min)

**Implémenté Phase 16+** :

```astro
<!-- docs/src/en/guides/get-started.md -->
<video controls width="800">
  <source src="/videos/create-key.webm" type="video/webm" />
  Your browser doesn't support HTML5 video.
</video>
```

**Production workflow :**
- Screencast local dev build
- Cut + annotate (arrows, highlights)
- Export WebM (smaller than MP4)
- Host on GitHub Pages or CDN

---

### 2.3 API documentation auto-generated

**Opportunité :** Générer la doc des commandes Tauri depuis les types Rust.

**Implémentable Phase 6** avec tauri-specta :

```bash
# Generate OpenAPI spec from Tauri commands
tauri-specta --output docs/api-spec.json
```

Astro docgen :

```astro
---
// docs/src/en/api/commands.md
import specs from '@/api-spec.json';
---

# Tauri Commands

{specs.commands.map(cmd => (
  <div key={cmd.name}>
    <h2>{cmd.name}</h2>
    <pre>{JSON.stringify(cmd.schema, null, 2)}</pre>
  </div>
))}
```

**Developer experience :** Devs can reference API directly without manual docs.

---

## 3. UI/UX enhancements (v0.8.0+)

### 3.1 Animated transitions (Phase 4–5)

**Opportunité :** Web frontend supports smooth animations (CSS + React Spring).

**Cas d'usage :**
- Page transitions : fade-in, slide-left
- Master-detail panel : slide-out detail panel
- Chat messages : fade-in new messages with scale
- Theme switch : cross-fade Catppuccin → USSR
- Loading skeletons : pulsing placeholders

**Implémenté Phase 4–5** en CSS :

```css
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.view-enter {
  animation: fadeIn 300ms ease-out;
}
```

Or React Spring for advanced interactions :

```typescript
import { useTransition, animated } from '@react-spring/web';

function KeyDetail() {
  const transition = useTransition(showDetail, {
    from: { opacity: 0, x: 100 },
    enter: { opacity: 1, x: 0 },
    leave: { opacity: 0, x: -100 },
  });
  
  return transition((style, item) =>
    item && <animated.div style={style}>{...}</animated.div>
  );
}
```

---

### 3.2 Advanced typography (Phase 5)

**Opportunité :** Web allows advanced typography (variable fonts, OpenType features).

**USSR theme (already using Bebas Neue + Russo One) :**
- Variable fonts : weight 100–900 slider
- Ligatures : ffi, ffl, etc.
- Small caps : titles in CAPITALS
- Tabular numbers : fingerprints aligned

**Implémenté Phase 5** :

```css
@import url('https://fonts.googleapis.com/css2?family=Bebas+Neue:wght@400&display=swap');

.flavor-title {
  font-family: 'Bebas Neue', sans-serif;
  font-size: 2.5rem;
  font-feature-settings: 'smcp' 1; /* Small caps */
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.fingerprint {
  font-family: 'SF Mono', monospace;
  font-feature-settings: 'tnum' 1; /* Tabular numbers */
  letter-spacing: 0.1em;
}
```

---

### 3.3 Dark mode + theme transitions (Phase 5)

**Opportunité :** Native CSS dark mode + system preference detection.

**Implémentable Phase 5** :

```tsx
useEffect(() => {
  const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
  
  const handleChange = (e: MediaQueryListEvent) => {
    setTheme(e.matches ? 'ussr' : 'catppuccin');
  };
  
  darkModeQuery.addEventListener('change', handleChange);
  return () => darkModeQuery.removeEventListener('change', handleChange);
}, []);
```

**Design :** Auto-switch to USSR (dark) if system dark mode enabled.

---

## 4. Advanced cryptography features

### 4.1 Post-quantum crypto (v0.9+)

**Opportunité :** Dès que GnuPG 2.5 stable sort, ajouter support PQC.

**Implémentable Phase 21** (roadmap) :

```rust
// src/gpg/keyring.rs
pub enum SubkeyType {
    Sign,        // Ed25519
    Encr,        // cv25519
    Auth,        // Ed25519
    
    // v0.9+ : PQC
    SignPQC,     // Dilithium3 + Ed25519
    EncrPQC,     // ML-KEM-768 + cv25519
}
```

UI : algo selector in create key form :

```tsx
<fieldset>
  <legend>Signing algorithm</legend>
  <label><input type="radio" name="algo" value="ed25519" /> Ed25519</label>
  <label><input type="radio" name="algo" value="dilithium3" /> Dilithium3 (PQC)</label>
</fieldset>
```

---

### 4.2 Hardware security keys (FIDO2) (v0.9+)

**Opportunité :** Utiliser FIDO2 keys pour l'authentification (alternative YubiKey).

**Cas d'usage :**
- Login + 2FA : FIDO2 webauthn
- Sign files : touch YubiKey / FIDO2 key
- Decrypt : FIDO2 challenge (si clé support decryption)

**Requires :** `webauthn-rs` crate + browser WebAuthn API.

---

### 4.3 Encrypted backup in cloud (v0.9+)

**Opportunité :** Backup de clés secrètes dans le cloud (Nextcloud, S3, etc.).

**Cas d'usage :**
- Export secret key
- Encrypt with passphrase or dedicated backup key
- Upload to cloud provider
- Restore : download + decrypt + import

**Architecture :**
- Backend Rust : S3 / WebDAV integration
- UI : upload form + passphrase dialog
- Restore : download form + passphrase prompt

---

## 5. Community & social

### 5.1 Key sharing via QR code

**Opportunité :** Scanner/generate QR codes pour partager clés publiques.

**Implémentable Phase 7+** :

```typescript
import QRCode from 'qrcode';

async function generateQRCode(fp: string) {
  const pgpUrl = `pgp://keys/${fp}`;
  const qr = await QRCode.toDataURL(pgpUrl);
  return qr;
}
```

UI : Afficher QR code dans key detail, permettre scan via camera.

```tsx
<img src={qrCodeURL} alt="Key QR" />
<button>Scan QR code (camera)</button>
```

---

### 5.2 User profiles / trust network visualization

**Opportunité :** Afficher un graph visuel des connexions de confiance.

**Cas d'usage :**
- Afficher réseau : Alice → Bob (3 / full trust) → Charlie
- Distance : "Charlie is 2 steps from your web of trust"
- Trust path : click chain to validate signature

**Tech :** D3.js / Cytoscape.js pour graph rendering.

```tsx
import { Graph } from '@nivo/network';

export function TrustNetwork() {
  return (
    <Graph
      data={trustGraphData}
      linkComponent={TrustLink}
      nodeComponent={TrustNode}
    />
  );
}
```

---

### 5.3 Community keyserver explorer

**Opportunité :** Interface web pour explorer clés publiques (alternative WKD).

**Cas d'usage :**
- Search keys.openpgp.org directement depuis pgpilot
- Browse key popularity / signatures
- Afficher key stats : "100 people trust this key"

---

## 6. Developer tools

### 6.1 Built-in key debugger

**Opportunité :** Outil de diagnostic avancé pour les clés.

**Cas d'usage :**
- View binary key structure (packet format)
- Inspect subkey policies (signing capability, etc.)
- Check for weak algorithms (SHA-1 signatures)
- Validate key expiry + subkey expiry
- Show policy conflicts

**UI :** Advanced tab in key detail (health view).

---

### 6.2 GPG command builder

**Opportunité :** Interface pour construire des commandes GPG custom.

**Cas d'usage :**
- Power users : construct custom `gpg --encrypt` commands
- Debug : see exact command executed
- Learn : see how pgpilot translates UI to GPG

**UI :** Settings > Developer Tools > Command Builder.

```tsx
<form>
  <input placeholder="--recipient fp..." />
  <input placeholder="--sign-with fp..." />
  <button>Preview command</button>
  <code>{generatedCommand}</code>
</form>
```

---

### 6.3 Telemetry opt-in (v0.9+)

**Opportunité :** Opt-in anonymous telemetry pour améliorer UX.

**Data collected (user-approved) :**
- Features most used
- Errors encountered
- Performance metrics
- OS/device (no identifiers)
- No personal data

**Privacy-first :** Self-hosted analytics (Plausible ou Umami, not Google Analytics).

```rust
#[tauri::command]
pub async fn send_telemetry(event: String, data: serde_json::Value) -> Result<(), String> {
    if !should_send_telemetry() { return Ok(()); }
    
    reqwest::Client::new()
        .post("https://analytics.pgpilot.local/api/event")
        .json(&json!({ "event": event, "data": data }))
        .send()
        .await
        .ok();
    
    Ok(())
}
```

---

## 7. Mobile companion app (future)

### 7.1 Tauri Mobile roadmap (v0.9+)

**Opportunité :** Tauri v2 supports mobile (experimental iOS/Android).

**Cas d'usage :**
- Mobile key manager : view keys, manage trust
- Sign via YubiKey (OTG cable)
- Chat : receive notifications, reply quickly
- Verify signatures on mobile

**Challenges :**
- GPG binary availability on mobile
- YubiKey OTG support
- Storage (GNUPGHOME on mobile)

**Not v0.8.0, but interesting for v1.0.**

---

## 8. Accessibility innovations

### 8.1 Custom keyboard shortcuts

**Opportunité :** Power users = custom keybindings.

**Cas d'usage :**
- `Ctrl+Shift+E` → open encrypt
- `Ctrl+Shift+S` → open sign
- `Ctrl+Shift+L` → focus key list search
- `Ctrl+Shift+N` → new chat message

**Implémentable Phase 11+** :

```tsx
import { useHotkeys } from '@reeact-hotkeys-hook';

useHotkeys('ctrl+shift+e', () => navigate('/encrypt'));
useHotkeys('ctrl+shift+s', () => navigate('/sign'));
```

Settings : allow custom keybindings.

---

### 8.2 High contrast mode

**Opportunité :** Respect `prefers-contrast` media query.

```css
@media (prefers-contrast: more) {
  :root {
    --text-strong: #000;
    --detail-bg: #fff;
    --border: #000;
  }
}
```

---

### 8.3 Reduced motion mode

**Opportunité :** Respect `prefers-reduced-motion` for animations.

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## Summary : Innovation roadmap

| Innovation | Phase | Effort | Impact | Roadmap |
|-----------|-------|--------|--------|---------|
| **Notifications** | 9–10 | 1 day | High (UX) | v0.8.0+ |
| **System tray** | 10+ | 2 days | Medium | v0.9 |
| **Deep links** | 18+ | 1 day | Medium | v0.9 |
| **Interactive doc** | 16+ | 3 days | High (learning) | v0.8.0+ |
| **Video tutorials** | 16+ | 5 days | Medium | v0.8.0+ |
| **Animated transitions** | 4–5 | 1 day | Medium (polish) | v0.8.0 |
| **PQC support** | v0.9+ | 5 days | High (future) | v0.9 |
| **Cloud backup** | v0.9+ | 3 days | Medium | v0.9 |
| **QR codes** | v0.8.0+ | 2 days | Medium | v0.8.0+ |
| **Trust visualization** | v0.9+ | 5 days | High (UX) | v0.9 |
| **High contrast mode** | 11+ | 1 day | Low | v0.8.0+ |
| **Reduced motion** | 11+ | 1 day | Low | v0.8.0+ |

---

## Recommendation

**v0.8.0 (18 weeks)** : Focus sur la migration Tauri core + validation beta.

**v0.8.1–v0.9.0 (follow-up)** : Intégrer innovations créatives (notifications, tray, QR, animations).

**Post-v0.9 (long-term)** : PQC, mobile, cloud backup, advanced sharing.

Cette approche **délivre un MVP solide v0.8.0**, puis ajoute progressivement les features "wow" qui transformeront pgpilot en application grand public de référence.
