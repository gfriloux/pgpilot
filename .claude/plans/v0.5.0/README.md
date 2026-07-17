# Plan v0.5.0 — pgpilot

## Vue d'ensemble

Quatre axes parallélisables. Timeline estimé **6–7 semaines** en parallélisant les phases.

- [Axe 1 — Documentation](axe1-documentation.md) (mdbook + GitHub Pages)
- [Axe 2 — Internationalisation](axe2-i18n.md) (EN/FR + config YAML persistante)
- [Axe 3 — Tests unitaires](axe3-tests.md) (gpg layer + app handlers)
- [Axe 4 — Desktop assets](axe4-desktop-assets.md) (icône SVG + .desktop pour packaging NixOS externe)

## Dépendances entre axes

```
Phase 1 (parallèle) : T1.1 + T2.1 + T3.1 + T4.1
Phase 2 (parallèle) : T1.2→T1.5  +  T2.2→T2.3  +  T3.2→T3.5  +  T4.2
Phase 3 (parallèle) : T1.6        +  T2.4→T2.6  +  T3.6       +  T4.3
Phase 4             : T2.7 + T3.7 + T1.7
Phase 5             : T2.8 + T3.8→T3.9  →  tag v0.5.0
```

## Comment lancer l'exécution

### Option A — Lancer les 3 axes en parallèle (recommandé)

Dans une nouvelle conversation Claude Code, envoie ce message :

```
@agent-voltagent-biz:project-manager
Exécute la Phase 1 du plan v0.5.0 (fichiers dans .claude/plans/v0.5.0/).
Lance en parallèle les 3 tâches fondatrices :
- T1.1 (agent: voltagent-infra:deployment-engineer) — scaffolding mdbook + deploy-docs.yml
- T2.1 (agent: voltagent-lang:rust-engineer) — architecture i18n, Language enum, trait Strings, src/config/mod.rs
- T3.1 (agent: voltagent-qa-sec:test-automator) — infra tests, helpers setup_test_gnupghome(), fixtures clefs GPG
Détails dans .claude/plans/v0.5.0/axe1-documentation.md, axe2-i18n.md, axe3-tests.md
```

### Option B — Lancer axe par axe

Pour lancer un axe seul :

```
@agent-voltagent-lang:rust-engineer
Exécute les tâches T2.1 puis T2.2 du plan v0.5.0.
Fichier de spec : .claude/plans/v0.5.0/axe2-i18n.md
```

### Option C — Reprendre après une pause

Si la Phase 1 est terminée, lancer la Phase 2 :

```
@agent-voltagent-biz:project-manager
Les tâches T1.1, T2.1, T3.1 sont terminées (Phase 1 du plan v0.5.0).
Coordonne la Phase 2 en parallèle :
- T1.2→T1.5 (technical-writer)
- T2.2→T2.3 (rust-engineer)
- T3.2→T3.5 (test-automator)
Spec : .claude/plans/v0.5.0/
```

## Ressources nécessaires

| Agent | Tâches | Charge estimée |
|-------|--------|----------------|
| `voltagent-lang:rust-engineer` | T2.1, T2.2, T2.3, T2.4, T2.6 | ~5 semaines |
| `voltagent-biz:technical-writer` | T1.2–T1.6, T3.8 | ~4 semaines |
| `voltagent-infra:deployment-engineer` | T1.1, T3.7 | ~1 semaine |
| `voltagent-qa-sec:test-automator` | T3.1–T3.6, T3.9 | ~5 semaines |
| `voltagent-core-dev:fullstack-developer` | T2.5, T2.8, T3.6 | ~2 semaines |
| `voltagent-core-dev:ui-designer` | T4.1 | ~0.5 semaine |

## Critères d'acceptation globaux

- [ ] `cargo clippy -D warnings` ✓
- [ ] `cargo test` ✓ (tous axes)
- [ ] `cargo fmt --check` ✓
- [ ] `pre-commit run --all-files` ✓
- [ ] Site mdbook accessible sur GitHub Pages
- [ ] UI localisée EN/FR, langue persistante dans `~/.config/pgpilot/config.yaml`
- [ ] Coverage ≥ 60 % sur `src/gpg/` et `src/app/`
- [ ] `share/icons/hicolor/scalable/apps/pgpilot.svg` valide et lisible à 16 px
- [ ] `share/applications/pgpilot.desktop` valide (`desktop-file-validate` ✓)
