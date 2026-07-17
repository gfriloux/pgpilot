# Plan v0.8.9

## Contexte

Point de départ : mises à jour Renovate (deps npm + Rust). Des régressions sont apparues.
Une session précédente a introduit du code sans plan — certains hacks doivent être supprimés.

## Objectifs

1. **File pickers natifs** — tous les boutons qui manipulent des fichiers utilisent `@tauri-apps/plugin-dialog` (comme Encrypt/Decrypt/Sign/Verify le font déjà).
2. **Deps à jour** — appliquer les mises à jour Renovate (npm + Rust).
3. **Base propre** — build vert, clippy vert, tests automatisés verts, plan de tests manuels rédigé, Nix build vérifié.

## Périmètre

### In scope
- Export clé publique (MyKeys + PublicKeys via KeyDetail) → `save()` dialog natif
- Backup clé secrète (MyKeys) → `open({ directory: true })` dialog natif
- MAJ deps npm : `@tauri-apps/cli`, `@types/react`, `@vitejs/plugin-react`, `react-router-dom`, `vite`, `typescript-eslint`, `eslint`, `lucide-react`, storybook
- MAJ deps Rust : `serde_json`, `tauri`, `tauri-build`
- Fix compilations et tests cassés par les MAJ
- Rédaction du plan de tests manuels v0.8.9
- Release Nix : recompute `npmDepsHash` + vérification `nix build`

### Out of scope
- SECURITY_PLAN.md : document interne, ne pas commiter
- Import clé via fichier (déjà accessible via Import page)
- Revocation cert : GPG gère le chemin (`openpgp-revocs.d/`), pas de file picker
- Nouvelles features

---

## Convention tests (s'applique à toutes les versions)

### Automatisés (Playwright E2E + Rust)
Tout ce qui peut être testé sans intervention humaine doit l'être.
En mode `VITE_MOCK=true`, les dialogs natifs sont remplacés par `mock-dialog.ts`
(retourne un chemin fictif), ce qui permet de tester le flux complet sauf l'ouverture
physique du picker.

**Ce qu'on automatise :**
- Comportement UI après sélection d'un fichier (toast, état bouton, message d'erreur)
- Logique de trust / chips dans Encrypt
- Flux import, encrypt, sign, verify en mock mode
- Tests Rust unitaires (lib)

**Ce qu'on ne peut pas automatiser :**
- Ouverture physique du file picker natif (KDE Plasma)
- Interaction avec un vrai keyring GPG (sauf les tests `--ignored` avec gpg réel)
- Build Nix complet
- Comportement sur machine réelle (résolution, thème, pinentry)

### Plan de tests manuels par version
Pour chaque version, un fichier `manual_tests.md` est rédigé dans le dossier du plan.
Il est écrit **au fur et à mesure du développement**, enrichi à chaque phase qui ajoute
un comportement utilisateur. Il est **exécuté par l'utilisateur** pendant la validation finale.

Convention du fichier :
```
.claude/plans/v{X.Y.Z}/manual_tests.md
```

---

## État du working tree au démarrage

Des changements ont été faits sans plan dans la session précédente.
Certains sont à **conserver**, d'autres à **supprimer** car remplacés par les file pickers natifs.

### À conserver
| Fichier | Raison |
|---------|--------|
| `Cargo.toml`, `app/package.json`, `app/src-tauri/Cargo.toml`, `app/src-tauri/tauri.conf.json`, `packages/pgpilot/default.nix` | Version bump → 0.8.9 |
| `.github/workflows/release.yml` | Fix `git checkout main` avant push |
| `app/src/layout/AppLayout.tsx` + `.module.css` | Système toast global |
| `app/src/lib/mock-event.ts` | Mock `listen()` pour les tests E2E (chat events) |
| `app/src/lib/mock-dialog.ts` | Mock dialog (à compléter avec `save()`) |
| `app/vite.config.ts` | Alias `@tauri-apps/api/event` → mock-event.ts (à garder) |
| `tests/gpg_keyring.rs` | Fix `GNUPGHOME` + `IsTerminal` |
| `app/src/pages/Encrypt.tsx` | Fix trust 'full' → `--trust-model always` |
| `app/src/pages/PublicKeys.tsx` | `key={selectedKey.fingerprint}` sur KeyDetail |
| `app/src/pages/MyKeys.tsx` | `key={selectedKey.fingerprint}` sur KeyDetail |

### À supprimer (hacks remplacés par file pickers)
| Fichier | Ce qui doit être supprimé |
|---------|--------------------------|
| `app/src/components/KeyDetail.tsx` | Import `downloadDir` + `@tauri-apps/api/path` + modal export text-input |
| `app/src/pages/MyKeys.tsx` | Import `downloadDir` + `@tauri-apps/api/path` + modal backup text-input + état `backupState` |
| `app/src/lib/mock-path.ts` | Supprimer ce fichier (ne sera plus importé) |
| `app/vite.config.ts` | Supprimer l'alias `@tauri-apps/api/path` → mock-path.ts |

---

## Phases

### Phase 0 — Audit pré-travaux
**Agent** : Explore (rapide)
**But** : état réel de compilation et des tests avant de toucher quoi que ce soit.
**Livrable** : `phase0_results.md` dans ce dossier.

```bash
nix develop --command sh -c '
  cargo build 2>&1 | tail -5
  cargo clippy -- -D warnings 2>&1 | tail -10
  cargo test --package pgpilot --lib 2>&1 | tail -10
  cd app && npm run test:e2e 2>&1 | tail -30
'
```

---

### Phase 1 — Nettoyage + File pickers natifs
**Agent** : `voltagent-lang:react-specialist`
**Dépend de** : Phase 0
**Livrable** : code propre + `manual_tests.md` enrichi (section file pickers)

#### 1a — Supprimer les hacks

**`app/src/components/KeyDetail.tsx`** :
- Supprimer import `downloadDir` + import `@tauri-apps/api/path`
- Supprimer état `modal` kind `'export'`
- Supprimer `handleExportConfirm()`
- Supprimer le bloc JSX du modal export
- Restaurer `handleExportPublic()` proprement

**`app/src/pages/MyKeys.tsx`** :
- Supprimer import `downloadDir` + import `@tauri-apps/api/path`
- Supprimer imports `Input` et `Modal` si plus utilisés dans ce fichier
- Supprimer états `backupState` + `backupLoading`
- Supprimer `handleBackupConfirm()`
- Supprimer le bloc JSX du modal backup
- Restaurer `handleBackup()` proprement

**`app/src/lib/mock-path.ts`** : supprimer.

**`app/vite.config.ts`** : supprimer l'alias `@tauri-apps/api/path`.

#### 1b — Implémenter les file pickers natifs

**Export clé publique (`KeyDetail.tsx`)** :
```ts
import { save } from '@tauri-apps/plugin-dialog';

async function handleExportPublic(): Promise<void> {
  const path = await save({
    defaultPath: `${fp.slice(-16)}.asc`,
    filters: [{ name: 'PGP Key', extensions: ['asc'] }],
  });
  if (path === null) return;
  setActionLoading(true);
  exportPublicKeyToFile(fp, path)
    .then(() => { setStatus('success', `Exported to ${path}`); })
    .catch((err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      setStatus('error', `Export failed: ${msg}`);
    })
    .finally(() => { setActionLoading(false); });
}
```

**Backup clé secrète (`MyKeys.tsx`)** :
```ts
import { open } from '@tauri-apps/plugin-dialog';

async function handleBackup(fp: string): Promise<void> {
  const dir = await open({ directory: true });
  if (dir === null) return;
  backupKey(fp, dir)
    .then((files) => { setStatus('success', `Backup saved: ${files.join(', ')}`); })
    .catch((err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      setStatus('error', `Backup failed: ${msg}`);
    });
}
```

**`app/src/lib/mock-dialog.ts`** — ajouter `save()` :
```ts
export async function save(_options?: {
  defaultPath?: string;
  filters?: { name: string; extensions: string[] }[];
}): Promise<string | null> {
  return '/tmp/mock-export-key.asc';
}
```

#### 1c — Tests automatisés (nouveaux)
**Agent** : `voltagent-qa-sec:test-automator` (en parallèle ou juste après 1b)

Écrire ou mettre à jour des tests E2E Playwright pour couvrir le flux file picker en mock mode :
- Export clé publique : clic sur "Export" → mock `save()` retourne un chemin → toast "Exported to …" visible
- Backup clé : clic sur "Backup" → mock `open()` retourne un dossier → toast "Backup saved: …" visible
- Vérifier que le test existant `trusted key chip` est analysé (voir Phase 4)

```bash
cd app && npm run test:e2e
```

#### 1d — Lint + TypeScript
```bash
nix develop --command sh -c 'cd app && npx tsc --noEmit && npm run lint'
```

---

### Phase 2 — MAJ dépendances npm
**Agent** : `voltagent-dev-exp:dependency-manager`
**Dépend de** : Phase 1 terminée
**Peut tourner en parallèle avec Phase 3**

| Package | Version cible |
|---------|--------------|
| `@tauri-apps/cli` | v2.11.2 |
| `@types/react` | v19.2.15 |
| `@vitejs/plugin-react` | v6.0.2 |
| `react-router-dom` | v7.15.1 |
| `vite` | v8.0.14 |
| `@typescript-eslint/eslint-plugin` | v8.59.4 |
| `@typescript-eslint/parser` | v8.59.4 |
| `typescript-eslint` | v8.59.4 |
| `eslint` | v10.4.0 |
| `lucide-react` | v1.16.0 |
| `@storybook/react` | v10.4.1 |
| `@storybook/react-vite` | v10.4.1 |
| `storybook` | v10.4.1 |

Après MAJ :
```bash
cd app && npm install
npx tsc --noEmit
npm run lint
npm run test:e2e
```

Si breaking changes (ex: react-router-dom v7 API, eslint v10 config), les corriger dans cette phase.

---

### Phase 3 — MAJ dépendances Rust
**Agent** : `voltagent-lang:rust-engineer`
**Dépend de** : Phase 1 terminée
**Peut tourner en parallèle avec Phase 2**

| Crate | Scope |
|-------|-------|
| `serde_json` | workspace |
| `tauri` | app/src-tauri |
| `tauri-build` | app/src-tauri build-dep |

```bash
nix develop --command sh -c '
  cargo update serde_json tauri tauri-build
  cargo build
  cargo clippy -- -D warnings
  cargo test --package pgpilot --lib
  cargo audit --ignore RUSTSEC-2026-0098 --ignore RUSTSEC-2026-0099 \
              --ignore RUSTSEC-2026-0104 --ignore RUSTSEC-2026-0049
'
```

Vérifier que les CVEs ignorées dans `cargo audit` sont toujours les seules (et toujours upstream-bloquées).

---

### Phase 4 — Fix test E2E cassé
**Agent** : `voltagent-qa-sec:test-automator`
**Dépend de** : Phases 1, 2, 3

Test cassé : `encrypt.spec.ts >> trusted key chip shows checkmark indicator`
Locator : `locator('[aria-pressed]').filter({ hasText: 'Alice Dupont' }).locator('[aria-label="trusted"]')`

Démarche :
1. Lire `app/tests/e2e/encrypt.spec.ts` — comprendre ce que le test attend exactement
2. Lire `app/src/pages/Encrypt.tsx` — vérifier si le chip `[aria-label="trusted"]` est rendu
3. Lire le mock pour Alice (`mock-tauri.ts`) — vérifier son niveau de trust
4. Décider : le test est-il juste (l'implémentation manque le chip) ou le test est-il obsolète ?
5. Corriger l'implémentation OU le test, pas les deux à la fois

Suite complète après fix :
```bash
cd app && npm run test:e2e
```
Tous les tests doivent être verts avant de passer à la phase suivante.

---

### Phase 5 — Code review + sécurité
**Agents** : `voltagent-qa-sec:code-reviewer` + `voltagent-qa-sec:security-auditor`
**Dépend de** : Phases 1–4

Points à inspecter :
- Fix trust 'full' dans `Encrypt.tsx` — la logique `--trust-model always` est-elle correcte et sans abus possible ?
- Nouveaux file pickers — le chemin retourné par `save()` est-il validé côté Rust avant usage ? (`canonicalize()`, etc.)
- Conventions CLAUDE.md : pas de commentaires superflus, pas d'abstractions inutiles, pas de gestion d'erreurs pour cas impossibles
- IPC Rust ↔ TypeScript : signatures cohérentes, types corrects
- Aucune régression de sécurité introduite par les MAJ de deps

---

### Phase 6 — Release Nix
**Dépend de** : Phase 2 (package-lock.json potentiellement modifié), Phases 3 + 5

#### 6a — Recompute npmDepsHash
```bash
nix run nixpkgs#prefetch-npm-deps -- app/package-lock.json
```
Mettre à jour `npmDepsHash` dans `packages/pgpilot/default.nix`.

#### 6b — Vérification nix build
```bash
nix build .#pgpilot 2>&1 | tail -20
```
Le build Nix compile le backend Rust + frontend React via `cargo-tauri.hook`.
Il doit produire un binaire dans `./result/bin/pgpilot-app`.

**Note** : ce build est long (~20–40 min la première fois). Le lancer avant le tag pour
s'assurer que le package Nix est installable par les utilisateurs home-manager.

#### 6c — Vérification flake
```bash
nix flake check
```
Vérifie l'intégrité du flake, les outputs déclarés, et que le module home-manager
est syntaxiquement valide.

---

### Phase 7 — Plan de tests manuels + Validation finale
**Exécutant** : utilisateur

#### 7a — Exécuter le plan de tests manuels
Voir `manual_tests.md` dans ce dossier pour la liste complète.

#### 7b — Checklist release
- [ ] `cargo build` vert
- [ ] `cargo clippy -- -D warnings` vert
- [ ] `cargo test --package pgpilot --lib` vert
- [ ] `cd app && npm run test:e2e` — tous les tests verts
- [ ] `nix build .#pgpilot` produit `./result/bin/pgpilot-app` fonctionnel
- [ ] `nix flake check` vert
- [ ] `npmDepsHash` à jour dans `packages/pgpilot/default.nix`
- [ ] SECURITY_PLAN.md non commité
- [ ] Tests manuels exécutés et validés
- [ ] `git tag v0.8.9 && git push --tags`

---

## Décisions techniques

| Décision | Choix | Raison |
|----------|-------|--------|
| Export public key | `save()` plugin-dialog | Cohérence avec Encrypt/Sign/Verify |
| Backup key | `open({ directory: true })` | L'utilisateur choisit le dossier, Rust génère les noms de fichiers |
| Revocation cert | Pas de file picker | GPG décide du chemin (`openpgp-revocs.d/`) |
| mock-path.ts | Supprimer | Plus utilisé après remplacement des hacks |
| SECURITY_PLAN.md | Ne pas commiter | Document interne de suivi |
| Plan de tests manuels | `manual_tests.md` par version | Accumulation progressive, exécuté en Phase 7 |

---

## Ordre d'exécution

```
Phase 0 (audit)
    ↓
Phase 1 (nettoyage + file pickers + tests mock)
    ↓
Phase 2 ←→ Phase 3 (parallèles)
    ↓
Phase 4 (fix test E2E)
    ↓
Phase 5 (code review + sécurité)
    ↓
Phase 6 (release Nix : npmDepsHash + nix build + flake check)
    ↓
Phase 7 (tests manuels + validation + tag)
```
