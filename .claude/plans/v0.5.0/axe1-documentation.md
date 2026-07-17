# Axe 1 — Documentation (mdbook + GitHub Pages)

## Objectif

Créer un site de documentation hébergé sur GitHub Pages via mdbook, couvrant les 19 fonctionnalités
implémentées du roadmap, avec déploiement automatique via GitHub Actions.

URL cible : `https://gfriloux.github.io/pgpilot/`

---

## T1.1 — Scaffolding mdbook + workflow deploy

**Complexité** : M  
**Agent** : `voltagent-infra:deployment-engineer`  
**Dépendances** : aucune

### Ce qui est à faire

1. Créer `book/` à la racine du repo :
   ```
   book/
   ├── book.toml
   └── src/
       ├── SUMMARY.md
       └── 0-intro.md   (placeholder)
   ```

2. `book/book.toml` minimal :
   ```toml
   [book]
   title = "pgpilot"
   authors = ["Guillaume Friloux"]
   language = "en"
   src = "src"

   [output.html]
   site-url = "/pgpilot/"
   git-repository-url = "https://github.com/gfriloux/pgpilot"
   edit-url-template = "https://github.com/gfriloux/pgpilot/edit/main/book/src/{path}"
   ```

3. Créer `.github/workflows/deploy-docs.yml` :
   ```yaml
   name: Deploy docs
   on:
     push:
       branches: [main]
   permissions:
     contents: write
   jobs:
     deploy:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: peaceiris/actions-mdbook@v2
           with:
             mdbook-version: latest
         - run: mdbook build book
         - uses: peaceiris/actions-gh-pages@v4
           with:
             github_token: ${{ secrets.GITHUB_TOKEN }}
             publish_dir: ./book/book
   ```

4. Activer GitHub Pages sur le repo : source → `gh-pages` branch (instruction dans PR description)

**Commit** : `chore(docs): initialize mdbook structure and deploy workflow`

---

## T1.2 — Getting started + installation + quickstart

**Complexité** : M  
**Agent** : `voltagent-biz:technical-writer`  
**Dépendances** : T1.1

### Fichiers à créer

- `book/src/SUMMARY.md` — table des matières complète
- `book/src/0-intro.md` — pitch pgpilot (1 page, pourquoi pgpilot, pas un remplacement de gpg cli)
- `book/src/1-installation.md` :
  - Prérequis : `gpg` ≥ 2.2, `gpg-agent`, `pinentry`
  - Installation binaire (GitHub Releases)
  - Build from source (Rust 1.80+, `nix develop`)
  - Vérification : lancer pgpilot, ouvrir page Diagnostic
- `book/src/2-quickstart.md` :
  - Créer sa première clef en 5 étapes (screenshots)
  - Exporter la clef publique
  - Importer la clef d'un contact

---

## T1.3 — Guide : gestion des clefs

**Complexité** : L  
**Agent** : `voltagent-biz:technical-writer`  
**Dépendances** : T1.2

### Fichier : `book/src/3-key-management.md`

Sections :
- Créer une clef (algo ed25519, structure cert-only + subkeys S/E/A, pas d'expiration volontaire)
- Naviguer dans la liste de clefs (liste gauche + panneau détail droit)
- Affichage confiance (badge Undefined / Marginal / Full)
- Modifier le niveau de confiance
- Subkeys : afficher, ajouter, renouveler, faire pivoter (rotate)
- Exporter clef publique (fichier / presse-papiers / lien paste.rs)
- Importer clef (fichier, URL, keyserver par fp/ID/email, coller texte armored)
- Supprimer clef (public / secret / stub YubiKey)
- Backup clef privée + certificat de révocation

---

## T1.4 — Guide : keyserver + YubiKey + opérations fichiers

**Complexité** : L  
**Agent** : `voltagent-biz:technical-writer`  
**Dépendances** : T1.2

### Fichiers

`book/src/4-keyserver.md` :
- Publier sur keys.openpgp.org vs keyserver.ubuntu.com
- Badge de statut de publication
- Auto-republication après rotation
- Lien de partage (paste.rs)

`book/src/5-file-operations.md` :
- Chiffrer fichiers (mono-destinataire, multi-destinataires)
- Trust override (`--trust-model always`) et modal d'avertissement
- Toggle format `.gpg` vs `.asc`
- Drag & drop (X11 ; Wayland selon compositeur)
- Signer un fichier (produit `<fichier>.sig`)
- Vérifier une signature (5 états : Valid / BadSig / UnknownKey / ExpiredKey / RevokedKey)

`book/src/6-smartcard.md` :
- Détecter YubiKey / smartcard (icône dans sidebar)
- Migrer une subkey vers la carte
- Limitations et notes de sécurité (clef non récupérable depuis la carte)

---

## T1.5 — FAQ + troubleshooting + sécurité

**Complexité** : M  
**Agent** : `voltagent-biz:technical-writer`  
**Dépendances** : T1.2

### Fichiers

`book/src/7-faq.md` :
- Quelle différence entre clef publique et privée ?
- Pourquoi mes clefs pgpilot n'expirent-elles pas ?
- Fingerprint vs Key ID ?
- Pourquoi des subkeys séparées (Sign / Encr / Auth) ?
- Comment révoquer une clef ?

`book/src/8-troubleshooting.md` :
- `gpg: not found` → path, Nix
- pinentry fails / blank dialog → `pinentry-gtk`, Wayland, `GPG_TTY`
- Keyserver connection error → DNS, pare-feu, HKP port 11371 vs HTTPS
- YubiKey not detected → `pcscd` running, `gpg --card-status`
- File dialog hangs on Wayland → `libdbus` requis, Nix shell
- Renvoyer vers page Diagnostic (`View::Health`)

`book/src/9-security.md` :
- Résumé du modèle de menace (source : `THREAT_MODEL.md`)
- Bonnes pratiques : backup clef privée, certificat de révocation, YubiKey
- Limitations : pgpilot délègue toute la crypto à `gpg` (pas de crypto maison)
- Signalement de vulnérabilités (SECURITY.md)

---

## T1.6 — Review + polish

**Complexité** : M  
**Agent** : `voltagent-biz:technical-writer`  
**Dépendances** : T1.2, T1.3, T1.4, T1.5

- Cohérence de style et ton (tutoriels en anglais)
- Screenshots à jour (prendre après merge axe 2 i18n pour cohérence visuelle)
- Vérifier tous les liens internes (`mdbook test`)
- `mdbook serve` local — vérifier navigation, responsive mobile
- Ajouter `book/src/CHANGELOG.md` (lien vers CHANGELOG.md à la racine)

---

## T1.7 — Merge + déploiement initial

**Complexité** : S  
**Agent** : `voltagent-infra:deployment-engineer`  
**Dépendances** : T1.1, T1.6

- Push vers `main` → GitHub Actions build mdbook → deploy vers `gh-pages`
- Vérifier site accessible publiquement
- Commit : `docs: publish v0.5.0 user documentation`

---

## Nouveaux fichiers créés

```
book/
├── book.toml
└── src/
    ├── SUMMARY.md
    ├── 0-intro.md
    ├── 1-installation.md
    ├── 2-quickstart.md
    ├── 3-key-management.md
    ├── 4-keyserver.md
    ├── 5-file-operations.md
    ├── 6-smartcard.md
    ├── 7-faq.md
    ├── 8-troubleshooting.md
    └── 9-security.md
.github/workflows/deploy-docs.yml
```

## Critères d'acceptation

- [ ] `mdbook build book` réussit sans erreurs ni warnings
- [ ] Site déployé sur `https://gfriloux.github.io/pgpilot/`
- [ ] Workflow GitHub Actions `deploy-docs.yml` trigger et déploie automatiquement
- [ ] Tous les liens internes OK (`mdbook test`)
- [ ] Navigation mobile fonctionnelle
- [ ] 19 fonctionnalités du roadmap couvertes
