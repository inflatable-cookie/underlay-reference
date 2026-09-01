# Agents Guide: acme-front

## Scope

`acme-front` is the SvelteKit public frontend reference app.

## Hard Rules

- Use `bun` for JS/TS tasks.
- Keep implementation aligned with shared frontend patterns.
- Favor composable shared client/UI utilities over page-specific duplication.
- Keep public-facing pages lightweight and consistent.

## Effigy-First Execution

Root `AGENTS.md` owns the runtime stance; this section only adds what is local
to this scope.

Run `effigy tasks` from `apps/acme-front/` to see what this app owns, and prefer
`effigy <task>` over raw package commands where Effigy already covers the path.

## Validation

- `effigy health`
- `effigy validate`

## Reference Docs

Use `../../docs/` as the reference-app docs authority. Do not create package-local roadmap or report docs.

- `../../docs/processes/210-reference-implementation-notes.md`
- `../../../underlay/docs/guides/066-spa-deployment-and-static-auth.md`
- `../../../underlay/docs/guides/100-frontend-web.md`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `../../docs/policy/internal-writing-style.md`
