# Agents Guide: acme-client

## Scope

`acme-client` is the typed API client boundary shared by admin/front apps.

## Hard Rules

- Keep this package transport-focused and app-agnostic.
- Do not add UI concerns here.
- Keep command and type exports stable and explicit.
- Use Underlay JSON naming conventions at the wire boundary.
- Use `bun` for all JS/TS operations.

## Validation

```bash
cd acme-client && bun check
cd acme-client && bun run build
```

## Reference Docs

- `/Users/betterthanclay/Dev/apps/underlay-reference/docs/reference-implementation-notes.md`
- `/Users/betterthanclay/Dev/libraries/underlay/docs/guides/071-json-naming.md`
- `/Users/betterthanclay/Dev/libraries/underlay/docs/guides/080-typescript-client.md`
