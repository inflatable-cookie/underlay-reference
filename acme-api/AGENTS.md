# Agents Guide: acme-api

## Scope

`acme-api` is the Rust backend reference implementation (API + jobs). Keep patterns canonical and reusable for future project bootstraps.

## Hard Rules

- Keep transport/API behavior aligned with Underlay conventions.
- Keep query and migration changes explicit and minimal.
- Do not add app-specific hacks that reduce template clarity.
- Prefer existing shared error/response utilities over custom wrappers.

## Effigy-First Execution

Default flow inside `acme-api/`:
1. Run `effigy tasks --repo .`
2. Run `effigy health --repo .`
3. Run `effigy validate --repo .`
4. Prefer `effigy <task> --repo .` for repo-owned work instead of raw Cargo commands where Effigy already covers the path

Repo notes:
- `health` and `validate` currently use `build` as the stable Rust baseline
- `db:*` stays owned here and resolves from the workspace root through child-catalog routing
- raw `cargo` commands are fallback for work Effigy does not yet model directly

## Validation

- `effigy health --repo .`
- `effigy validate --repo .`
- Optional deeper check when needed: `cargo test`

## Reference Docs

Use `../acme-docs/` as the reference-app docs authority. Do not create package-local roadmap or report docs.

- `../acme-docs/processes/210-reference-implementation-notes.md`
- `../underlay/docs/guides/040-rust-backend.md`
- `../underlay/docs/guides/050-database.md`
- `../underlay/docs/guides/055-background-jobs.md`
- `../underlay/docs/guides/070-api-handlers.md`
