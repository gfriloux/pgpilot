# Procédure — Gestion d'une nouvelle version

Ce document définit la procédure à suivre **systématiquement** avant de commencer tout travail
sur une nouvelle version ou un nouveau plan fonctionnel.

> **Règle fondamentale : on ne code pas sans plan validé.**
> Toute session qui touche au code commence par la création ou la relecture du plan de version.
>
> Et : lis [`DESIGN.md`](DESIGN.md) avant tout changement. **DESIGN.md fait foi.** Si une idée
> ne s'inscrit pas dans les invariants, la réponse est non — sauf décision DESIGN explicite,
> qui n'est pas un simple PLAN d'implémentation.

---

## 1. Créer le plan avant tout

Dès qu'une nouvelle version est évoquée, créer le dossier et les fichiers de plan :

```
.claude/plans/v{X.Y.Z}/
  plan.md           ← phases, agents, décisions techniques, périmètre
  manual_tests.md   ← tests manuels (enrichi au fil du dev, exécuté en phase finale)
  phase0_results.md ← état réel de la base avant de coder (cf. §2)
```

Les plans vivent **dans `.claude/plans/`**, jamais à la racine, et sont **commités**
(seuls `.claude/settings.local.json` et `.claude/worktrees/` restent locaux). Un plan
obsolète est **supprimé**, pas dupliqué en `_v2`/`_v3`. Le layout complet et la carte
des versions sont dans [`.claude/plans/README.md`](.claude/plans/README.md) — à mettre à
jour à chaque release (calqué sur `stc`/`astropath`).

### Contenu minimal de `plan.md`
- **Contexte** : d'où on part, pourquoi cette version
- **Objectifs** : ce qu'on veut atteindre
- **Périmètre** : in scope / out of scope explicites
- **État du working tree** : ce qui est déjà là, ce qui doit être supprimé
- **Phases ordonnées** : avec agent assigné, dépendances, livrables
- **Décisions techniques** : choix et justifications

### Contenu minimal de `manual_tests.md`
- Enrichi **après chaque phase** qui ajoute un comportement utilisateur
- Structuré en sections par fonctionnalité
- Exécuté par l'utilisateur lors de la validation finale

---

## 2. Phase 0 — Audit obligatoire

**Avant de toucher au code**, vérifier l'état réel — ne jamais supposer la base propre :

```bash
nix develop --command just ci
```

Consigner le résultat (exit code, sorties pertinentes, branche de départ) dans
`.claude/plans/v{X.Y.Z}/phase0_results.md`.

---

## 3. Politique git — hybride

- Claude travaille sur une **branche dédiée** (`feat/…`, `fix/…`, `chore/…`,
  `refactor/…`, `docs/…`), jamais directement sur `main`.
- Claude **commite atomiquement** : un changement logique = un commit, en
  [Conventional Commits](#convention-de-commit). Chaque commit passe `just ci` seul.
- Claude ne fait **jamais** `merge`, `push` ni `tag`. L'utilisateur relit, merge sur
  `main`, push, et pose les tags de version.
- **Un plan se termine toujours par un merge sur `main`.** À la clôture (portes vertes),
  l'utilisateur merge la branche du plan sur `main` et push, **avant** de démarrer le plan
  suivant. On ne laisse pas une branche de plan terminée non mergée : chaque plan part d'un
  `main` à jour.

### Convention de commit

```
type(scope): message court à l'impératif
```

- **type** : `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `chore`, `ci`.
- **scope** : `gpg`, `chat`, `config`, `tauri`, `ui`, `nix`, `docs`, ou le module touché.

La doc se met à jour **dans le même commit** que le code qu'elle décrit. Un changement
structurel commité sans MAJ de `DESIGN.md`/`README.md`/`CLAUDE.md` rend la doc périmée —
c'est un défaut, pas un « à faire plus tard ».

---

## 4. Agents à mobiliser systématiquement

Chaque plan doit inclure une phase dédiée à chacun de ces agents.
Ne pas les regrouper : chacun a un rôle distinct.

| Rôle | Agent | Quand |
|------|-------|-------|
| Sécurité | `voltagent-qa-sec:security-auditor` | Après chaque phase qui touche à la crypto, l'IPC, les fichiers |
| Qualité | `voltagent-qa-sec:code-reviewer` | Avant la validation finale |
| Refactorisation | `voltagent-dev-exp:refactoring-specialist` | Uniquement sur les parties de code **touchées** par la version, pas en scope global |
| Tests automatisés | `voltagent-qa-sec:test-automator` | Après chaque implémentation de feature |
| Documentation | `voltagent-biz:technical-writer` ou `voltagent-dev-exp:documentation-engineer` | Si une feature visible est ajoutée |
| Dépendances | `voltagent-dev-exp:dependency-manager` | Pour les MAJ npm |
| Rust | `voltagent-lang:rust-engineer` | Pour les MAJ crates et tout code Rust non trivial |
| React/TS | `voltagent-lang:react-specialist` ou `voltagent-lang:typescript-pro` | Pour le frontend |

---

## 5. Bump de version

À chaque nouvelle version, mettre à jour **les 5 fichiers suivants** :

| Fichier | Champ |
|---------|-------|
| `Cargo.toml` (racine) | `[package] version` |
| `app/src-tauri/Cargo.toml` | `[package] version` |
| `app/package.json` | `"version"` |
| `app/src-tauri/tauri.conf.json` | `"version"` |
| `packages/pgpilot/default.nix` | `version = "..."` |

`Cargo.lock` se met à jour automatiquement via `cargo build`.

---

## 6. Convention tests

### Automatisés (Playwright E2E + Rust)

Tout ce qui ne nécessite pas d'interaction physique avec le système.
En mode `VITE_MOCK=true`, les dialogs natifs sont remplacés par `app/src/lib/mock-dialog.ts`.

**On automatise :**
- Flux UI complets en mock mode (encrypt, sign, verify, import, export…)
- Comportement des toasts, états de boutons, messages d'erreur
- Logique de trust et chips dans Encrypt
- Tests Rust unitaires (`cargo test --package pgpilot --lib`)

**On n'automatise pas :**
- Ouverture physique du file picker natif (KDE Plasma)
- Interaction avec un vrai keyring GPG (sauf tests `--ignored`)
- Build Nix complet
- Comportement sur machine réelle (résolution, thème, pinentry)

### Tests manuels (`manual_tests.md`)

- Rédiger la section correspondante **immédiatement après** chaque phase qui ajoute
  un comportement utilisateur (ne pas attendre la fin)
- Exécuter l'intégralité du fichier lors de la validation finale (cf. §8)
- Format : tableau Action / Résultat attendu avec cases à cocher

---

## 7. Release Nix (obligatoire avant tout tag)

Le projet est installable via home-manager. Un tag sans Nix vérifié casse l'installation
pour tous les utilisateurs.

### 7a — Recompute npmDepsHash (si `app/package-lock.json` a changé)
```bash
nix run nixpkgs#prefetch-npm-deps -- app/package-lock.json
# Puis mettre à jour npmDepsHash dans packages/pgpilot/default.nix
```

### 7b — Vérifier le build Nix
```bash
nix build
# Doit produire ./result/bin/pgpilot-app
./result/bin/pgpilot-app  # Lancer manuellement pour vérifier
```

Ce build est long (~20–40 min la première fois). Le lancer avant le tag.

### 7c — Vérifier le flake
```bash
nix flake check
# Vérifie l'intégrité du flake et que le module home-manager est syntaxiquement valide
```

---

## 8. Validation finale et tag

### Checklist automatisée
```bash
nix develop --command just ci   # fmt-check + clippy + tests Rust + E2E
nix build                       # build Nix complet
nix flake check                 # intégrité flake + module home-manager
```

### Checklist manuelle (dans `manual_tests.md`)
- Exécuter tous les tests manuels de la version
- Cocher chaque case

### Clôture + tag (par l'utilisateur)

Claude s'arrête ici : portes vertes, branche du plan prête. **L'utilisateur** relit, merge la
branche sur `main`, push, puis pose le tag :

```bash
git tag v{X.Y.Z}
git push --tags
```

La CI génère ensuite automatiquement le CHANGELOG via `git-cliff` et publie la release GitHub
avec les artefacts `.deb` et `.rpm`.

---

## 9. Checklist spécifique KDE/Plasma (NixOS)

À vérifier à **chaque release** qui touche aux dialogs fichier :

- [ ] `GTK_USE_PORTAL=1` présent dans le shellHook du dev shell (pour `just dev`)
- [ ] `"--set GTK_USE_PORTAL 1"` présent dans `makeWrapperArgs` de `packages/pgpilot/default.nix` (pour le binaire Nix)

Sans les deux, le binaire installé crash avec :
```
GLib-GIO-ERROR: Settings schema 'org.gtk.Settings.FileChooser' is not installed
```

Le shellHook seul ne suffit pas — le binaire installé n'en hérite pas.

---

## 10. Ce qui ne change pas entre les versions

- **DESIGN.md fait foi.** Hors invariants → non (cf. en-tête + `DESIGN.md`).
- **SECURITY_PLAN.md** : document interne, ne jamais commiter
- **Git : hybride** — Claude bosse sur une branche dédiée, commite atomiquement
  (Conventional Commits, cf. §3), et ne fait **jamais** `merge`/`push`/`tag`. L'utilisateur
  relit, merge sur `main`, push, tague. Chaque plan se termine par un merge sur `main`.
- **Porte qualité** : `just ci` est l'unique définition des gates — pre-commit et la CI
  l'appellent. Tout commit passe `just ci` seul.
- **Nix dev shell** : toujours utiliser `nix develop --command …` pour les commandes non interactives
- **File pickers** : toujours utiliser `@tauri-apps/plugin-dialog` — uniquement `open()`, jamais `save()`. Sur KDE/Plasma, `save()` crash même avec `GTK_USE_PORTAL=1`. Pour écrire un fichier : `open({ directory: true })` + nom auto-généré côté code.
- **CVEs connues** : les RUSTSEC-2026-* liées à `rumqttc` sont ignorées en CI (bloquées upstream) — vérifier à chaque version qu'aucune nouvelle CVE ne s'y est ajoutée
