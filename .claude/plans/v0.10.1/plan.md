# Plan v0.10.1 — Vague Renovate #2 + migration astro 7

## Contexte

`v0.10.0` a consolidé une première vague de 10 PR Renovate. Deux de ces branches
(`renovate/rust-deps`, `renovate/react-monorepo`) sont **récurrentes** : Renovate les
force-push avec de nouvelles versions au lieu de les fermer. Une seconde vague de 10 PR
s'est donc reconstituée : **#19, #29, #31, #32, #33, #34, #35, #36, #37, #38**.

Contrairement à la vague v0.10.0, celle-ci n'est **pas** homogène. L'audit (cf.
[`phase0_results.md`](phase0_results.md)) montre que 4 PR sur 10 sont inmergeables en
l'état, et que la CI GitHub affiche un ✅ sur les 9 PR npm sans en tester une seule ligne.

## Objectifs

1. Intégrer les 6 PR qui sont saines, en respectant leurs couplages.
2. Réussir la migration docs astro 6 → 7 + starlight 0.39 → 0.42, qui est un vrai
   changement de configuration et non un bump.
3. Documenter le sort des PR bloquées pour ne pas les re-auditer à la vague suivante.

## Périmètre

### In scope

- Les 6 PR saines : #19, #29, #31, #32, #37, #38 (app + Rust) et #34 (docs).
- La migration couplée #35 + #33 (astro 7 + starlight 0.42), collection `i18n` incluse.
- Recalcul de `npmDepsHash` et vérification `nix build` (PROCEDURE §7a/§7b).
- Bump de version v0.10.1 sur les 5 fichiers (PROCEDURE §5).

### Out of scope (explicite)

- **#36 typescript 7** — bloqué upstream, cf. décision D3. Aucun contournement retenu.
- **Ajouter un job front/docs à `ci.yml`** — cause racine identifiée, laissée au backlog
  sur décision utilisateur. Sans ce job, la vague suivante repassera au vert sans test.
- **`npm run lint` cassé sur `main`** (`AppLayout.tsx:62`, variable `theme` inutilisée) —
  dette préexistante, chantier séparé.
- **Automatiser `npmDepsHash`** (Renovate `postUpgradeTasks`) — backlog.

## État du working tree

Fichiers non suivis présents à l'audit, à **ne pas** committer : `SECURITY_PLAN.md`,
`docker/`, `target-docker/`, `result`, `app/result`.

`app/playwright-report/index.html` est **suivi** et réécrit à chaque `npm run test:e2e`.
Le restaurer (`git checkout --`) avant tout commit pour ne pas polluer les diffs.

## Verdicts par PR

| PR | Contenu | Verdict |
|----|---------|---------|
| #19 | Rust deps — dont `base64` 0.22→**0.23** (breaking), `tauri` 2.11.5→**2.12.0**, `tauri-plugin-dialog` 2.7.1→**2.8.0** | ✅ gate Rust complet vert. **Couplée à #31 et #37** (D1) |
| #38 | prettier → 3.9.9 (lock) | ✅ devDep, sans risque |
| #37 | `@tauri-apps/plugin-dialog` → **2.8.0** (lock) | ✅ mais **exige #19** (D1). ⚠️ Renovate a force-push : la PR ciblait 2.7.3, elle cible maintenant 2.8.0 |
| #32 | `@vitejs/plugin-react` → 6.1.1 | ✅ |
| #31 | tauri monorepo npm → **2.12.0** | ✅ mais **exige #19** (D1) |
| #29 | react monorepo → 19.3.0 (+ scheduler 0.28.0) | ✅ |
| #34 | astro → 6.4.8 (lock docs seul) | ✅ build docs 25 pages, 0 warning. Indépendante |
| #35 | astro → **^7.0.0** (manifest seul) | ❌ seule : starlight 0.39 exige `astro ^6`. **Exige #33** (D2) |
| #33 | `@astrojs/starlight` → **^0.42.0** (manifest seul) | ❌ seule : starlight 0.42 exige `astro ^7.2.10`. **Exige #35** (D2) |
| #36 | typescript → **^7.0.0** | ❌ **bloqué upstream** (D3) |

Les 3 PR `renovate/artifacts: FAILURE` (#33, #35, #36) sont exactement celles dont
Renovate n'a **pas pu** régénérer le lockfile — c'est le signal fiable, contrairement au
✅ de la CI.

## Phases

### Phase 0 — Audit (fait)

Livrable : [`phase0_results.md`](phase0_results.md). ✅ Terminé le 2026-09-27.

### Phase 1 — Consolidation app + Rust (faite, hors branche de plan)

Branche **`chore/deps-app-npm`**, 7 commits : les 6 commits Renovate cherry-pickés
(#19, #38, #37, #32, #31, #29 — `#19` en premier pour que le couplage D1 tienne à chaque
commit) + `chore(nix): update npmDepsHash…`.

Livrée séparément pour être mergeable immédiatement, sans attendre la validation du
présent plan.

Validation exécutée :

- `npm ci` (lockfile cohérent avec le manifeste) ✅
- `tsc && vite build` ✅
- Playwright **92/92** ✅
- gate Rust complet (fmt, clippy `-D warnings`, tests unit + `--ignored`) ✅
- `nix build .#pgpilot` ✅ — `npmDepsHash` = `sha256-vhQinrruVZOR697EzKSwZTshUOaZhWrwsNLdTpWUELg=`
- hooks `alejandra` + `deadnix` ✅

### Phase 2 — Migration astro 7 + starlight 0.42 ✅ faite le 2026-09-27

Étapes exécutées, dans l'ordre :

1. `docs/package.json` : `astro` → `^7.0.0`, `@astrojs/starlight` → `^0.42.0`, **dans le
   même commit** (D2 — aucun des deux ne résout seul).
2. **Supprimer** `docs/package-lock.json` puis `npm install`. Un `npm install` sur le
   lock existant échoue (`ERESOLVE`, il tente de conserver starlight 0.39.2) — la
   suppression est nécessaire, pas cosmétique.
3. `docs/src/content.config.ts` : déclaration de la collection `i18n` (D4).
4. `docs/src/content/i18n/{en,fr}.json` contenant `{}` (D4).

Résolutions obtenues : astro **7.3.5**, starlight **0.42.4**, vite **8.3.1**.
Node requis par astro 7 : `>=22.12.0` — dev shell local 22.22.2 ✅, `docs.yml` node 24 ✅.

Validation mesurée :

| Contrôle | Résultat |
|----------|----------|
| `npm install` puis `npm ci` (ce que fait `docs.yml`) | ✅ |
| `npm audit` | ✅ **0 vulnérabilité** |
| `npm run build` | ✅ **25 pages**, aucun warning nouveau (D7) |
| Pages générées | ✅ 12 EN + 12 FR + racine, identiques à astro 6 |
| Chaînes UI starlight | ✅ « Sur cette page » en FR, « On this page » en EN |
| `<html lang>` | ✅ `fr` sur `/fr/…`, `en` à la racine |
| `base: '/pgpilot/'` | ✅ tous les chemins absolus préfixés, aucun lien hors base |
| Screenshots | ✅ 18 PNG dans `dist/`, aucune référence morte |
| Recherche Pagefind | ✅ index **bilingue** (`pagefind.en_*`, `pagefind.fr_*`) |
| Sitemap | ✅ `sitemap-index.xml` + `sitemap-0.xml` |

Reste à faire côté utilisateur : la validation visuelle (`manual_tests.md` §4).

⚠️ `#34` (astro 6.4.8) est désormais **sans objet** : astro ^7 l'englobe. Ne pas la
merger.

### Phase 3 — Bump de version + release Nix

1. `just release 0.10.1` → 5 fichiers + recalcul `npmDepsHash` (PROCEDURE §5).
2. `nix build` → `./result/bin/pgpilot-app`, lancé manuellement (PROCEDURE §7b).
3. `nix flake check` (PROCEDURE §7c).
4. `just ci` vert sur le dernier commit.
5. `manual_tests.md` exécuté par l'utilisateur.

### Phase 4 — Clôture

Merge par l'utilisateur, puis tag `v0.10.1`. Les PR #19, #29, #31, #32, #33, #34, #35,
#37, #38 se referment au scan Renovate suivant. **#36 reste ouverte** : c'est le rappel
de la dépendance upstream (D3).

## Décisions techniques

### D1 — #19, #31 et #37 forment un lot indivisible

Le CLI Tauri vérifie au build que le crate Rust et le paquet npm sont sur les mêmes
major/minor. Merger #31 ou #37 sans #19 produit :

```
Error Found version mismatched Tauri packages.
tauri (v2.11.5) : @tauri-apps/api (v2.12.0)
tauri-plugin-dialog (v2.7.1) : @tauri-apps/plugin-dialog (v2.8.0)
```

Renovate ne voit pas ce couplage : il range Rust et npm dans deux managers distincts.
`Cargo.lock` de #19 porte déjà tauri **2.12.0** et tauri-plugin-dialog **2.8.0**, donc
le lot est cohérent — mais uniquement une fois réuni.

**Ce mode de panne n'est détecté par aucun gate existant** : ni `just ci` (pas de
`cargo tauri build`), ni `ci.yml`, ni `nix flake check`. Il faut `nix build .#pgpilot`
ou `just build`. C'est la raison d'être de l'étape §7b de la PROCEDURE.

### D2 — #35 et #33 forment un second lot indivisible

Peers mesurés : `@astrojs/starlight@0.39.2` → `astro ^6.0.0` ;
`@astrojs/starlight@0.42.0` → `astro ^7.2.10` (+ `@astrojs/markdown-remark ^7.3.0`,
fourni par astro). Aucune des deux PR ne résout seule, d'où le double
`renovate/artifacts: FAILURE`.

À noter : le range `^7.0.0` de #35 est plus permissif que le `^7.2.10` exigé — il
résout vers 7.3.5, donc ça tombe juste, mais par chance. Le plan retient `^7.0.0`
tel que proposé, le lock figeant la version réelle.

### D3 — #36 (typescript 7) est bloquée upstream, on ne contourne pas

`@typescript-eslint/parser` plafonne à `typescript >=4.8.4 <6.1.0` — y compris dans sa
**dernière** version (8.70.1), vérifié sur le registre. Il n'existe donc aujourd'hui
aucune version de typescript-eslint compatible TS 7.

Testé avec `--legacy-peer-deps` pour isoler la cause :

- `tsc 7.0.2 --noEmit` → **exit 0**, le code du projet est déjà propre en TS 7 ;
- `eslint src` → crash dur, `TypeError: Cannot read properties of undefined (reading 'Cjs')`
  dans `@typescript-eslint/typescript-estree`.

Le blocage est donc **entièrement dans l'outillage de lint**, pas dans le code. Forcer
avec `--legacy-peer-deps` livrerait un dépôt où le lint ne tourne plus : refusé.

**PR laissée ouverte** plutôt que fermée : la fermer ferait ignorer l'update par
Renovate et on perdrait le signal quand typescript-eslint publiera son support TS 7.

### D4 — Déclarer la collection `i18n` plutôt que tolérer le warning

starlight 0.42 cherche une collection `i18n` dès que `locales` est configuré (le site est
bilingue EN/FR). Sans elle : `[WARN] [content] The collection "i18n" does not exist or is
empty`. Le build aboutit et le français reste correct, mais on part d'une base à 0
warning (phase 0) et on la garde.

Déclarer la collection seule déplace le warning
(`[starlight-i18n-loader] The base directory … does not exist`) : il faut **aussi** le
répertoire. Deux dictionnaires `{}` suffisent — aucune chaîne UI n'est surchargée
aujourd'hui, et le fichier devient le point d'entrée naturel si le besoin arrive.

### D5 — `MODULE_LEVEL_DIRECTIVE` : bruit amont, non actionnable

astro 7 embarque vite 8 / rolldown, qui émet un warning par fichier `.mdx` sur la
directive `"use astro:head-inject"` — directive générée par **astro lui-même**, pas par
le projet. 24 occurrences, aucune action possible côté pgpilot. Exclu du critère
d'acceptation de la phase 2.

### D6 — Ne jamais régénérer un lockfile que Renovate a su produire

npm local 10.9.7 vs npm 11 chez Renovate : un `npm install` local retire les champs
`libc` des paquets optionnels et crée du churn étranger à la MAJ (mesuré en phase 0).
On cherry-pick les commits Renovate tels quels. La seule exception est la phase 2, où
Renovate a justement échoué à produire le lock.

### D7 — Le warning `404` reste, il est préexistant et hors périmètre

Un seul message subsiste au build de la phase 2 :
`[WARN] [content] Entry docs → 404 was not found`.

Il n'est **pas** une régression : la même condition existe sur astro 6 (aucun `404.md`
dans la collection `docs`, starlight retombe sur sa page 404 intégrée). astro 7 se
contente de la remonter en `[WARN]` là où astro 6 l'émettait en ligne de log ordinaire.
`dist/404.html` est bien généré.

Le critère d'acceptation de la phase 2 se lit donc « **aucun warning nouveau** », pas
« zéro ligne de warning ». Ajouter une vraie page 404 personnalisée est un travail de
contenu bilingue, à faire dans un chantier docs dédié, pas dans un bump de dépendances.
