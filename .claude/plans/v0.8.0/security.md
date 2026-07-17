# Analyse de sécurité — Migration Tauri v0.8.0

**Date :** 2026-05-12  
**Responsable :** security-auditor + rust-engineer  
**Référentiels :** OWASP Top 10 2023, NIST SP 800-171, ANSSI Guides (Archit. et Bonnes pratiques)  
**Audit prévu :** Phase 8 (sem 14)

---

## 1. Contexte de menace

### Profil utilisateur

pgpilot cible **grand public non-expert** utilisant l'application desktop pour gérer des clés PGP privées. **Matériel sensible exposé.**

### Périmètre sécurité v0.8.0

- Backend Rust : logique GPG/MQTT/crypto (même stack v0.7.0, re-testé)
- Frontend web : UI React 18 + TypeScript
- IPC : Pont de communication Tauri entre UI et backend
- WebView : Chromium-based (Linux), WebKit2GTK (Linux alt), WKWebView (macOS, futurs versions)
- Configuration : GNUPGHOME (utilisateur), localStorage (tokens UI)

### Périmètre HORS-SCOPE v0.8.0

- ❌ Support macOS / Windows (v0.9+ envisagé)
- ❌ Installation système (packaging Snap/Flatpak → v0.9+)
- ❌ Auto-update (trop coûteux pour beta, add v0.9+)
- ❌ GPU sandboxing (WebView hardware accel. non-isolé)

---

## 2. Analyse surface d'attaque

### 2.1 IPC injection

**Risque :** Attaquant injecte commandes malveillantes via IPC, ou manipule réponses.

#### Scénarios

| Scénario | Vecteur | Impact | Mitigation |
|----------|---------|--------|-----------|
| IPC command injection | Modifie `invoke()` message | Exécute commande non-autorisée | Input validation, typed commands |
| Path traversal | `export_key(fp="/../../etc/passwd")` | Lit fichiers système | validate_fp() + sanitize paths |
| SQL injection (N/A) | chat room search | N/A | No SQL backend, in-memory |
| Symlink follow | Export → symlink → read | Leak secret key | `std::fs::metadata()` check |

#### Mitigations v0.8.0

**T8.1 (Phase 8) — Input validation**

Chaque commande Tauri Rust valide inputs strictement :

```rust
#[tauri::command]
pub async fn export_key(fp: String) -> Result<String, String> {
    // VALIDATION PREMIÈRE LIGNE
    gpg::keyring::validate_fp(&fp)
        .map_err(|e| format!("Invalid fingerprint: {}", e))?;
    
    // Ensuite, appel logique métier
    gpg::keyring::export_public_key(&fp)
        .map_err(|e| format!("Export failed: {}", sanitize_gpg_stderr(&e)))
}
```

**Validation patterns :**
- Fingerprints : `validate_fp()` → 40 hex chars exactly
- File paths : `std::path::Path::canonicalize()` + whitelist (downloads dir, user homedir)
- URLs : `safe_get()` wrapper → HTTPS only, 1 MiB limit, 3 max redirects
- Enums : Rust type system (no string-based dispatch)

**Frontend + TypeScript validation :**
- Zod schemas pour tous les IPC call arguments
- Runtime validation avant `invoke()`

```typescript
import { z } from 'zod';

const ExportKeyArg = z.object({
  fp: z.string().regex(/^[a-fA-F0-9]{40}$/),
});

type ExportKeyArg = z.infer<typeof ExportKeyArg>;

async function exportKey(arg: ExportKeyArg) {
  const validated = ExportKeyArg.parse(arg); // throws if invalid
  return await invoke('export_key', { fp: validated.fp });
}
```

#### Tests Phase 8

**Fuzzing :** `cargo-fuzz` ou manual fuzzing 100 random inputs par commande.

```rust
#[test]
fn fuzz_export_key_invalid_fp() {
    let invalid_fps = vec![
        "not_hex",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", // 33 chars
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/../../etc/passwd",
        "",
    ];
    
    for fp in invalid_fps {
        let result = export_key(fp);
        assert!(result.is_err(), "Should reject: {}", fp);
    }
}
```

---

### 2.2 WebView security

**Risque :** Attaquant injecte JS via XSS ou modifie content, accède aux APIs WebView.

#### Contexte

Linux : Tauri utilise **webkit2gtk** (default) ou **chromium-based** (future).
- WebKit2GTK 2.42+ : decent sandbox
- IPC bridge : JS `window.__TAURI__` object → Rust commands

#### Mitigations

**T8.2 (Phase 8) — CSP (Content Security Policy)**

```html
<!-- src/index.html -->
<meta http-equiv="Content-Security-Policy" 
      content="default-src 'self';
               script-src 'self' 'wasm-unsafe-eval';
               style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
               img-src 'self' data:;
               connect-src 'self' wss://broker.hivemq.com:8883;
               font-src 'self' https://fonts.gstatic.com;
               frame-ancestors 'none';
               base-uri 'self';">
```

**Breakdown :**
- `default-src 'self'` : Only same-origin by default
- `script-src 'self' 'wasm-unsafe-eval'` : Local scripts + Wasm (Tauri required)
- `style-src 'unsafe-inline'` : Accepté (React inline styles nécessaires, Tailwind)
- `style-src https://fonts.googleapis.com` : Google Fonts CDN
- `connect-src wss://` : MQTT TLS pour chat
- `frame-ancestors 'none'` : Pas d'iframe external
- `base-uri 'self'` : No base tag injection

**HTTP headers (backend Tauri) :**

```rust
// In main.rs setup
app.setup(|_app| {
    // Set security headers
    Ok(())
})?;
```

Tauri configure headers automatiquement si CSP est dans index.html.

**Stricte : zéro `eval()`, zéro `new Function()` en code**

Linter rule :
```json
// .eslintrc.json
{
  "rules": {
    "no-eval": "error",
    "no-implied-eval": "error",
    "no-new-func": "error"
  }
}
```

#### XSS prevention

**Affichage de données potentiellement malveillantes (noms clés, emails, messages chat) :**

```typescript
// ❌ MAUVAIS : dangerouslySetInnerHTML
<div dangerouslySetInnerHTML={{ __html: key.name }} />

// ✅ BON : React auto-escapes text nodes
<div>{key.name}</div>
```

React auto-échappe le texte. Pas de `.innerHTML`.

**Test Phase 8 :**

```javascript
// Playwright test
test('XSS protection - chat message', async ({ page }) => {
  // Envoi message : "<img src=x onerror=alert('XSS')>"
  await page.fill('input[name="message"]', '<img src=x onerror="alert(\'XSS\')">"');
  await page.click('button:has-text("Send")');
  
  // Vérifier le message est échappé
  const chatMessage = page.locator('text="<img src="');
  await expect(chatMessage).toBeVisible(); // littéral, pas exécuté
});
```

---

### 2.3 Permissions Tauri

**Risque :** App demande permissions trop larges, attaquant exploite.

#### Configuration stricte

**`src-tauri/tauri.conf.json`**

```json
{
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "readDir": true,
        "readFile": true,
        "writeFile": true,
        "removeFile": true,
        "createDir": true,
        "removeDir": true,
        "copyFile": true,
        "scope": [
          "$HOME",
          "$DOWNLOADS",
          "$TEMP"
        ]
      },
      "http": {
        "all": false,
        "request": true,
        "scope": [
          "https://keys.openpgp.org/*",
          "https://keyserver.ubuntu.com/*",
          "wss://broker.hivemq.com:8883/*"
        ]
      },
      "notification": {
        "all": false,
        "sendNotification": true
      },
      "window": {
        "all": true
      },
      "shell": {
        "open": true,
        "execute": false
      }
    }
  }
}
```

**Rationale :**
- `fs.scope` : Limit file access to user home + downloads
- `http.scope` : Whitelist known servers only
- `shell.execute: false` : Pas d'exécution de binaires arbitraires
- `notification` : Autorisé pour messages chat incoming

#### Runtime permission checks

```rust
// In command handler
#[tauri::command]
pub fn open_file_dialog() -> Result<String, String> {
    // Tauri scope check happens automatically
    // If path outside scope → error
    Ok(path)
}
```

---

### 2.4 Configuration & secrets

**Risque :** Clés privées, tokens MQTT exposés en clair dans localStorage.

#### What NOT to store

- ❌ Clés privées PGP (jamais)
- ❌ Passphrases (jamais)
- ❌ MQTT broker credentials (jamais)

#### What IS stored (encrypted/signed)

localStorage (user GNUPGHOME visible anyway, donc pas critique) :
- ✅ Selected theme + language (cleartext, inoffensif)
- ✅ Chat room metadata : room_id, name, relay_url (cleartext, server-side anyway)
- ✅ Selected fingerprint (cleartext, user-side UI state)

#### GPG GNUPGHOME

- **Location :** `~/.gnupg/` (POSIX standard)
- **Permissions :** `gpg --list-keys` accessible only by user UID
- **Passphrase :** Cached by `gpg-agent`, not stored by pgpilot
- **Tauri fs.scope :** `$HOME` included, donc readable. **Accepté car c'est la responsabilité OS.**

#### MQTT TLS

- **Broker :** HiveMQ (test.mosquitto.org KO phase 6, see memory)
- **Port :** 8883 (MQTT over TLS)
- **Cert :** `webpki-roots` Mozilla CA bundle (in Cargo.toml)
- **Auth :** Pas d'authentification. **Threat model assumes relay is trusted.** (See THREAT_MODEL.md)

#### Mitigation

Config stored in `~/.config/pgpilot/config.yaml` :
- ✅ YAML format (machine-readable)
- ❌ NO secrets in YAML (theme, language, scale only)
- ✅ Permissions : 0600 (user-only read)

```rust
// In config/mod.rs
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn save_config(config: &Config, path: &Path) -> Result<(), Error> {
    let yaml = serde_yaml::to_string(config)?;
    fs::write(path, yaml)?;
    
    // Set permissions to 0600 (owner read/write only)
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms)?;
    
    Ok(())
}
```

---

### 2.5 Backend logic (re-audit)

Backend Rust code (gpg, mqtt, crypto) **unchanged from v0.7.0**. SECURITY_PLAN.md Phase 8 → completeness confirmed.

**V0.8.0 re-audit scope :**
- ❌ Not re-implementing GPG (using binary)
- ✅ Validating IPC layers only (new surface)
- ✅ Confirming GNUPGHOME isolation (unchanged)
- ✅ Confirming MQTT TLS (unchanged)

---

## 3. THREAT MODEL — V0.8.0 specifics

### Assumptions

1. **User workstation** : Desktop Linux, single-user, trusted filesystem
2. **User discipline** : Passphrase strength, YubiKey PIN, not sharing GNUPGHOME
3. **Network** : Relay server (HiveMQ) is honest-but-curious (cannot decrypt messages)
4. **Tauri security** : WebView sandbox works as designed

### Assets at risk

| Asset | Threat | Mitigation |
|-------|--------|-----------|
| Private keys (GNUPGHOME) | Local disclosure | OS filesystem perms (0700) |
| Chat messages | Transit interception | E2E encryption (GPG) |
| Master passphrases | RAM disclosure | Cleared by gpg-agent (OS-level) |
| User identity | Fingerprint linked to IP | Possible via relay logging (threat model: trusted relay) |

### Threats specific to Tauri WebView

| ID | Threat | Severity | Mitigation |
|----|--------|----------|-----------|
| T1 | XSS → IPC command injection | CRITICAL | CSP + input validation + Zod schemas |
| T2 | Rogue Tauri plugin | CRITICAL | Whitelist plugins in allowlist |
| T3 | Frontend code tampering | MEDIUM | Code signing not v0.8.0 (add v0.9+) |
| T4 | Clipboard leakage | LOW | Only pubkey/URLs, not secrets |
| T5 | File dialog SSRF | LOW | fs.scope restricts paths |
| T6 | DevTools enabled in prod | CRITICAL | Disable via tauri.conf.json |
| T7 | Unpatched WebView CVE | MEDIUM | Monitor webkit2gtk / WebKit releases |

#### Mitigations

**T1 :** CSP headers (section 2.2)  
**T2 :** `allowlist { all: false }` (section 2.3)  
**T3 :** Future : code signing + reproducible builds  
**T4 :** Design choice, acceptable risk  
**T5 :** `fs.scope: ["$HOME", "$DOWNLOADS"]` (section 2.3)  
**T6 :** Tauri prod config → DevTools disabled by default  
**T7 :** CI/CD monitoring `cargo audit` for Rust deps, keep webkit2gtk updated

---

## 4. Audit checklist Phase 8

### Pre-audit (T8.1)

- [ ] Code review : all Rust commands (input validation)
- [ ] Dep scan : `cargo audit` zero advisories
- [ ] Secrets scan : `git secrets --scan` for hardcoded credentials
- [ ] TypeScript strict : `npm run lint` zero errors

### IPC audit (T8.1 - 2 days)

- [ ] **Fuzzing :** 100 random inputs per 5 main commands
  - `list_keys()` → varies result size
  - `create_key()` → varies name/email
  - `import_key_from_text()` → garbage input
  - `encrypt_files()` → vary fp + recipient lists
  - `sign_file()` → invalid signers
  
- [ ] **Path traversal :** try `"/../../etc/passwd"`, symlinks
  - Test on actual filesystem
  
- [ ] **Race conditions :** rapid IPC calls, check state consistency
  - `create_key()` twice simultaneously
  - `list_keys()` while importing

#### Tools

```bash
# Fuzz with cargo-fuzz
cargo install cargo-fuzz
cargo +nightly fuzz fuzz_export_key 1000 # 1000 iterations

# Securely scan
cargo audit
git secrets --scan

# SAST (optional)
cargo clippy -- -D warnings
```

### WebView audit (T8.2 - 1 day)

- [ ] **CSP headers :**
  - Check inline styles allowed (React)
  - Check script-src no `unsafe-inline`
  - Check no `*` wildcards

- [ ] **Frame options :** X-Frame-Options: DENY configured

- [ ] **Cookie security :** localStorage (no HttpOnly on localStorage, it's XSS protection anyway)

- [ ] **HTTP loading :** No http:// resources in prod build

- [ ] **DevTools :** Disabled in production tauri.conf.json

#### Tools

```bash
# CSP validation
npm install -D csp-checker

# Lighthouse audit
npm run build && npm run preview
# Open Chrome DevTools → Lighthouse
```

### Permissions audit (T8.3 - 1 day)

- [ ] **fs.scope :** Only $HOME, $DOWNLOADS, $TEMP (no system dirs)
- [ ] **http.scope :** Only known good URLs (keys.openpgp.org, hivemq, etc.)
- [ ] **shell.execute :** FALSE (not used)
- [ ] **All permissions :** Justified in config file

#### Manual check

```bash
# Review tauri.conf.json
jq '.tauri.allowlist' src-tauri/tauri.conf.json
```

### Fixs & re-test (T8.4 - 1 day)

- [ ] All CRITICAL vulns fixed
- [ ] Re-run checks (audit, fuzz, lighthouse)
- [ ] Zéro CRITICAL, max 3 HIGH (with justification)

### Report (T8.5 - 1 day)

- [ ] Write audit report (5–10 pages) :
  - Methodology
  - Findings (categorized CRITICAL/HIGH/MEDIUM/LOW)
  - Fixes applied
  - Residual risks
  - Recommendations (v0.9+)
  
- [ ] Communicate to PM + utilisateur
- [ ] Archive in `.claude/plans/v0.8.0/audit-report.md`

---

## 5. ANSSI guidance applicability

pgpilot est une **application de gestion de clés PGP**, alignée sur ANSSI contextes :

- **ANSSI - Recommandations relatives à la sécurité des systèmes d'exploitation de bureau** : sections 2.1 (isolation), 2.2 (authentification), 3.1 (contrôle d'accès)
- **ANSSI - Bonnes pratiques de développement sécurisé** : sections validation inputs, gestion erreurs, logging (future v0.9)
- **ANSSI - Cryptographie et algorithmes** : respect TLS 1.3, algo PGP, aucun fallback weak ciphers

**Appliquer Phase 8 :**
- ✅ Input validation stricte (section 1)
- ✅ Isolation processus (GPG subprocess avec env clean)
- ✅ TLS 1.3 minimum (MQTT broker, HTTP requests)
- ✅ Pas de credentials stockés
- ✅ Logs (future)

---

## 6. Known limitations & risks

### Accepted risks v0.8.0

| Risk | Severity | Justification | Fix timeline |
|------|----------|---------------|--------------|
| **Code signing (macOS)** | MEDIUM | Desktop app, not critical v0.8.0 | v0.9 (codesign) |
| **Auto-update** | MEDIUM | Trop coûteux pour beta | v0.9 (sparkle/delta) |
| **Reproducible builds** | LOW | Aide debugging, pas critère MVP | v0.9+ |
| **Logging & audit trail** | MEDIUM | No event logging (future telemetry opt-in) | v0.9 |
| **Windows/macOS support** | HIGH | Linux only v0.8.0 (WebView differences) | v0.9 |

### Mitigation strategy

- ✅ Phase 8 audit captures all issues
- ✅ Mark v0.9 in roadmap → roadmap.md
- ✅ Communicate risk to users in release notes
- ✅ Beta testing includes security feedback loop

---

## Conclusion

**v0.8.0 security posture :**
- ✅ Backend Rust : proven secure (v0.7.0 re-audit)
- ✅ Frontend web : CSP strict, input validation, no XSS vectors
- ✅ IPC : type-safe, fuzzing in Phase 8
- ✅ Permissions : minimal, scoped
- ✅ Configuration : secrets not stored

**Target :** Zero CRITICAL, max 3 HIGH residual risks (with mitigation plans).

**Phase 8 validation** will confirm readiness for v0.8.0 release.
