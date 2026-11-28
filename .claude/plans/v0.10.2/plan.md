# Plan v0.10.2 — Correctifs Nix (nixpkgs 26.11) + advisories TLS

## Contexte

`v0.10.1` (tag `b02c31c`, 2026-09-27) ne se construit plus et ne se lance plus
correctement sur un système NixOS à jour : le `flake.lock` épinglait un nixpkgs d'avril
2026. Trois correctifs ont été livrés et mergés sur `main` le 2026-11-28, chacun sur sa
branche. Ils sont décrits plus bas (phase 1) ; il reste à les publier.

L'audit de phase 0 a par ailleurs relevé 9 advisories RustSec apparues depuis
`v0.10.1` et absentes de la liste d'exclusion de la CI (cf.
[`phase0_results.md`](phase0_results.md)). Huit se corrigent par une mise à jour de
`Cargo.lock` ; la neuvième est bloquée par `rumqttc`.

## Objectifs

1. Publier les trois correctifs Nix déjà sur `main`.
2. Corriger les advisories corrigeables sans toucher aux manifestes.
3. Garder `cargo audit` lisible : une seule exclusion ajoutée, justifiée.

## Périmètre

### In scope

- Les trois correctifs déjà mergés (`fix/nix-aws-lc`, `fix/e2e-chromium-glibc`,
  `chore/flake-update`).
- `cargo update` ciblé sur 7 crates (8 paquets verrouillés), `Cargo.lock` seul.
- Exclusion de `RUSTSEC-2026-0382` dans `ci.yml` pour l'instance `rustls-webpki 0.102.8`.
- Bump de version v0.10.2 sur les 5 fichiers (PROCEDURE §5).

### Out of scope (explicite)

- **Les 10 PR Renovate ouvertes** (#36, #39 à #46, #54) : vague à traiter dans un plan
  dédié. #54 (`@tauri-apps/plugin-dialog` 2.8.4) reste dans le même minor que le crate
  Rust, donc sans urgence.
- **Le job front/docs absent de `ci.yml`** et **`npm run lint` cassé sur `main`** :
  toujours au backlog, cf. plan v0.10.1.
- **Les avertissements `unmaintained` / `unsound`** de `cargo audit` (chrono, zvariant,
  glib 0.18, event-listener…) : dépendances transitives de Tauri/GTK, sans correctif
  disponible par simple mise à jour du lock.

## État du working tree

Branche `chore/release-0.10.2`, partie de `main` à `0d9a3c0`.

Fichiers non suivis à **ne pas** committer : `SECURITY_PLAN.md`, `docker/`,
`target-docker/`, `result`, `app/result`.

`app/playwright-report/index.html` est suivi et réécrit à chaque `just e2e` : le
restaurer (`git checkout --`) avant tout commit.

## Phases

### Phase 0 — Audit ✅ 2026-11-28

Livrable : [`phase0_results.md`](phase0_results.md). `just ci` vert sur `main`.

### Phase 1 — Correctifs Nix ✅ déjà sur `main`

| Commit | Branche | Contenu |
|--------|---------|---------|
| `a87ba7a` | `fix/nix-aws-lc` | `aws-lc` dans les `buildInputs` |
| `e24e1e4` | `fix/e2e-chromium-glibc` | Chromium du store lancé sans `LD_LIBRARY_PATH` |
| `8f4a5ee` | `chore/flake-update` | nixpkgs 2026-04-30 → 2026-11-24 |

Détail de chaque correctif dans les sections « Correctif » en fin de document.

### Phase 2 — Advisories RustSec

1. `cargo update -p rustls -p rustls-webpki@0.103.15 -p webpki-roots@1.0.9 -p url
   -p cookie -p tokio -p rustix@1.1.4` (D1).
2. `--ignore RUSTSEC-2026-0382` ajouté à `ci.yml`, et `CLAUDE.md` mis à jour (D2).
3. Contrôle : `cargo audit` avec la liste de la CI ne remonte plus aucune vulnérabilité.

### Phase 3 — Bump de version + release Nix

1. `just release 0.10.2` : 5 fichiers + `app/package-lock.json` + recalcul de
   `npmDepsHash` (attendu inchangé, le lock npm ne bouge pas).
2. `cargo build` pour répercuter la version dans `Cargo.lock`.
3. `nix build`, puis `env -u LD_LIBRARY_PATH ./result/bin/pgpilot-app` (PROCEDURE §7b).
4. `nix flake check` (PROCEDURE §7c).
5. `just ci` vert sur le dernier commit.
6. `manual_tests.md` exécuté par l'utilisateur.

### Phase 4 — Clôture

Merge de `chore/release-0.10.2` sur `main` par l'utilisateur, push, puis tag `v0.10.2`.

## Agents (PROCEDURE §4)

Aucun agent mobilisé : la version ne modifie aucun fichier source Rust ou TypeScript
de l'application. Les changements portent sur `Cargo.lock`, `flake.lock`, le paquet
Nix, la configuration Playwright et `ci.yml`. Le contrôle repose sur les portes
(`just ci`, `nix build`, `nix flake check`, `cargo audit`) et sur les tests manuels.

## Décisions techniques

### D1 — Mise à jour ciblée du lock, pas de `cargo update` global

Seules les crates visées par une advisory sont mises à jour, toutes dans leur version
de correctif compatible semver :

| Crate | Avant | Après | Advisory corrigée |
|-------|-------|-------|-------------------|
| `rustls` | 0.23.45 | 0.23.46 | RUSTSEC-2026-0346, -0347, -0348, -0349 |
| `rustls-webpki` | 0.103.15 | 0.103.17 | RUSTSEC-2026-0382 |
| `webpki-roots` | 1.0.9 | 1.0.12 | RUSTSEC-2026-0381 |
| `url` (+ `form_urlencoded` 1.2.2 → 1.2.3) | 2.5.8 | 2.5.9 | RUSTSEC-2026-0592 |
| `cookie` | 0.18.1 | 0.18.3 | RUSTSEC-2026-0594 |
| `tokio` | 1.53.1 | 1.54.1 | RUSTSEC-2026-0386 (unsound) |
| `rustix` | 1.1.4 | 1.1.6 | RUSTSEC-2026-0486 (unsound) |

Un `cargo update` global toucherait 167 autres paquets, dont `tauri` : cela rouvrirait
le couplage npm ↔ Rust (plan v0.10.1, D1), qui relève de la vague Renovate.

`rustls`, `rustls-webpki` et `webpki-roots` sont sur le chemin TLS du chat MQTT et des
requêtes keyserver (`ureq`) : c'est la partie exposée au réseau, d'où le choix de ne pas
attendre la vague Renovate.

### D2 — `RUSTSEC-2026-0382` exclue pour l'instance 0.102.8

`rumqttc 0.25.1` dépend directement de `rustls-webpki 0.102.8`, branche qui ne reçoit
pas le correctif. C'est le même blocage que les quatre advisories `rustls-webpki` déjà
exclues. `cargo audit --ignore` porte sur l'identifiant, pas sur la version : l'instance
0.103 est donc à vérifier à la main, et elle est corrigée par D1 (0.103.17).

---

## Correctif post-release — build Nix sur nixpkgs 26.11 (2026-11-28)

Branche `fix/nix-aws-lc`.

**Symptôme.** `nix build` échoue dès que le paquet est évalué avec un nixpkgs récent
(par exemple `inputs.pgpilot.inputs.nixpkgs.follows`), alors qu'il passe avec le
`flake.lock` du dépôt (nixpkgs du 2026-04-30) :

```
error: failed to run custom build command for `aws-lc-sys v0.45.0`
AWS_LC_SYS_USE_SYSTEM=1 requires a system-installed AWS-LC, but none was found.
```

**Cause.** Depuis nixpkgs 26.11, `rustPlatform.cargoSetupHook` force plusieurs
bibliothèques C à venir de nixpkgs plutôt que de la copie embarquée des crates
(notes de version `rl-2611`), et exporte pour cela `AWS_LC_SYS_USE_SYSTEM=1`.
`aws-lc-sys` arrive par `rustls` ; le paquet ne fournissait pas `aws-lc`.

**Correctif.** `aws-lc` ajouté aux `buildInputs` de `packages/pgpilot/default.nix`.
Le paquet existe aussi dans le nixpkgs épinglé, où la variable n'est pas exportée et
où la copie embarquée continue d'être utilisée : le build reste valide des deux côtés.

**Écarté.** `dontCargoUseSystemLibs = true;` (retrait prévu par nixpkgs) : une ligne,
mais on resterait sur les copies embarquées, à rebours de la direction de nixpkgs.

**Hors périmètre.** La mise à jour de `flake.lock` : changement séparé.

## Correctif post-release — E2E : Chromium du store et `LD_LIBRARY_PATH` (2026-11-28)

Branche `fix/e2e-chromium-glibc`.

**Symptôme.** `just e2e` échoue sur tous les tests quand il est lancé depuis un
environnement qui exporte un `LD_LIBRARY_PATH` étranger (observé : le lanceur Nix de
Claude Code y ajoute une `alsa-lib` liée à glibc 2.43) :

```
chromium: .../glibc-2.42-61/lib/libm.so.6: version `GLIBC_2.43' not found
```

**Cause.** `playwright.config.ts` choisit un Chromium dans `/nix/store`, donc issu d'un
nixpkgs quelconque, puis le lance avec le `LD_LIBRARY_PATH` hérité. Chromium charge
alors des bibliothèques liées à une autre glibc que la sienne. Le dev shell injecte
lui-même les bibliothèques du nixpkgs épinglé : le même défaut pouvait survenir sans
aucun lanceur externe.

**Correctif.** Un Chromium du store est lancé sans `LD_LIBRARY_PATH` : il apporte ses
propres chemins. Le navigateur téléchargé (hors NixOS, CI GitHub) n'est pas concerné.

**Vérification.** `just ci` dans l'environnement fautif : 92 tests E2E passés, contre
un échec complet avant.

## Correctif post-release — fenêtre vide, `EGL_BAD_PARAMETER` (2026-11-28)

Branche `chore/flake-update`.

**Symptôme.** Le binaire issu de `nix build` ouvre une fenêtre vide et affiche
`Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...`.

**Cause.** `flake.lock` épinglait nixpkgs au 2026-04-30 (glibc 2.42). Sur un système
plus récent (glibc 2.44, Mesa 26.2.6), WebKitGTK doit charger le pilote EGL de
`/run/opengl-driver`, compilé pour une glibc plus récente que celle du processus : le
chargement échoue. Ce n'est pas propre à Tauri ; toute application qui utilise le GPU
charge ce pilote.

**Correctif.** `nix flake update nixpkgs` : 2026-04-30 → 2026-11-24. Le correctif
`aws-lc` ci-dessus en est un prérequis.

**Vérification.** `nix build`, `nix flake check` et `just ci` verts (92 tests E2E).
Binaire lancé : plus d'erreur EGL, interface affichée, connexion à une room de chat
confirmée par l'utilisateur sur un build équivalent.

**À retenir.** Le même symptôme réapparaît si on lance le binaire avec un
`LD_LIBRARY_PATH` hérité d'un dev shell plus ancien (les bibliothèques de ce chemin
passent avant celles du paquet). Tester avec `env -u LD_LIBRARY_PATH`, et relancer
toute session ouverte depuis l'ancien dev shell. Un `flake.lock` qui prend du retard
sur le système de l'utilisateur recréera le problème.
