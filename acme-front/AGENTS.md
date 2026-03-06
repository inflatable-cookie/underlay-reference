# Agents Guide: acme-front

## Scope

`acme-front` is the SvelteKit public frontend reference app.

## Hard Rules

- Use `bun` for JS/TS tasks.
- Keep implementation aligned with shared frontend patterns.
- Favor composable shared client/UI utilities over page-specific duplication.
- Keep public-facing pages lightweight and consistent.

## Effigy-First Execution

Default flow inside `acme-front/`:
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
- `../underlay/docs/guides/066-spa-deployment-and-static-auth.md`
- `../underlay/docs/guides/100-frontend-web.md`
