# Plans — pgpilot

This directory holds every planning document, following the convention shared with
the sibling projects `stc` and `astropath`.

## Layout

- `vX.Y.Z/` — the plan that produced release `vX.Y.Z`. Each directory holds at
  least an entry document:
  - `plan.md` — context, scope, atomic steps, decisions (single-chantier plans).
  - `README.md` — index of a multi-axis plan; the axes then live in `axeN-*.md`
    files alongside it (used by the larger v0.5.0–v0.8.0 plans).
  - Optional companions: `manual_tests.md` (manual test sheet, run at final
    validation) and `phase0_results.md` (baseline audit, cf. `PROCEDURE_PLANS.md` §2).
- `release/` — plans about tooling/infrastructure (tags, changelog, CI, working
  methods) rather than a single product feature.

Rules:

- Plans always live here, **never at the repository root**.
- An obsolete plan is **deleted**, never duplicated as `_v2` / `_v3`.
- A plan is authored before any code (see [`PROCEDURE_PLANS.md`](../../PROCEDURE_PLANS.md)).
- `.claude/settings.local.json` and `.claude/worktrees/` stay **local** (gitignored).
  Everything under `.claude/plans/` is committed.

## Version map

pgpilot shipped its first four releases (the iced UI era) before plans were kept.
From `v0.5.0` on, every release has a plan directory. Patch releases are folded
under their minor's plan.

| Tag | Commit | Date | Content | Plan |
|-----|--------|------|---------|------|
| `v0.1.0`–`v0.4.0` | `b670537`…`d7a79de` | 2026-05-03 | iced UI era: master-detail layout, iced 0.13→0.14 migration, key-list polish | — |
| `v0.5.0` | `abd10f7` | 2026-05-04 | Documentation, i18n, tests, desktop assets (4 axes) | [`v0.5.0/`](v0.5.0/README.md) |
| `v0.5.1` | `5936f5f` | 2026-05-04 | Patch on v0.5.0 | [`v0.5.0/`](v0.5.0/README.md) |
| `v0.6.0` | `8250e78` | 2026-05-06 | i18n EN/FR + encrypted PGP/MQTT ephemeral chat | [`v0.6.0/`](v0.6.0/README.md) |
| `v0.7.0` | `ac54e59` | 2026-05-08 | USSR theme v2 | [`v0.7.0/`](v0.7.0/README.md) |
| `v0.8.0` | `b135cb6` | 2026-05-12 | Migration iced → Tauri v2 (React + TS frontend) | [`v0.8.0/`](v0.8.0/README.md) |
| `v0.8.1`–`v0.8.8` | `b24ca71`…`5228f68` | 2026-05-12…15 | Tauri-series fixes (dialogs, packaging, CI) | [`v0.8.0/`](v0.8.0/README.md) |
| `v0.8.9` | `7514a1f` | 2026-05-23 | Native file pickers + CI fixes | [`v0.8.9/`](v0.8.9/plan.md) |
| `v0.9.0` | `28f31e7` | 2026-05-23 | Quality & security hardening | [`v0.9.0/`](v0.9.0/plan.md) |
| `v0.9.1` | `b105370` | 2026-05-31 | AppImage EGL crash fix | [`v0.9.1/`](v0.9.1/plan.md) |
| `v0.10.0` | _pending_ | 2026-07-17 | Drop the non-functional AppImage + consolidate 10 Renovate dependency PRs | [`v0.10.0/`](v0.10.0/plan.md) |
