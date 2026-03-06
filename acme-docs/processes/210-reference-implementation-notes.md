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

- `acme-client/src/commands/` — command modules per domain
- `acme-client/src/types/` — transport and domain DTOs
- `acme-client/src/utils/http-client.ts` — request/response mechanics
- `acme-client/src/index.ts` — public exports

### Client boundary rules

- Keep client code transport-focused and reusable.
- Keep command behavior typed and predictable.
- Avoid app-specific UI concerns in the shared client package.
- For admin edit entities, provide `get*WithEtag` and `update*WithEtag(..., { ifMatch })` helpers while keeping legacy wrappers that return `.data`.

## acme-admin details

### Main areas

- `acme-admin/src/routes/(app)/` — protected admin routes
- `acme-admin/src/routes/(auth)/` — unauthenticated routes
- `acme-admin/src/lib/` — shared stores/components/helpers

### Page patterns

- Prefer Underlay components and patterns over one-off UI implementations.
- Keep page-level data loading close to route boundaries.
- For edit forms of canonical entities, track current ETag, send `If-Match` on save, and on `412` reload latest values and prompt users to reapply edits.

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
# Workspace baseline
effigy health --repo .
effigy validate --repo .

# Docs rollout checks
effigy validate --repo acme-docs

# Repo-local baselines
effigy validate --repo acme-api
effigy validate --repo acme-client
effigy validate --repo acme-admin
effigy validate --repo acme-front
effigy validate --repo acme-ui
```
