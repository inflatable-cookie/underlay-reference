# Underlay Reference Implementation Notes

This document keeps detailed reference information that was previously duplicated across `AGENTS.md` files.

Use this as a lookup when implementing or copying patterns from the Acme reference apps.

## Package map

- `acme-api/` — Rust backend (API + jobs)
- `acme-client/` — typed TypeScript API client
- `acme-admin/` — SvelteKit admin app
- `acme-front/` — SvelteKit public app
- `acme-ui/` — shared UI package

## acme-api details

### Crate organization

- `crates/core` — primitives and shared types
- `crates/infra` — configuration, logging, email setup
- `crates/db` — database access and migrations integration
- `crates/auth` — authentication logic
- `crates/domain` — business/domain logic
- `crates/jobs` — background job handlers
- `crates/api` — HTTP routes and server wiring

### Key files

- `acme-api/crates/api/src/main.rs`
- `acme-api/crates/jobs/src/main.rs`
- `acme-api/crates/infra/src/config.rs`
- `acme-api/migrations/`

## acme-client details

### Main areas

- `acme-client/src/commands/` — command modules per domain
- `acme-client/src/types/` — transport and domain DTOs
- `acme-client/src/utils/http-client.ts` — request/response mechanics
- `acme-client/src/index.ts` — public exports

### Client boundary rules

- Keep client code transport-focused and reusable.
- Keep command behavior typed and predictable.
- Avoid app-specific UI concerns in the shared client package.

## acme-admin details

### Main areas

- `acme-admin/src/routes/(app)/` — protected admin routes
- `acme-admin/src/routes/(auth)/` — unauthenticated routes
- `acme-admin/src/lib/` — shared stores/components/helpers

### Page patterns

- Prefer Underlay components and patterns over one-off UI implementations.
- Keep page-level data loading close to route boundaries.

## acme-front details

### Main areas

- `acme-front/src/routes/` — public-facing pages
- `acme-front/src/lib/` — shared site code
- `acme-front/src/hooks.server.ts` and `hooks.client.ts` — runtime setup

### Frontend patterns

- Keep public routes lightweight and SEO-aware.
- Use protected route groups only where needed.

## Common implementation tasks

### Add API endpoint

1. Add DTOs in `acme-api/crates/api/src/dto/`.
2. Add route handler in `acme-api/crates/api/src/routes/`.
3. Register route module in route wiring.
4. Add client command in `acme-client/src/commands/`.
5. Export command and related types from `acme-client/src/index.ts`.

### Add database table

1. Add migration in `acme-api/migrations/`.
2. Add query functions in `acme-api/crates/db/src/`.
3. Re-export query module from `acme-api/crates/db/src/lib.rs`.

### Add admin page

1. Add route in `acme-admin/src/routes/(app)/`.
2. Add navigation entry in `acme-admin/src/lib/ui/AdminNavList.svelte`.
3. Prefer shared Underlay page/layout components.

## Validation quick commands

```bash
# API
cd acme-api && cargo build

# Client
cd acme-client && bun check

# Admin
cd acme-admin && bun check

# Front
cd acme-front && bun check
```
