# Axe 4 — Desktop assets (icône + .desktop)

## Objectif

Créer l'icône applicative et le fichier `.desktop` dans des emplacements qui reflètent
la hiérarchie d'installation standard (freedesktop.org / XDG), afin que le flake NixOS
externe puisse les installer directement avec `install -Dm644 ...`.

Ce repo ne contient **pas** de flake de packaging — il fournit uniquement les assets.

---

## Emplacements cibles dans le repo

```
share/
├── applications/
│   └── pgpilot.desktop
└── icons/
    └── hicolor/
        └── scalable/
            └── apps/
                └── pgpilot.svg
```

Ces chemins mirrorent exactement `$out/share/...` dans un package Nix standard.
Le flake externe n'aura qu'à faire :

```nix
install -Dm644 share/applications/pgpilot.desktop \
  $out/share/applications/pgpilot.desktop
install -Dm644 share/icons/hicolor/scalable/apps/pgpilot.svg \
  $out/share/icons/hicolor/scalable/apps/pgpilot.svg
```

---

## T4.1 — Créer l'icône SVG

**Complexité** : M  
**Agent** : `voltagent-core-dev:ui-designer`  
**Dépendances** : aucune

### Contraintes

- Format : **SVG** (scalable — une seule source, pas besoin de PNG multi-résolutions)
- Taille de dessin : viewport `0 0 128 128` (convention courante)
- Style : cohérent avec le thème **Catppuccin Frappé** de l'app
  - Fond : Mantle `#292c3c` ou transparent
  - Accent : Mauve `#ca9ee6`
  - Texte / formes : Surface 2 `#626880`, Text `#c6d0f5`
- Concept visuel : clef GPG / cadenas / clef stylisée — simple, lisible à 16 px
- Pas de dégradés complexes ni d'effets flou (rendu correct à petite taille obligatoire)
- Fichier autoportant (pas de dépendances externes, pas de `<image>` embedded)

### Fichier à créer

`share/icons/hicolor/scalable/apps/pgpilot.svg`

### Vérification

- Ouvrir dans un visionneur SVG et vérifier le rendu à 16 px, 32 px, 48 px, 128 px
- Valider le SVG (pas d'erreurs XML) : `xmllint --noout share/icons/hicolor/scalable/apps/pgpilot.svg`

**Commit** : `feat(assets): add application SVG icon`

---

## T4.2 — Créer le fichier .desktop

**Complexité** : S  
**Agent** : `voltagent-lang:rust-engineer` (ou tout agent)  
**Dépendances** : T4.1 (pour que le nom `Icon=` soit cohérent)

### Contenu `share/applications/pgpilot.desktop`

```ini
[Desktop Entry]
Type=Application
Name=pgpilot
GenericName=GPG Key Manager
Comment=Manage your GPG keys — create, import, export, sign, encrypt
Exec=pgpilot
Icon=pgpilot
Categories=Utility;Security;
Keywords=gpg;pgp;encryption;keys;security;yubikey;
StartupNotify=true
```

### Règles

- `Icon=pgpilot` (sans extension ni chemin — le DE résout via hicolor theme)
- `Exec=pgpilot` (sans chemin absolu — le binaire doit être dans `$PATH` après installation)
- Pas de `Path=` ni de `Terminal=true`
- Encodage : UTF-8, fin de ligne LF

### Vérification

```bash
desktop-file-validate share/applications/pgpilot.desktop
```

(`desktop-file-utils` disponible sur la plupart des systèmes Linux ; skip si absent)

**Commit** : `feat(assets): add .desktop entry file`

---

## T4.3 — Documenter pour le flake externe

**Complexité** : S  
**Agent** : `voltagent-biz:technical-writer`  
**Dépendances** : T4.1, T4.2

Ajouter une section dans `README.md` du projet (ou un fichier `PACKAGING.md`) :

```markdown
## Packaging

The following assets are provided for system integration:

| File | Install path |
|------|-------------|
| `share/applications/pgpilot.desktop` | `$out/share/applications/pgpilot.desktop` |
| `share/icons/hicolor/scalable/apps/pgpilot.svg` | `$out/share/icons/hicolor/scalable/apps/pgpilot.svg` |

A Nix package can install them with:
```nix
install -Dm644 share/applications/pgpilot.desktop \
  $out/share/applications/pgpilot.desktop
install -Dm644 share/icons/hicolor/scalable/apps/pgpilot.svg \
  $out/share/icons/hicolor/scalable/apps/pgpilot.svg
```
```

**Commit** : `docs: document desktop assets for downstream packaging`

---

## Fichiers créés

```
share/applications/pgpilot.desktop
share/icons/hicolor/scalable/apps/pgpilot.svg
```

(Optionnel futur : PNG raster `share/icons/hicolor/256x256/apps/pgpilot.png` si un DE
ne supporte pas les SVG — non nécessaire pour NixOS/KDE/GNOME modernes)

## Critères d'acceptation

- [ ] SVG valide (xmllint OK), rendu lisible à 16 px
- [ ] `.desktop` valide (`desktop-file-validate` OK)
- [ ] `Icon=pgpilot` résout vers le SVG installé via hicolor theme
- [ ] Section packaging dans README ou PACKAGING.md
- [ ] Aucune modification du `flake.nix` de ce repo requise
