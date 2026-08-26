# Agents Guide: acme-front

## Scope

`acme-front` is the SvelteKit public frontend reference app.

## Hard Rules

- Use `bun` for JS/TS tasks.
- Keep implementation aligned with shared frontend patterns.
- Favor composable shared client/UI utilities over page-specific duplication.
- Keep public-facing pages lightweight and consistent.

## Effigy-First Execution

- Let Effigy choose host vs container for normal work. Do not touch host-side `node_modules`, `vendor`, `target`, `.pnpm-store`, or `.svelte-kit` expecting it to affect the live runtime.
- Use `effigy <task>`, `effigy prep`, or `effigy container shell` when you need to change runtime dependencies or inspect the live environment.

Default flow inside `apps/acme-front/`:
1. Run `effigy tasks`
2. Run `effigy health`
3. Run `effigy validate`
4. Prefer `effigy <task>` for repo-owned work instead of raw package commands where Effigy already covers the path

## Validation

- `effigy health`
- `effigy validate`

## Reference Docs

Use `../../docs/` as the reference-app docs authority. Do not create package-local roadmap or report docs.

- `../../docs/processes/210-reference-implementation-notes.md`
- `../underlay/docs/guides/066-spa-deployment-and-static-auth.md`
- `../underlay/docs/guides/100-frontend-web.md`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `../../docs/policy/internal-writing-style.md`
