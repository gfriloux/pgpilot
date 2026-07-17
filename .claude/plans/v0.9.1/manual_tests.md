# Tests manuels — v0.9.1

## Contexte

Cette version corrige uniquement le crash EGL au démarrage de l'AppImage.
Les tests manuels se concentrent sur la vérification du démarrage et de
l'absence de régression visible dans les flux principaux.

---

## 1. Test principal — AppImage démarre sans crash EGL

> À exécuter sur une machine **non-Ubuntu** (Arch Linux, NixOS, Fedora…).

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 1 | Télécharger `PGPilot_0.9.1_amd64.AppImage` depuis la release GitHub | Fichier présent |
| 2 | `chmod +x PGPilot_0.9.1_amd64.AppImage` | — |
| 3 | `./PGPilot_0.9.1_amd64.AppImage` (ou via `appimage-run` sur NixOS) | L'app s'ouvre, **aucun message `EGL_BAD_PARAMETER`**, aucun abort |
| 4 | La fenêtre principale s'affiche avec la liste de clefs | Interface visible et fonctionnelle |
| 5 | Vérifier `WEBKIT_DISABLE_DMABUF_RENDERER` : `env \| grep WEBKIT` dans un terminal lancé depuis l'app | Non applicable (env var interne au processus) |

---

## 2. Vérification de la variable d'environnement

> À exécuter avec le binaire local (`just build-bin`) ou via le binaire Nix.

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 1 | Lancer l'app normalement | Démarre sans erreur EGL |
| 2 | `WEBKIT_DISABLE_DMABUF_RENDERER=0 ./pgpilot-app` | L'app respecte l'override utilisateur (peut crasher sur systèmes avec EGL cassé — comportement attendu : l'override est respecté) |

---

## 3. Régression — Flux principaux

> À exécuter après confirmation du démarrage. Ne pas répéter si déjà validé en v0.9.0.

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 1 | Page **Mes clefs** : liste visible, clic sur une clef affiche le détail | Pas de régression |
| 2 | Page **Chiffrer** : sélection d'un destinataire, chiffrement d'un fichier test | Pas de régression |
| 3 | Page **Déchiffrer** : déchiffrement du fichier produit à l'étape précédente | Pas de régression |
| 4 | Page **Santé** : les checks s'affichent | Pas de régression |
| 5 | Thème USSR : switch depuis Paramètres, vérifier que l'UI se met à jour | Pas de régression |

---

## 4. Vérification version

| # | Action | Résultat attendu |
|---|--------|-----------------|
| 1 | Page **Paramètres** → version affichée | `0.9.1` |
| 2 | Titre de la fenêtre ou About | `PGPilot 0.9.1` ou similaire |
