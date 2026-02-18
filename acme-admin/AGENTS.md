# Agents Guide: acme-admin

## Scope

`acme-admin` is the SvelteKit admin reference app. Prefer shared Underlay components and standard admin patterns.

## Hard Rules

- Use `bun` for JS/TS tasks.
- Prefer Underlay components/patterns over custom one-off UI structures.
- Keep route behavior and data loading close to route boundaries.
- Keep admin behavior consistent with shared patterns documented in Underlay guides.

## Validation

```bash
cd acme-admin && bun check
```

## Reference Docs

- `/Users/betterthanclay/Dev/apps/underlay-reference/docs/reference-implementation-notes.md`
- `/Users/betterthanclay/Dev/libraries/underlay/docs/guides/090-ui-kit.md`
- `/Users/betterthanclay/Dev/libraries/underlay/docs/guides/098-shared-admin-patterns.md`
- `/Users/betterthanclay/Dev/libraries/underlay/docs/guides/110-admin.md`
