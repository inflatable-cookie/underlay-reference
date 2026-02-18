# Agents Guide: acme-api

## Scope

`acme-api` is the Rust backend reference implementation (API + jobs). Keep patterns canonical and reusable for future project bootstraps.

## Hard Rules

- Keep transport/API behavior aligned with Underlay conventions.
- Keep query and migration changes explicit and minimal.
- Do not add app-specific hacks that reduce template clarity.
- Prefer existing shared error/response utilities over custom wrappers.

## Validation

```bash
cd acme-api && cargo build
# Optional deeper check when needed:
cd acme-api && cargo test
```

## Reference Docs

- `/Users/betterthanclay/Dev/apps/underlay-reference/docs/reference-implementation-notes.md`
- `/Users/betterthanclay/Dev/libraries/underlay/docs/guides/040-rust-backend.md`
- `/Users/betterthanclay/Dev/libraries/underlay/docs/guides/050-database.md`
- `/Users/betterthanclay/Dev/libraries/underlay/docs/guides/055-background-jobs.md`
- `/Users/betterthanclay/Dev/libraries/underlay/docs/guides/070-api-handlers.md`
