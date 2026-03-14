# Agents Guide: acme-admin

## Scope

`acme-admin` is the SvelteKit admin reference app. Prefer shared Underlay components and standard admin patterns.

## Hard Rules

- Use `bun` for JS/TS tasks.
- Prefer Underlay components/patterns over custom one-off UI structures.
- Keep route behavior and data loading close to route boundaries.
- Keep admin behavior consistent with shared patterns documented in Underlay guides.

## Effigy-First Execution

Default flow inside `acme-admin/`:
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
- `../underlay/docs/guides/098-shared-admin-patterns.md`
- `../underlay/docs/guides/110-admin.md`
