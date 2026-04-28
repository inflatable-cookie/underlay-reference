# Agents Guide: acme-ui

## Scope

`acme-ui` is the shared UI package for the Underlay reference workspace.

## Hard Rules

- Use `bun` for JS/TS tasks.
- Keep exports stable and reusable across admin and front apps.
- Prefer shared Underlay-compatible patterns over app-specific wrappers.
- Keep package changes focused on reusable UI or rendering concerns.

## Effigy-First Execution

- Let Effigy choose host vs container for normal work. Do not touch host-side `node_modules`, `vendor`, `target`, `.pnpm-store`, or `.svelte-kit` expecting it to affect the live runtime.
- Use `effigy <task>`, `effigy prep`, or `effigy container shell` when you need to change runtime dependencies or inspect the live environment.

Default flow inside `acme-ui/`:
1. Run `effigy tasks`
2. Run `effigy health`
3. Run `effigy validate`
4. Prefer `effigy <task>` for repo-owned work instead of raw package commands where Effigy already covers the path

## Validation

- `effigy health`
- `effigy validate`

## Reference Docs

Use `../acme-docs/` as the reference-app docs authority. Do not create package-local roadmap or report docs.

- `../acme-docs/processes/210-reference-implementation-notes.md`
- `../underlay/docs/guides/090-ui-kit.md`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `../acme-docs/policy/internal-writing-style.md`
