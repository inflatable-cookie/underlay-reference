# Agents Guide: acme-api

## Scope

`acme-api` is the Rust backend reference implementation (API + jobs). Keep patterns canonical and reusable for future project bootstraps.

## Hard Rules

- Keep transport/API behavior aligned with Underlay conventions.
- Keep query and migration changes explicit and minimal.
- Do not add app-specific hacks that reduce template clarity.
- Prefer existing shared error/response utilities over custom wrappers.
- Register routes in the family builder they belong to (`routes/runtime.rs`,
  `routes/shared/router.rs`, `routes/front/router.rs`, `routes/admin/router.rs`).
  Never register a route in `main.rs`.
- Read env in bootstrap only. Handlers and services take typed config or
  extractors. Every env key must appear in root `config/env-manifest.txt`.
- Client IP used for auth, lockout, rate limiting, or audit comes from the
  peer-aware `RequestContext`. Never parse forwarding headers in a handler.
- There is no `.env` file. Non-secret values live in the root `config/` stack.

## Effigy-First Execution

Root `AGENTS.md` owns the runtime stance; this section only adds what is local
to this scope.

Run `effigy tasks` from `apps/acme-api/` to see what this crate workspace owns,
and prefer `effigy <task>` over raw Cargo where Effigy already covers the path.

Repo notes:
- `health` uses `fmt` plus a cheap `cargo check` baseline; `validate` still runs `build`
- the built-in `rust` test suite intentionally pins `cargo test --workspace`;
  the checked-in Nextest profiles remain available for explicit direct use
- `migration:*` stays owned here and resolves from the workspace root through child-catalog routing
- raw `cargo` commands are fallback for work Effigy does not yet model directly

## Validation

- `effigy health`
- `effigy validate`
- Optional deeper check when needed: `cargo test`

## Reference Docs

Use `../../docs/` as the reference-app docs authority. Do not create package-local roadmap or report docs.

- `../../docs/processes/210-reference-implementation-notes.md`
- `../../../underlay/docs/guides/040-rust-backend.md`
- `../../../underlay/docs/guides/050-database.md`
- `../../../underlay/docs/guides/055-background-jobs.md`
- `../../../underlay/docs/guides/070-api-handlers.md`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `../../docs/policy/internal-writing-style.md`

<!-- northstar:rust-quality:start -->
## Northstar Rust Quality

Scope: Rust source, Cargo manifests, build files, tests, and directly related
documentation under this directory.

Use Northstar's strict everyday-authoring route for ordinary Rust work. Resolve
the repository-owned profile and deviations under `docs/contracts/`; never
assume a universal MSRV. Re-enter at task start and coherent batch closeout.
Preserve unrelated work. A quality audit, no-slop pass, or audit-and-fix request
is explicit audit intent; never route it through everyday authoring.
<!-- northstar:rust-quality:end -->
