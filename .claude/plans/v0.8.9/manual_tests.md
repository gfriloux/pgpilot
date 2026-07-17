# Tests manuels — v0.8.9

Exécuter avant le tag. Cocher chaque case au fur et à mesure.
Environnement requis : pgpilot lancé en mode réel (`just dev`), avec un keyring GPG peuplé.

---

## Prérequis

- [ ] Au moins une clé secrète dans le keyring (`My Keys`)
- [ ] Au moins une clé publique importée (`Public Keys`), trust ≠ ultimate
- [ ] Le file picker KDE Plasma fonctionne (`GTK_USE_PORTAL=1` actif — cf. `nix develop`)

---

## 1 — Export clé publique (My Keys)

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 1.1 | Aller sur My Keys, sélectionner une clé | Le panneau KeyDetail s'affiche |
| 1.2 | Cliquer sur "Export" | Le directory picker Plasma s'ouvre |
| 1.3 | Choisir un dossier et valider | Le picker se ferme, toast "Exported to …/`<last16chars>.asc`" visible |
| 1.4 | Vérifier que le fichier `.asc` existe dans le dossier choisi | Fichier présent, contenu PGP armor valide |
| 1.5 | Cliquer sur "Export", puis annuler le picker | Rien ne se passe, pas de toast d'erreur |

## 2 — Export clé publique (Public Keys)

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 2.1 | Aller sur Public Keys, sélectionner une clé | Le panneau KeyDetail s'affiche |
| 2.2 | Cliquer sur "Export" | Le directory picker Plasma s'ouvre |
| 2.3 | Choisir un dossier et valider | Toast "Exported to …" visible |
| 2.4 | Annuler le picker | Rien ne se passe |

## 3 — Backup clé secrète (My Keys)

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 3.1 | Aller sur My Keys, cliquer sur "Backup" pour une clé | Le directory picker Plasma s'ouvre |
| 3.2 | Pendant que le picker est ouvert | Le bouton Backup est désactivé (loading) |
| 3.3 | Choisir un dossier et valider | Le picker se ferme, toast "Backup saved: …" avec la liste des fichiers |
| 3.4 | Vérifier le dossier | Fichiers de backup présents (clé secrète + cert de révocation) |
| 3.5 | Faire un backup deux fois sur le même dossier | Erreur "file already exists" — pas d'écrasement silencieux |
| 3.6 | Annuler le picker | Rien ne se passe, pas de toast d'erreur |

## 4 — Régressions file pickers existants

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 4.1 | Encrypt → "Select files" | File picker multi-sélection s'ouvre |
| 4.2 | Decrypt → "Select files" | File picker s'ouvre |
| 4.3 | Sign → "Select file" | File picker s'ouvre |
| 4.4 | Verify → "Select file" | File picker s'ouvre |

## 5 — Encrypt : clé à trust 'full'

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 5.1 | Importer une clé, lui donner trust 'full' via `gpg --import-ownertrust` | Clé visible dans Public Keys avec trust full |
| 5.2 | Aller sur Encrypt, sélectionner un fichier | Fichier sélectionné |
| 5.3 | Sélectionner la clé full-trust comme destinataire | Pas de modal d'avertissement untrusted |
| 5.4 | Chiffrer | Chiffrement réussit sans erreur GPG |

## 6 — Toast system

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 6.1 | Déclencher une action qui réussit (ex: Export) | Toast vert en bas à droite, disparaît après ~4 s |
| 6.2 | Cliquer sur le toast | Il disparaît immédiatement |
| 6.3 | Déclencher une erreur (ex: Export vers chemin invalide) | Toast rouge visible |

## 7 — Nix : binaire installé

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 7.1 | `nix build` (sans argument) | Produit `./result/bin/pgpilot-app` |
| 7.2 | `./result/bin/pgpilot-app` | L'application démarre |
| 7.3 | File pickers fonctionnent dans le binaire Nix | Même comportement que `just dev` |

---

*Enrichir ce fichier à chaque phase qui ajoute un comportement utilisateur.*
