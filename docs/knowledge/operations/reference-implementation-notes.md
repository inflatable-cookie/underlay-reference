# Underlay Reference Implementation Notes

This document keeps detailed reference information that was previously duplicated across `AGENTS.md` files.

Use this as a lookup when implementing or copying patterns from the Acme reference apps.

## Package map

- `apps/acme-api/` — Rust backend (API + jobs)
- `packages/acme-client/` — typed TypeScript API client
- `apps/acme-admin/` — SvelteKit admin app
- `apps/acme-front/` — SvelteKit public app
- `packages/acme-ui/` — shared UI package

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

- `apps/acme-api/crates/api/src/main.rs`
- `apps/acme-api/crates/jobs/src/main.rs`
- `apps/acme-api/crates/infra/src/config.rs`
- `apps/acme-api/migrations/`

### HTTP freshness contract (admin detail/edit flows)

- Detail GET endpoints should emit:
  - `ETag`
  - `Cache-Control: private, no-cache, must-revalidate`
- Detail GET endpoints should support `If-None-Match` and return `304` when unchanged.
- Conflict-prone update endpoints should accept `If-Match` and return `412` with code `resource.precondition_failed` on mismatch.
- Successful updates should return fresh payload + fresh `ETag` + admin cache-control header.

### Reorder conflict recovery (canonical-order lists only)

- Canonical manual-order entities support reorder:
  - projects
  - categories
  - tasks within a project
- Non-canonical list views are intentionally non-reorderable:
  - labels (name-scoped list)
  - date-sorted/computed feeds (activity, jobs, logs, dashboard aggregates)
- Reorder endpoints should return `409` with stable conflict message and context keys:
  - `added_ids: string[]`
  - `removed_ids: string[]`
- Admin reorder UIs should wire `ReorderableList` `onsubmiterror` to app-local recovery:
  - parse conflict payload (`extractReorderConflict`)
  - apply pending-state merge/remove (`applyReorderConflict`)
  - keep reorder mode active; user reviews and explicitly saves again.

### Auth security alerting contract (failed-login + lockout pressure)

- Use shared Underlay primitives from `underlay-security-alerts`:
  - `load_ip_signal_counts(...)`
  - `evaluate_alerts(...)`
  - `has_recent_alert(...)`
  - `insert_alert_event(...)`
- Persist alert events in `auth.security_alert_events` with indexes on `created_at` and `(alert_type, ip_address, created_at)`.
- Config defaults (from `config/default.toml`):
  - `security_alert_window_secs = 600`
  - `security_alert_cooldown_secs = 1800`
  - `security_alert_failed_attempts_threshold = 20`
  - `security_alert_distinct_users_threshold = 5`
  - `security_alert_lockouts_threshold = 3`
- Failed-login and lockout-denied attempts should both contribute to `auth.login_attempts` signals.
- For each emitted alert:
  - write structured `warn!` log
  - append audit log event (`action = "auth.security_alert_emitted"`, `resource_type = "security_alert_event"`).

## acme-client details

### Main areas

- `packages/acme-client/src/commands/` — command modules per domain
- `packages/acme-client/src/types/` — transport and domain DTOs
- `packages/acme-client/src/utils/http-client.ts` — request/response mechanics
- `packages/acme-client/src/index.ts` — public exports

### Client boundary rules

- Keep client code transport-focused and reusable.
- Keep command behavior typed and predictable.
- Avoid app-specific UI concerns in the shared client package.
- For admin edit entities, provide `get*WithEtag` and `update*WithEtag(..., { ifMatch })` helpers while keeping legacy wrappers that return `.data`.

## acme-ui details

### Nightfire block-module reference pattern

Use the task notes / project description block family as the canonical reference:

- TS block-family source:
  - `packages/acme-ui/src/nightfire/notes/registrations.ts`
- TS thin registration entrypoints:
  - `packages/acme-ui/src/nightfire/notes/editor.ts`
  - `packages/acme-ui/src/nightfire/notes/render.ts`
  - `packages/acme-ui/src/nightfire/notes/validation.ts`
  - `packages/acme-ui/src/nightfire/project-description/editor.ts`
  - `packages/acme-ui/src/nightfire/project-description/render.ts`
  - `packages/acme-ui/src/nightfire/project-description/validation.ts`

What this pattern demonstrates:

- one block-family source owns schema ids, block labels, validators, editors, and renderers
- package entrypoints stay stable:
  - `@acme/ui/editor`
  - `@acme/ui/render`
  - `@acme/ui/validation`
- multiple Nightfire fields can assemble from the same block-family source with different schema ids and labels

For new block families, copy this shape instead of creating separate editor/render/validation lists by hand.

## acme-admin details

### Main areas

- `apps/acme-admin/src/routes/(app)/` — protected admin routes
- `apps/acme-admin/src/routes/(auth)/` — unauthenticated routes
- `apps/acme-admin/src/lib/` — shared stores/components/helpers

### Page patterns

- Prefer Underlay components and patterns over one-off UI implementations.
- Keep page-level data loading close to route boundaries.
- For edit forms of canonical entities, track current ETag, send `If-Match` on save, and on `412` reload latest values and prompt users to reapply edits.

## acme-front details

### Main areas

- `apps/acme-front/src/routes/` — public-facing pages
- `apps/acme-front/src/lib/` — shared site code
- `apps/acme-front/src/hooks.server.ts` and `hooks.client.ts` — runtime setup

### Frontend patterns

- Keep public routes lightweight and SEO-aware.
- Use protected route groups only where needed.

## Structured content reference pattern

### Nightfire + media usage integration

Use these files as the canonical Rust-side reference:

- app-level Nightfire glue:
  - `apps/acme-api/crates/api/src/nightfire/mod.rs`
- block/media module set:
  - `apps/acme-api/crates/api/src/nightfire/notes.rs`
- structured field route helpers:
  - `apps/acme-api/crates/api/src/routes/project_description.rs`
  - `apps/acme-api/crates/api/src/routes/admin/tasks.rs`

What this pattern demonstrates:

- prepare once:
  - ensure block ids
  - serialize exact Nightfire JSON
- persist the JSON field unchanged
- sync locator-aware media usage through the registry-backed extractor
- keep block/media registrations together in the app-level Nightfire module set

Reference fields already using this pattern:

- `project.description`
- `task.notes`

For future structured fields, extend the Nightfire module set first. Do not re-implement `ensure ids -> serialize -> sync/clear` inside each route.

## Common implementation tasks

### Add API endpoint

1. Add DTOs in `apps/acme-api/crates/api/src/dto/`.
2. Add route handler in `apps/acme-api/crates/api/src/routes/`.
3. Register route module in route wiring.
4. Add client command in `packages/acme-client/src/commands/`.
5. Export command and related types from `packages/acme-client/src/index.ts`.

### Add database table

1. Add migration in `apps/acme-api/migrations/`.
2. Add query functions in `apps/acme-api/crates/db/src/`.
3. Re-export query module from `apps/acme-api/crates/db/src/lib.rs`.

### Add admin page

1. Add route in `apps/acme-admin/src/routes/(app)/`.
2. Add navigation entry in `apps/acme-admin/src/lib/ui/AdminNavList.svelte`.
3. Prefer shared Underlay page/layout components.

## Validation quick commands

```bash
# Workspace baseline
effigy health
effigy validate

# One frozen root install for the JavaScript workspace
effigy workspace:js:prepare

# Docs rollout checks
effigy acme-docs/validate

# Package baselines (catalog-qualified selectors from the workspace root)
effigy acme-api/validate
effigy acme-client/validate
effigy acme-admin/validate
effigy acme-front/validate
effigy acme-ui/validate

# Workspace shape conformance
effigy qa:workspace-shape
```
