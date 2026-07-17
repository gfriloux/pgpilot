# Tests manuels v0.9.0

Ce fichier est enrichi après chaque phase qui ajoute un comportement utilisateur.
À exécuter intégralement en Phase 8 (validation finale).

---

## Phase 1 — Sécurité GPG subprocess

### Import de clef avec chemin malformé
| Action | Résultat attendu | ✓ |
|---|---|---|
| Tenter d'importer un fichier dont le nom commence par `-` (si possible via API) | gpg refuse proprement, toast d'erreur affiché, pas de crash | |
| Import d'une clef PGP valide via le dialog fichier | Import réussi normalement | |

### Verify signature
| Action | Résultat attendu | ✓ |
|---|---|---|
| Vérifier une signature valide | Résultat "valide" avec fingerprint du signataire | |
| Vérifier une signature invalide (fichier modifié) | Résultat "invalide" clairement affiché | |
| Vérifier avec une clef inconnue | Message "clef inconnue" ou équivalent | |

### Decrypt
| Action | Résultat attendu | ✓ |
|---|---|---|
| Déchiffrer un fichier chiffré pour soi | Déchiffrement réussi, fichier écrit | |
| Tenter de déchiffrer un fichier chiffré pour un autre | Message d'erreur "pas la bonne clef" | |

### Backup clef secrète
| Action | Résultat attendu | ✓ |
|---|---|---|
| Backup d'une clef secrète dans un dossier valide | Fichier créé avec permissions 600 | |
| Backup dans un dossier en lecture seule | Toast d'erreur explicite (plus de silence) | |

---

## Phase 2 — Bugs critiques frontend

### Thème
| Action | Résultat attendu | ✓ |
|---|---|---|
| Basculer en thème USSR | Interface USSR correctement appliquée | |
| Revenir en thème Catppuccin | Interface Catppuccin correctement appliquée | |
| Ouvrir DevTools, inspecter `<html>` | Classe `theme-ussr` présente en USSR, absente en Catppuccin. L'attribut `data-theme` n'est plus écrit du tout. | |

---

## Phase 5 — Frontend Zustand

### Suppression de room chat
| Action | Résultat attendu | ✓ |
|---|---|---|
| Supprimer une room (si backend échoue) | La room reste visible dans la liste (rollback) | |
| Supprimer une room (succès normal) | La room disparaît de la liste | |

---

## Phase 8 — Validation finale complète

### Navigation générale
| Action | Résultat attendu | ✓ |
|---|---|---|
| Lancer l'app | Démarrage sans erreur, My Keys affichée | |
| Naviguer dans toutes les sections | Pas de page blanche, pas de crash console | |

### Mes clefs
| Action | Résultat attendu | ✓ |
|---|---|---|
| Sélectionner une clef avec secret | Bouton Backup visible | |
| Sélectionner une clef sans secret | Bouton Backup absent | |
| Voir le détail d'une clef | Fingerprint, UID, sous-clefs affichés correctement | |

### Chiffrement / Signature / Vérification
| Action | Résultat attendu | ✓ |
|---|---|---|
| Chiffrer un fichier pour un destinataire | Fichier chiffré créé | |
| Signer un fichier | Fichier .asc créé | |
| Vérifier la signature créée | Résultat "valide" | |
| Déchiffrer le fichier chiffré | Contenu original récupéré | |

### Health
| Action | Résultat attendu | ✓ |
|---|---|---|
| Ouvrir la page Health | Checks affichés avec statuts | |

### Thèmes
| Action | Résultat attendu | ✓ |
|---|---|---|
| Basculer USSR → Catppuccin → USSR | Transitions correctes, persistance après reload | |

### Version
| Action | Résultat attendu | ✓ |
|---|---|---|
| Vérifier `Cargo.toml` racine | `version = "0.9.0"` | |
| Vérifier `app/package.json` | `"version": "0.9.0"` | |
| Vérifier `app/src-tauri/tauri.conf.json` | `"version": "0.9.0"` | |
| Vérifier `packages/pgpilot/default.nix` | `version = "0.9.0"` | |
