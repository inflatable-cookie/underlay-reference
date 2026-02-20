# Agents Guide: acme-front

## Scope

`acme-front` is the SvelteKit public frontend reference app.

## Hard Rules

- Use `bun` for JS/TS tasks.
- Keep implementation aligned with shared frontend patterns.
- Favor composable shared client/UI utilities over page-specific duplication.
- Keep public-facing pages lightweight and consistent.

## Validation

```bash
cd acme-front && bun check
```

## Reference Docs

- `../docs/reference-implementation-notes.md`
- `../../../libraries/underlay/docs/guides/066-spa-deployment-and-static-auth.md`
- `../../../libraries/underlay/docs/guides/100-frontend-web.md`
