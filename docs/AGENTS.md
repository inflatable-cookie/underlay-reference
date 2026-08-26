# Agents Guide: Acme Docs

## Scope

Root `docs/` is the documentation authority for the Underlay reference implementation, addressed through the `acme-docs` Effigy catalog alias. Keep planning, architecture, and execution history here rather than in package-local docs.

## Hard Rules

- Put reference-app roadmap work in `roadmaps/g*/` with three-digit IDs.
- Put execution history and sweep closeouts in `logs/YYYY-MM/` using `DD-HHMMSS-slug.md` filenames.
- Do not leave compatibility shim docs behind after moves; update links in place.
- Keep `vision/` high-level and stable, `architecture/` concrete, and `processes/` operational.
- Prefer Underlay source docs for shared framework doctrine and root `docs/` for reference-app-specific application.
- Keep new roadmap work in roadmap-ID and batch language. Treat inherited phase-era wording in imported roadmap files as historical unless a file is reopened for active work.

## Effigy-First Execution

- Let Effigy choose host vs container for normal work. Do not touch host-side `node_modules`, `vendor`, `target`, `.pnpm-store`, or `.svelte-kit` expecting it to affect the live runtime.
- Use `effigy <task>`, `effigy prep`, or `effigy container shell` when you need to change runtime dependencies or inspect the live environment.

Default flow from the workspace root:
1. Run `effigy tasks`
2. Prefer explicit docs-authority selectors such as `effigy acme-docs/health`, `effigy acme-docs/validate`, `effigy acme-docs/qa:docs`, and `effigy acme-docs/qa:northstar`
3. Run `effigy validate` when the broader workspace needs validation
4. Prefer explicit docs-authority tasks for rollout checks instead of calling shell scripts directly

Repo notes:
- `acme-docs/health` is the stable baseline for day-to-day docs validation
- `acme-docs/validate` runs the full current rollout-check set
- direct script execution is fallback only when debugging a specific check

## Validation

- `effigy acme-docs/health`
- `effigy acme-docs/validate`
- `effigy acme-docs/qa:docs`
- Confirm docs do not reintroduce deprecated flat docs paths when editing historical references.

## Reference Docs

- `vision/001-acme-reference-implementation-vision.md`
- `architecture/000-overview.md`
- `processes/210-reference-implementation-notes.md`
- `roadmaps/README.md`
- `logs/README.md`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `policy/internal-writing-style.md`
