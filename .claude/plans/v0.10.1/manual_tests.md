# Tests manuels — v0.10.1

Version de maintenance : seconde vague Renovate + migration docs astro 6 → 7.
Aucune fonctionnalité utilisateur ajoutée. Ces tests valident la non-régression après
les bumps et le rendu du site de documentation après la migration.

## 1. Portes qualité automatisées

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | `nix develop --command just ci` | Vert : fmt-check + clippy + tests Rust + 92 E2E | ☐ |
| 2 | `nix build && ./result/bin/pgpilot-app` | L'app démarre, fenêtre PGPilot, aucun crash | ☐ |
| 3 | `nix flake check` | `all checks passed!` | ☐ |
| 4 | `nix develop --command cargo audit` | Seules les advisories déjà ignorées (rustls-webpki, rustls-pemfile, quick-xml build-time) | ☐ |

> Note NixOS E2E : Playwright a besoin d'un Chromium *wrappé* dans `/nix/store`
> (`nix build nixpkgs#chromium` une fois).

## 2. Couplage npm ↔ Rust Tauri (décision D1)

C'est le point sensible de cette version : `just ci` **ne détecte pas** ce mode de panne.

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | `nix develop --command just build` | Produit `.deb` + `.rpm`. **Aucun** message `Found version mismatched Tauri packages` | ☐ |
| 2 | `grep -A1 '^name = "tauri"$' Cargo.lock` et `grep '"@tauri-apps/api"' app/package.json` | Même major/minor de part et d'autre (2.12.x) | ☐ |
| 3 | Idem pour `tauri-plugin-dialog` / `@tauri-apps/plugin-dialog` | Même major/minor (2.8.x) | ☐ |

## 3. Non-régression app après MAJ dépendances

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | `cd app && VITE_MOCK=true npm run dev` puis ouvrir l'URL affichée | L'app se charge | ☐ |
| 2 | Naviguer entre toutes les pages via la sidebar | Navigation fluide, aucun écran blanc (react 19.3.0) | ☐ |
| 3 | Basculer thème Catppuccin ↔ USSR (Settings) | Bascule immédiate, bannières USSR affichées | ☐ |
| 4 | Basculer langue EN ↔ FR | Textes traduits, persistance après reload | ☐ |
| 5 | Ouvrir une clef, vérifier le panneau détail (sous-clefs S/E/A) | Rendu correct | ☐ |
| 6 | **Dialogue de fichier** : page Import → bouton de sélection de fichier, et Sign/Encrypt → choix du fichier de sortie | Le dialogue **natif KDE** s'ouvre (portail XDG), pas le dialogue GTK. Test critique : `@tauri-apps/plugin-dialog` passe 2.7.1 → 2.8.0 | ☐ |
| 7 | Backup d'une clef vers un chemin choisi | Fichier écrit, permissions `0600` sur la clef secrète | ☐ |

> Le point 6 est le seul comportement utilisateur réellement touché par cette vague.
> Historique : le rendu du dialogue sur KDE a déjà cassé une fois (fix `xdg-portal` +
> `GTK_USE_PORTAL=1`), donc à revérifier à chaque bump du plugin.

## 4. Documentation — migration astro 7 + starlight 0.42 (phase 2)

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | `cd docs && npm ci && npm run build` | **25 pages**, **0 warning** (hors `MODULE_LEVEL_DIRECTIVE`, cf. D5) | ☐ |
| 2 | `npm audit` dans `docs/` | 0 vulnérabilité | ☐ |
| 3 | `just docs-dev`, ouvrir la racine `/` | Page d'accueil EN, sidebar complète (Getting Started / Features / Reference) | ☐ |
| 4 | Basculer vers `/fr/` via le sélecteur de langue | Site en français, sélecteur fonctionnel | ☐ |
| 5 | Sur une page `/fr/…`, vérifier les chaînes **d'interface** de starlight | « Sur cette page », « Rechercher » — pas leurs équivalents anglais. Valide la collection `i18n` (D4) | ☐ |
| 6 | Inspecter la source d'une page FR et d'une page EN | `<html lang="fr">` / `<html lang="en">` | ☐ |
| 7 | Utiliser la recherche (Pagefind), en EN puis en FR | Résultats pertinents dans les deux langues (index construit sur 25 fichiers HTML) | ☐ |
| 8 | Vérifier les captures d'écran d'une page qui en contient | Les 18 PNG de `public/screenshots/` s'affichent | ☐ |
| 9 | Ouvrir une URL inexistante, ex. `/pgpilot/nope/` | Page 404 starlight (le message `Entry docs → 404 was not found` au build est préexistant, cf. phase 0) | ☐ |
| 10 | Vérifier `base: '/pgpilot/'` : les liens internes et les assets | Aucun lien cassé, aucun 404 d'asset — le piège classique d'un bump astro majeur | ☐ |
| 11 | Après merge sur `main` : workflow « Deploy docs » | Vert, site publié sur GitHub Pages | ☐ |

## 5. Renovate (post-merge, côté GitHub)

| # | Action | Résultat attendu | OK |
|---|--------|------------------|----|
| 1 | Attendre le scan Renovate suivant (planifié « every weekend ») | #19, #29, #31, #32, #33, #34, #35, #37, #38 se referment automatiquement | ☐ |
| 2 | Vérifier **#36** (typescript 7) | **Reste ouverte** — c'est voulu (D3), elle sert de rappel sur le support TS 7 de typescript-eslint | ☐ |
| 3 | Sur #36, revérifier le blocage : `npm view @typescript-eslint/parser peerDependencies` | Si la borne `typescript <6.1.0` a bougé, le blocage est levé → nouveau plan | ☐ |
