# Rapport d'audit sécurité — pgpilot v0.8.0

## Résumé exécutif

| Sévérité | Nombre |
|----------|--------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 6 |
| LOW | 7 |
| Informatif | 5 |

**Verdict : ACCEPTABLE pour publication v0.8.0.** Zéro CRITICAL. La surface IPC (36 commandes) est globalement bien défendue. Voir findings détaillés ci-dessous.

## Findings prioritaires

### HIGH — H-01 : `import_key_from_keyserver` URL malformée + absence de whitelist host

**Impact :** SSRF/MitM si validation frontend contournée ; bug fonctionnel certain dans le path email (URL `https://https://...`).

**Remédiation :**
1. `validate_keyserver_url(url)` → parse URL, rejette schéma ≠ `https`, whitelist hôtes (`keys.openpgp.org`, `keyserver.ubuntu.com`)
2. Corriger la concaténation `format!("https://{keyserver_url}/...")` → utiliser l'URL canonique après validation
3. Appliquer la même whitelist dans `publish_key` (voir M-01)

### MEDIUM — M-01 : `publish_key` : `keyserver_url` passé verbatim à `gpg --keyserver`
**Remédiation :** même whitelist que H-01

### MEDIUM — M-02 : `chat_create_room` : `relay` et `my_fp` non validés
**Remédiation :** `parse_relay_url(&relay)?` + `validate_fp(&my_fp)?` + vérif keyring local + borner `name` à 256 octets

### MEDIUM — M-03 : `chat_add_participant` : `participant_fp` sans validate_fp()
**Remédiation :** ajouter `validate_fp(&participant_fp)?` en tête de commande

### MEDIUM — M-04 : `backup_key` : `dest_dir` sans canonicalisation ni vérification symlink
**Remédiation :** `canonicalize()` + `is_dir()` + `chmod 0o600` sur le fichier écrit

### MEDIUM — M-05 : `export_public_key_to_file` : écrasement silencieux
**Remédiation :** utiliser `create_new(true)` ou boucle anti-collision comme `encrypt_files`

### MEDIUM — M-06 : `create_key` : `name`/`email` non bornés ni filtrés (octets de contrôle)
**Remédiation :** borner name ≤ 64 octets, email ≤ 254 octets ; rejeter `\n \r \0 < >` ; valider email regex

### LOW (7 findings — voir rapport complet)
L-01 `style-src 'unsafe-inline'` CSP, L-02 `import_key_file` chemins non bornés, L-03 `publish_key` sans contre-vérification, L-04 ACKs chat sans validation d'appartenance, L-05 `sig_file` symlink, L-06 pas de rate-limiting `chat_join_room`, L-07 `mock-tauri.ts` dans `src/`

## Points positifs confirmés

- Zéro `dangerouslySetInnerHTML`, `eval()`, `new Function()` dans le frontend
- `gpg_command()` utilise `env_clear()` + `Command::args()` → pas d'injection shell
- `safe_get()` centralisé : HTTPS-only, 1 MiB, 3 redirects max, timeouts
- CSP stricte (`script-src 'self'`, pas de `'unsafe-eval'`, pas de `*`)
- `withGlobalTauri: false` → bridge non exposé globalement
- Capabilities minimales (`core:default`, `dialog:allow-open`, `dialog:allow-save`)
- Chat : `wire.sender ∈ room.participants`, `verified.signer_fp == wire.sender`, signature VALIDSIG sur clé maître, MAX_WIRE_MESSAGE_BYTES=64KiB côté émetteur ET récepteur
- `parse_relay_url()` refuse `mqtt://` vers hôtes non-locaux

## Scans automatisés (à exécuter avant release)

```bash
nix develop --command sh -c 'cargo audit'
nix develop --command sh -c 'cd tauri-app/src-tauri && cargo audit'
nix develop --command sh -c 'cd tauri-app && npm audit --audit-level=moderate'
nix develop --command sh -c 'cd tauri-app && npm run lint'
nix develop --command sh -c 'cd tauri-app && npm run build && grep -c "mock-tauri" dist/assets/*.js'  # → 0
```

## Recommandations v0.9+

1. Whitelist Rust-side des keyservers (H-01, M-01)
2. `validate_path()` centralisé pour tous les arguments path (M-04, M-05, L-02, L-05)
3. Validation `validate_fp()` systématique dans toutes les commandes chat
4. Borner toutes les `String` IPC par longueur documentée
5. Restreindre CSP à `style-src 'self'` après refactor styles inline
6. Déplacer `mock-tauri.ts` hors de `src/` (ex: `__mocks__/`)
7. `cargo-audit` bloquant en CI
8. Pen-test binaire packagé avant distribution v0.9
9. Journalisation d'audit minimale (`~/.local/share/pgpilot/audit.log`)
10. `check_keyserver` post-publication pour confirmer propagation
