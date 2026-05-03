# Modèle de menaces — pgpilot

## Portée

- Gestionnaire de clefs OpenPGP, application desktop single-user.
- Délègue toutes les opérations cryptographiques à GnuPG (binaire externe).
- Ne manipule jamais directement le matériel cryptographique (clefs privées) : pgpilot
  n'accède ni ne lit les clefs privées stockées dans `gpg-agent`.

---

## Actifs à protéger

| Actif | Description |
|---|---|
| **Clefs privées** | Stockées dans `gpg-agent`, jamais lues par pgpilot. |
| **Keyring GnuPG** | Ensemble des clefs publiques et de l'état de confiance (`TrustLevel`). |
| **Toile de confiance** | Niveaux Marginal / Full / Ultimate attribués par l'utilisateur ; pilote les décisions de chiffrement. |

---

## Menaces identifiées

### 1. Clef malveillante importée

**Description** : un attaquant contrôle le UID affiché dans pgpilot. Le champ
`signer_name` provient du token `GOODSIG` de GnuPG, qui n'est pas authentifié
cryptographiquement — seule la signature elle-même l'est. Un attaquant peut
donc créer une clef avec un UID imitant une identité connue.

**Statut : Atténuée**

- Le badge de confiance (`TrustLevel`) est affiché systématiquement à côté de
  chaque clef dans la liste et le panneau de détail.
- Les clefs dont le niveau est `Undefined` n'affichent pas de badge vert.
- L'utilisateur est responsable de vérifier l'empreinte par un canal secondaire
  avant d'élever la confiance.

---

### 2. MitM sur keyserver HTTP

**Description** : un attaquant positionné en homme-du-milieu substitue une
clef différente lors d'une importation depuis un serveur de clefs.

**Statut : Atténuée**

- HTTPS uniquement sur tous les appels réseau (`ureq` avec TLS activé par
  défaut).
- Les certificats TLS sont vérifiés par la pile de confiance du système
  d'exploitation.
- L'importation affiche toujours l'empreinte complète (40 hex) de la clef
  reçue, permettant une vérification manuelle.

---

### 3. OOM via réponse réseau surdimensionnée

**Description** : un serveur de clefs malveillant (ou compromis) retourne une
réponse de plusieurs gigaoctets, provoquant un épuisement de la mémoire vive
(`Out Of Memory`).

**Statut : Atténuée**

- Limite de 1 Mio appliquée sur toutes les lectures réseau dans `gpg/keyring.rs`
  (`read_to_string` limité via `take(1 << 20)`).
- Au-delà de cette limite, la lecture est interrompue et une erreur explicite
  est renvoyée à l'utilisateur.

---

### 4. Injection de paramètres HKP

**Description** : une requête vers un serveur HKP construite avec un
fingerprint ou une adresse e-mail non encodée permet à un attaquant de forger
l'URL et d'injecter des paramètres arbitraires.

**Statut : Atténuée**

- Validation stricte à l'entrée : 40 hex (fingerprint complet), 16 hex (key ID
  long) ou format e-mail reconnu.
- Encodage URL complet de tous les paramètres avant construction de la requête
  HTTP.
- Les valeurs non conformes sont rejetées avec un message d'erreur explicite,
  sans tentative de correction silencieuse.

---

### 5. Fingerprint court — attaque Evil32

**Description** : les identifiants courts sur 8 caractères (32 bits) sont
triviallement collisibles. Un attaquant peut générer une clef dont les 8
derniers caractères de l'empreinte coïncident avec une clef ciblée.

**Statut : Atténuée**

- Validation de 40 caractères hexadécimaux (160 bits) obligatoire sur toutes
  les fonctions du module `gpg/`.
- Les identifiants 16 hex (key ID long, 64 bits) sont acceptés uniquement pour
  la recherche sur keyserver, jamais pour les opérations cryptographiques
  directes.
- `key_id` dans `KeyInfo` et `SubkeyInfo` est systématiquement le long ID
  (16 derniers hex de l'empreinte).

---

### 6. Fichier forgé via drag-and-drop

**Description** : un fichier malveillant (p. ex. bombes de décompression,
contenu non sollicité) est déposé par l'utilisateur dans la zone de
chiffrement. pgpilot le transmet à GnuPG sans inspection préalable.

**Statut : Résiduelle**

- pgpilot chiffre ce que l'utilisateur dépose explicitement ; il n'interprète
  pas le contenu du fichier.
- La vérification de la légitimité du fichier avant chiffrement est à la charge
  de l'utilisateur.
- GnuPG ne décompresse ni n'exécute le contenu ; le risque est limité à
  l'exposition de données non souhaitées.

---

### 7. Presse-papiers exposé

**Description** : une clef publique armoriée, une URL `paste.rs` ou toute autre
donnée copiée dans le presse-papiers reste accessible à tous les processus de
la session X11 ou Wayland jusqu'à remplacement.

**Statut : Résiduelle**

- Sur Wayland, la durée de vie du presse-papiers est liée au focus de la
  fenêtre source ; un effacement automatique fiable n'est pas possible sans
  contrôle du compositeur.
- Sur X11, aucun mécanisme standard n'empêche un autre processus de lire le
  presse-papiers.
- pgpilot n'y copie jamais de matériel privé (clef secrète, passphrase).

---

### 8. Enregistrement d'écran (screen recording)

**Description** : un logiciel d'enregistrement d'écran (légitime ou malveillant)
capture les données affichées : empreintes, UIDs, statuts de confiance, contenu
de fichiers sélectionnés.

**Statut : Hors-scope**

- La protection contre l'enregistrement d'écran relève du niveau OS /
  compositeur.
- pgpilot n'affiche pas de matériel privé (clefs secrètes, passphrases).

---

### 9. LD_PRELOAD / injection d'environnement

**Description** : une bibliothèque malveillante injectée via `LD_PRELOAD` ou
une variable d'environnement altérée dans les sous-processus `gpg` pourrait
intercepter ou modifier les opérations cryptographiques.

**Statut : Atténuée**

- `env_clear()` est appelé sur chaque `Command` gpg, puis seules les variables
  nécessaires (`GNUPGHOME`, `PATH`) sont transmises explicitement.
- L'environnement hérité du processus parent (potentiellement compromis) est
  ainsi écarté.

---

### 10. GNUPGHOME non défini ou inattendu

**Description** : si `GNUPGHOME` n'est pas défini, GnuPG utilise le répertoire
par défaut (`~/.gnupg`), ce qui peut provoquer des opérations sur un keyring
inattendu dans des environnements de test ou de CI.

**Statut : Atténuée**

- La fonction `gnupg_dir()` (`gpg/mod.rs`) est appelée en tête de chaque
  opération gpg et retourne une erreur explicite si la variable n'est pas
  définie.
- Aucune opération ne se replie silencieusement sur un chemin par défaut.

---

## Hors-scope

Les menaces suivantes sont reconnues mais ne font pas partie du périmètre de
pgpilot :

- **Compromission de la machine locale** — OS, RAM, disque (keylogger, rootkit,
  accès physique).
- **Attaque physique sur YubiKey / smartcard** — extraction du matériel
  cryptographique par side-channel ou décapsulation.
- **Compromission de la chaîne TLS** — autorité de certification compromise ou
  attaque HTTPS à grande échelle.
- **Attaques par canal auxiliaire sur le matériel** — timing, consommation
  électrique, rayonnement électromagnétique.

---

## Hypothèses de sécurité

Le modèle de menaces repose sur les hypothèses suivantes. Si l'une d'elles est
violée, les atténuations décrites peuvent devenir inefficaces.

| Hypothèse | Justification |
|---|---|
| Le binaire `gpg` dans `PATH` est de confiance | pgpilot délègue toute la cryptographie à GnuPG. |
| `gpg-agent` n'est pas compromis | Les clefs privées y sont stockées et protégées. |
| `pinentry` fonctionne correctement et n'est pas remplacé | La saisie de passphrase doit être isolée du reste de la session. |
| L'OS n'est pas compromis au moment de l'exécution | Les primitives de sécurité (mémoire, processus, filesystem) doivent être fiables. |
