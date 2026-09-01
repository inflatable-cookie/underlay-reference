# Agents Guide: acme-admin

## Scope

`acme-admin` is the SvelteKit admin reference app. Prefer shared Underlay components and standard admin patterns.

## Hard Rules

- Use `bun` for JS/TS tasks.
- Prefer Underlay components/patterns over custom one-off UI structures.
- Keep route behavior and data loading close to route boundaries.
- Keep admin behavior consistent with shared patterns documented in Underlay guides.

## Effigy-First Execution

Root `AGENTS.md` owns the runtime stance; this section only adds what is local
to this scope.

Run `effigy tasks` from `apps/acme-admin/` to see what this app owns, and prefer
`effigy <task>` over raw package commands where Effigy already covers the path.

## Validation

- `effigy health`
- `effigy validate`

## Reference Docs

Use `../../docs/` as the reference-app docs authority. Do not create package-local roadmap or report docs.

- `../../docs/processes/210-reference-implementation-notes.md`
- `../../../underlay/docs/guides/090-ui-kit.md`
- `../../../underlay/docs/guides/098-shared-admin-patterns.md`
- `../../../underlay/docs/guides/110-admin.md`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `../../docs/policy/internal-writing-style.md`
