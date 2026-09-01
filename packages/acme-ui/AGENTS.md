# Agents Guide: acme-ui

## Scope

`acme-ui` is the shared UI package for the Underlay reference workspace.

## Hard Rules

- Use `bun` for JS/TS tasks.
- Keep exports stable and reusable across admin and front apps.
- Prefer shared Underlay-compatible patterns over app-specific wrappers.
- Keep package changes focused on reusable UI or rendering concerns.

## Effigy-First Execution

Root `AGENTS.md` owns the runtime stance; this section only adds what is local
to this scope.

Run `effigy tasks` from `packages/acme-ui/` to see what this package owns, and
prefer `effigy <task>` over raw package commands where Effigy already covers the
path.

## Validation

- `effigy health`
- `effigy validate`

## Reference Docs

Use `../../docs/` as the reference-app docs authority. Do not create package-local roadmap or report docs.

- `../../docs/processes/210-reference-implementation-notes.md`
- `../../../underlay/docs/guides/090-ui-kit.md`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `../../docs/policy/internal-writing-style.md`
