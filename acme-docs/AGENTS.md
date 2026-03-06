# Agents Guide: Acme Docs

## Scope

`acme-docs` is the documentation authority for the Underlay reference implementation. Keep planning, architecture, and execution history here rather than in package-local docs.

## Hard Rules

- Put reference-app roadmap work in `roadmaps/g*/` with three-digit IDs.
- Put execution history and sweep closeouts in `logs/YYYY-MM/` using `DD-HHMMSS-slug.md` filenames.
- Do not leave compatibility shim docs behind after moves; update links in place.
- Keep `vision/` high-level and stable, `architecture/` concrete, and `processes/` operational.
- Prefer Underlay source docs for shared framework doctrine and `acme-docs` for reference-app-specific application.
- Keep new roadmap work in roadmap-ID and batch language. Treat inherited phase-era wording in imported roadmap files as historical unless a file is reopened for active work.

## Effigy-First Execution

Default flow inside `acme-docs/`:
1. Run `effigy tasks --repo .`
2. Run `effigy health --repo .`
3. Run `effigy validate --repo .`
4. Prefer `effigy <task> --repo .` for rollout checks instead of calling shell scripts directly

Repo notes:
- `health` is the stable baseline for day-to-day docs validation
- `validate` runs the full current rollout-check set
- direct script execution is fallback only when debugging a specific check

## Validation

- `effigy health --repo .`
- `effigy validate --repo .`
- Confirm docs do not reintroduce deprecated flat docs paths when editing historical references.

## Reference Docs

- `vision/001-acme-reference-implementation-vision.md`
- `architecture/000-overview.md`
- `processes/210-reference-implementation-notes.md`
- `roadmaps/README.md`
- `logs/README.md`
