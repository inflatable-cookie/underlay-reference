# Agents Guide: acme-ui

## Scope

`acme-ui` is the shared UI package for the Underlay reference workspace.

## Hard Rules

- Use `bun` for JS/TS tasks.
- Keep exports stable and reusable across admin and front apps.
- Prefer shared Underlay-compatible patterns over app-specific wrappers.
- Keep package changes focused on reusable UI or rendering concerns.

## Effigy-First Execution

Default flow inside `acme-ui/`:
1. Run `effigy tasks --repo .`
2. Run `effigy health --repo .`
3. Run `effigy validate --repo .`
4. Prefer `effigy <task> --repo .` for repo-owned work instead of raw package commands where Effigy already covers the path

## Validation

- `effigy health --repo .`
- `effigy validate --repo .`

## Reference Docs

Use `../acme-docs/` as the reference-app docs authority. Do not create package-local roadmap or report docs.

- `../acme-docs/processes/210-reference-implementation-notes.md`
- `../underlay/docs/guides/090-ui-kit.md`
