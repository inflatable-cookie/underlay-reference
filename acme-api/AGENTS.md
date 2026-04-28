# Agents Guide: acme-api

## Scope

`acme-api` is the Rust backend reference implementation (API + jobs). Keep patterns canonical and reusable for future project bootstraps.

## Hard Rules

- Keep transport/API behavior aligned with Underlay conventions.
- Keep query and migration changes explicit and minimal.
- Do not add app-specific hacks that reduce template clarity.
- Prefer existing shared error/response utilities over custom wrappers.

## Effigy-First Execution

- Let Effigy choose host vs container for normal work. Do not touch host-side `node_modules`, `vendor`, `target`, `.pnpm-store`, or `.svelte-kit` expecting it to affect the live runtime.
- Use `effigy <task>`, `effigy prep`, or `effigy container shell` when you need to change runtime dependencies or inspect the live environment.

Default flow inside `acme-api/`:
1. Run `effigy tasks`
2. Run `effigy health`
3. Run `effigy validate`
4. Prefer `effigy <task>` for repo-owned work instead of raw Cargo commands where Effigy already covers the path

Repo notes:
- `health` and `validate` currently use `build` as the stable Rust baseline
- `db:*` stays owned here and resolves from the workspace root through child-catalog routing
- raw `cargo` commands are fallback for work Effigy does not yet model directly

## Validation

- `effigy health`
- `effigy validate`
- Optional deeper check when needed: `cargo test`

## Reference Docs

Use `../acme-docs/` as the reference-app docs authority. Do not create package-local roadmap or report docs.

- `../acme-docs/processes/210-reference-implementation-notes.md`
- `../underlay/docs/guides/040-rust-backend.md`
- `../underlay/docs/guides/050-database.md`
- `../underlay/docs/guides/055-background-jobs.md`
- `../underlay/docs/guides/070-api-handlers.md`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `../acme-docs/policy/internal-writing-style.md`
