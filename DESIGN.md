# PGPilot — Design Document (Source de vérité)

Ce document définit les règles fondamentales de PGPilot.
Toute demande de fonctionnalité ou correction doit être validée contre ce document.
Si une demande contredit une règle, on challenge la règle — on ne fait pas de hotfix.

---

## Philosophie

PGPilot est un gestionnaire GPG **opiniated** : il impose des pratiques solides
plutôt que d'exposer toute la flexibilité de GnuPG.

L'objectif est de rendre accessible une utilisation saine de GPG, conforme aux
recommandations ANSSI ([PG-083](https://messervices.cyber.gouv.fr/documents-guides/anssi-guide-mecanismes-crypto-3.00.pdf) — *Règles et recommandations concernant le choix
et le dimensionnement des mécanismes cryptographiques*, mars 2026), à des
utilisateurs qui ne sont pas experts en cryptographie.
PGPilot prend des décisions à leur place pour éviter les erreurs courantes.

Deux principes fondamentaux :
- **Toute opération cryptographique passe par le binaire `gpg` en subprocess** — jamais de crypto inline.
- **Les opérations GPG sont exposées, pas la complexité GPG** — l'utilisateur voit ce qui compte.

---

## Modèle de clef

### Structure obligatoire

Une clef PGPilot suit le modèle recommandé par l'ANSSI et la communauté OpenPGP :

```
[Clef primaire] — certification uniquement [C]
    ├── [Sous-clef S] — signature
    ├── [Sous-clef E] — chiffrement
    └── [Sous-clef A] — authentification
```

Invariants :
- La clef primaire a **exclusivement** le drapeau `[C]` (certify)
- Elle ne signe jamais un message, ne chiffre jamais, n'authentifie jamais directement
- Les 3 sous-clefs sont **obligatoires** — y compris `[A]`, même si non utilisée activement
- Chaque sous-clef a **exactement un usage**

### Algorithmes

Pour les nouvelles clefs créées dans PGPilot :
- Certification/Signature : **Ed25519** uniquement
- Chiffrement : **ECDH Curve25519** (X25519) uniquement
- RSA, DSA, ElGamal : **interdits pour la création**
- PGPilot choisit les algorithmes — l'utilisateur ne les sélectionne pas

Pour les clefs importées (legacy) :
- RSA ≥ 2048 bits : accepté, marqué comme legacy, incité à la rotation
- RSA < 2048 bits : refusé à l'import
- La politique de rotation encourage la migration progressive vers Ed25519 :
  les clefs RSA sont signalées comme priorité de rotation dans les diagnostics

> **Source :** ANSSI PG-083 — Ed25519 et ECDH Curve25519 figurent parmi les mécanismes
> recommandés. RSA < 2048 bits est explicitement déconseillé.

### Expiration

- **Sous-clefs : expiration obligatoire, durée maximale 2 ans** — conforme ANSSI PG-083
- **Clef primaire : expiration obligatoire, durée maximale 3 ans**
- PGPilot **bloque** la création si la durée dépasse ces seuils (erreur, pas avertissement)
- PGPilot avertit si une clef approche de l'expiration (< 90 jours) : badge dans la liste
- L'absence d'expiration est traitée comme une erreur dans les diagnostics Health

### Révocation

- Un certificat de révocation est **généré et sauvegardé** à la création de chaque clef primaire
- PGPilot refuse de terminer la création sans certificat exporté
- L'emplacement du certificat est affiché et mémorisé dans la vue clef

### Clef primaire et support physique

PGPilot ne force pas l'usage d'une YubiKey ou d'une carte à puce. Cependant :
- Si une YubiKey est détectée au démarrage, un **indicateur visible** est affiché dans la sidebar
- Si la clef primaire réside sur une YubiKey, cet état est mis en avant comme configuration "optimale"
- Si la clef primaire est uniquement en keyring local (`~/.gnupg`), un badge informatif le signale
  sans bloquer — mais les diagnostics Health le remontent en information

---

## Rotation de la clef primaire

La rotation de la clef primaire est l'opération la plus délicate du cycle de vie GPG.
PGPilot cible des débutants : son rôle est de les accompagner pas à pas, sans qu'ils
ne perdent l'accès à leurs anciens messages ni leur identité numérique.

### Pourquoi faire une rotation

Trois cas déclencheurs :
- **Expiration approchante** (< 90 jours) — cas le plus courant, planifiable
- **Migration legacy RSA → Ed25519** — clef importée ancienne génération
- **Compromission** — cas urgent, flux différent (révocation immédiate, pas de transition)

### Ce qui est préservé

| Élément | Comportement après rotation |
|---------|----------------------------|
| Anciens messages chiffrés | Déchiffrables si les anciennes sous-clefs secrètes sont conservées |
| Signatures passées | Restent valides avec l'ancienne clef |
| Identité numérique | Continuée via cross-signature entre les deux clefs |

### Ce qui n'est pas automatique

- Les certifications des tiers (web of trust) sur l'ancienne clef **ne se transfèrent pas** vers la nouvelle — elles doivent être redemandées
- Les correspondants qui ne mettent pas à jour leur keyring continueront à chiffrer vers l'ancienne clef pendant la période de grâce

### Séquence de rotation (cas normal — expiration ou migration)

```
1. Créer la nouvelle clef primaire Ed25519 + 3 sous-clefs dans PGPilot
2. Signer la nouvelle clef avec l'ancienne (cross-certification)
3. Signer l'ancienne clef avec la nouvelle (déclaration de transition mutuelle)
4. Exporter et publier les deux clefs sur le keyserver
5. Générer et partager une déclaration de transition (signée par les deux clefs)
6. Archiver l'ancienne clef : conserver les sous-clefs secrètes [E] pour le déchiffrement
   des anciens messages — ne pas supprimer
7. Période de grâce (recommandée : 6 mois) — les deux clefs coexistent
8. À l'issue de la période de grâce : révoquer l'ancienne clef primaire
```

### Rôle de PGPilot dans ce flow

PGPilot doit accompagner chaque étape :
- **Wizard guidé** — pas de manipulation manuelle de `gpg` pour le débutant
- **Vérification avant archivage** — s'assurer que les anciennes sous-clefs secrètes sont exportées
- **Génération de la déclaration de transition** — texte signé par les deux clefs, prêt à envoyer
- **État "archivée"** — l'ancienne clef reste visible dans l'UI en mode lecture seule, accessible uniquement pour le déchiffrement
- **Rappel période de grâce** — badge visible jusqu'à la révocation finale

### Cas compromission (flux urgent)

Ce flux **ne permet pas de période de grâce** :
```
1. Révoquer immédiatement l'ancienne clef (certificat de révocation)
2. Publier la révocation sur le keyserver
3. Créer la nouvelle clef primaire + sous-clefs
4. Notifier les correspondants en urgence
```
L'accès aux anciens messages chiffrés est perdu si les sous-clefs secrètes
étaient sur la clef compromise. PGPilot doit avertir explicitement de ce risque.

---

## Sécurité

### Gardes systématiques (backend Tauri)

Appliqués à chaque commande Tauri :
- `validate_fp(fp)` — **première ligne** de toute fonction prenant une empreinte (format : 40 caractères hex ASCII)
- `validate_keyserver_url(url)` — whitelist stricte : `keys.openpgp.org`, `keyserver.ubuntu.com`
- `canonicalize(path)` — toute opération sur un fichier, prévention du path traversal

Ces gardes ne sont pas optionnels et ne doivent pas être contournés, même pour les tests.

### IPC Tauri

- Aucune donnée sensible (passphrase, clef privée) ne transite en clair dans l'IPC
- Les commandes Tauri retournent des résultats (succès/erreur + métadonnées), jamais les secrets
- Type de retour : `Result<T, String>` — pas de panics

### Subprocess GPG

- Toute opération cryptographique passe par `gpg_command(homedir)` — jamais `Command::new("gpg")`
- Les clefs privées ne sont jamais lues par le code Rust — elles restent dans le keyring GPG
- La séparation des usages (signer ≠ chiffrer ≠ authentifier) est garantie par le modèle de clef,
  pas par une logique applicative

---

## Chat chiffré

- **Ephémère par conception** : aucun message n'est persisté sur disque
- 500 messages maximum par room (FIFO — les plus anciens supprimés en premier)
- Chiffrement via subprocess `gpg --encrypt --sign` / `gpg --decrypt` (pas de crypto inline)
- Les rooms persistent dans `~/.config/pgpilot/rooms.yaml` (métadonnées uniquement, jamais les messages)

---

## Règles UI non négociables

### File picker

Le file picker est la fonctionnalité la plus fragile de l'application.
Elle a cassé à de nombreuses reprises lors de changements de dépendances.

Règle fixée définitivement :
- **Utiliser exclusivement `tauri-plugin-dialog`** avec `default-features = false, features = ["xdg-portal"]`
- `GTK_USE_PORTAL=1` doit être présent dans l'environnement (fourni par le dev shell Nix)
- **Ne jamais revenir à `rfd` direct** — incompatible NixOS/Wayland/KDE hors portal
- Ne jamais implémenter un file picker custom

Si une mise à jour de `tauri-plugin-dialog` casse le file picker, investiguer le portal
avant toute autre solution de contournement.

### Thèmes

PGPilot a deux thèmes : **Catppuccin Frappé** (défaut, sombre) et **USSR** (clair, vintage soviétique).

- Toute modification UI doit être vérifiée dans les deux thèmes
- Les variables CSS de `theme.css` sont la source de vérité — jamais de couleur codée en dur dans un composant
- **`--detail-bg` est noir en USSR** — ne jamais l'utiliser comme fond de composant ;
  utiliser `--input-bg` ou `--card-bg` selon le contexte
- Les assets USSR (`public/banners/`) clipent via `overflow: hidden` sur les cards —
  ne pas ajouter de padding sur le container du banner

### Icônes Nerd Font

- **Plage FA4 uniquement : `\u{f000}` → `\u{f2e0}`**
- Au-delà de cette plage = icône invisible, sans erreur au build
- Vérifier le codepoint avant tout ajout d'icône

### Composants

- 12 composants de base réutilisables — ne pas créer de one-offs qui dupliquent leur logique
- CSS Modules uniquement — pas de Tailwind, pas de styles inline
- `exactOptionalPropertyTypes` activé — pattern spread obligatoire pour les props optionnelles :
  `{...(value !== undefined ? { prop: value } : {})}`

---

## Règles non négociables

| # | Règle | Raison |
|---|-------|--------|
| R1 | Clef primaire = `[C]` uniquement | Séparation des usages — ANSSI PG-083 |
| R2 | 3 sous-clefs `[S]` `[E]` `[A]` obligatoires | Séparation des usages |
| R3 | Ed25519 / ECDH Curve25519 uniquement pour les nouvelles clefs | Algorithmes recommandés ANSSI PG-083 |
| R4 | Expiration ≤ 2 ans (sous-clefs), ≤ 3 ans (primaire) — bloquant | ANSSI PG-083 |
| R5 | Certificat de révocation exporté à la création | Résilience, traçabilité |
| R6 | Toute crypto via subprocess `gpg`, jamais inline | Délégation aux primitives éprouvées |
| R7 | `validate_fp()` première ligne de toute commande Tauri avec empreinte | Sécurité IPC |
| R8 | Aucune donnée sensible en clair dans l'IPC Tauri | Sécurité |
| R9 | Chat = ephémère, RAM uniquement, 500 messages FIFO | Vie privée par conception |
| R10 | File picker = `tauri-plugin-dialog` + xdg-portal uniquement | Stabilité NixOS/Wayland/KDE |
| R11 | Les deux thèmes doivent fonctionner après chaque modification UI | Qualité produit |
| R12 | RSA < 2048 bits refusé à l'import ; RSA ≥ 2048 accepté mais legacy | ANSSI PG-083 |

---

## Ce que PGPilot ne fait pas (volontairement)

- **PQC (post-quantique)** — en attente de support stable dans GnuPG
- **Synchronisation de clefs entre machines** — hors scope
- **Persistance des messages chat** — ephémère par principe
- **Forcer l'usage d'une YubiKey** — recommandé, visible, mais non imposé
- **Créer des clefs RSA** — Ed25519 / ECDH Curve25519 uniquement pour les nouvelles clefs
- **Laisser l'utilisateur choisir l'algorithme** — PGPilot choisit pour garantir la conformité ANSSI
- **Exposer la flexibilité complète de GnuPG** — PGPilot est opiniated par conception
