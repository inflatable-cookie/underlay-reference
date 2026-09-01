# Agents Guide: acme-client

## Scope

`acme-client` is the typed API client boundary shared by admin/front apps.

## Hard Rules

- Keep this package transport-focused and app-agnostic.
- Do not add UI concerns here.
- Keep command and type exports stable and explicit.
- Use Underlay JSON naming conventions at the wire boundary.
- Use `bun` for all JS/TS operations.

## Effigy-First Execution

Root `AGENTS.md` owns the runtime stance; this section only adds what is local
to this scope.

Run `effigy tasks` from `packages/acme-client/` to see what this package owns,
and prefer `effigy <task>` over raw package commands where Effigy already covers
the path.

## Validation

- `effigy health`
- `effigy validate`

## Reference Docs

Use `../../docs/` as the reference-app docs authority. Do not create package-local roadmap or report docs.

- `../../docs/processes/210-reference-implementation-notes.md`
- `../../../underlay/docs/guides/071-json-naming.md`
- `../../../underlay/docs/guides/080-typescript-client.md`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `../../docs/policy/internal-writing-style.md`
