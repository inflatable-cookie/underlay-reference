# Comprehensive Sweeps Remediation Roadmap

This roadmap consolidates findings from running Underlay sweeps **001-018** against this reference repo (`acme-api`, `acme-client`, `acme-admin`, `acme-front`).

Run date: 2026-02-13

Related reports:
- `acme-docs/logs/2026-02/13-000200-underlay-reuse-sweep.md`
- `acme-docs/logs/2026-02/13-000300-frontend-consistency-sweep.md`

## Coverage Summary

Sweep status snapshot:

- 001 Security: partial
- 002 Underlay Reuse: partial
- 003 Frontend Consistency: partial
- 004 Tab Count Badges: fail
- 005 API/Client Contract Drift: fail
- 006 Query Efficiency: partial
- 007 Error Diagnostics/Logging: fail
- 008 Form/Nightfire Validation: fail
- 009 Rich Text Storage Alignment: partial
- 010 Authorization Boundary: fail
- 011 Migration Safety: partial
- 012 Observability/Audit: partial
- 013 Jobs/Scheduler Reliability: fail
- 014 Accessibility/Keyboard: fail
- 015 Test Coverage/Critical Paths: fail
- 016 API Versioning/Backward Compat: fail
- 017 Dependency/Supply Chain Hygiene: partial
- 018 Privacy/Sensitive Data Handling: fail

Severity totals across all sweeps:

- Critical: 1
- High: 13
- Medium: 20
- Low: 8
- Notes: 4

---

## Batch 0 - Immediate Blocking Fixes

Priority: **Critical**

### 0.1 Close task ownership mutation bypass

- [x] Constrain user task update/delete queries by both `task_id` and `project_id` (and owner scope)
- [x] Add explicit task-to-project ownership check before mutations
- [x] Add regression tests for cross-project/cross-user mutation attempts

Primary files:
- `acme-api/crates/api/src/routes/tasks.rs`
- `acme-api/crates/db/src/tasks/crud.rs`

Why: prevents unauthorized task mutation if foreign UUID is known.

---

## Batch 1 - Security, Privacy, and Contract Integrity

Priority: **High**

### 1.1 Enforce API version policy on server

- [x] Add middleware to validate/interpret `X-Api-Version`
- [x] Reject unsupported versions with explicit error codes
- [x] Document supported/deprecated version behavior

Files:
- `acme-api/crates/api/src/routes/mod.rs`
- `acme-api/crates/api/src/main.rs`

### 1.2 Remove refresh token from default browser response payloads

- [x] Keep refresh token cookie-based for web flows
- [x] Restrict body refresh token mode to explicit non-browser clients
- [x] Update client contracts accordingly

Files:
- `acme-api/crates/api/src/dto/auth.rs`
- `acme-api/crates/api/src/routes/shared/auth/mod.rs`
- `acme-client/src/utils/auth-manager.ts`

### 1.3 Require encryption key in non-dev environments

- [x] Fail startup when `ENCRYPTION_KEY` is missing outside local/dev
- [x] Keep dev fallback behavior only for local reference use
- [x] Add deployment check docs

Files:
- `acme-api/crates/auth/src/local/mod.rs`
- `acme-api/.env.example`

### 1.4 Remove/complete drifted auth endpoints

- [x] Either implement login email fallback/resend routes or remove command/UI usage
- [x] Either implement OAuth routes or remove/stub exports with clear deprecation path

Files:
- `acme-client/src/commands/auth/core-commands.ts`
- `acme-client/src/commands/auth/oauth-commands.ts`
- `acme-api/crates/api/src/routes/mod.rs`
- `acme-admin/src/routes/(auth)/login/+page.svelte`

### 1.5 Redact PII in password reset logs

- [x] Remove plaintext email from reset-request log line
- [x] Log hashed/redacted identifier instead

Files:
- `acme-api/crates/api/src/routes/shared/auth/password_reset.rs`

---

## Batch 2 - API/Client and Data Access Efficiency

Priority: **High/Medium**

### 2.1 Fix admin task batch endpoint path drift

- [x] Align client command paths with API routes (`tasks/batch-*`)
- [x] Add command integration tests covering batch delete/update

Files:
- `acme-client/src/commands/admin/task-commands.ts`
- `acme-api/crates/api/src/routes/mod.rs`

### 2.2 Move tab badges to detail DTO counts; lazy-load heavy tabs

- [x] Use detail count fields for tab badges instead of eager list length
- [x] Fetch sessions/activity/usages/versions only on tab activation

Files:
- `acme-admin/src/routes/(app)/users/[userId]/+page.svelte`
- `acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte`
- `acme-api/crates/db/src/users.rs`
- `acme-api/crates/api/src/dto/media.rs`

### 2.3 Remove unsafe DB string matching and raw `to_string()` error responses

- [x] Replace string contains logic with typed DB diagnostics
- [x] Map validation errors to structured field errors consistently

Files:
- `acme-api/crates/api/src/routes/admin/media/crud.rs`
- `acme-api/crates/api/src/routes/admin/users.rs`
- `acme-api/crates/api/src/routes/tasks.rs`

---

## Batch 3 - Observability, Audit, and Migration Safety

Priority: **High/Medium**

### 3.1 Add durable audit logging for user-facing mutations

- [x] Emit activity/audit events for project/task create/update/delete in user routes
- [x] Include actor/resource/action consistently

Files:
- `acme-api/crates/api/src/routes/tasks.rs`
- `acme-api/crates/db/src/activity.rs`

### 3.2 Persist request correlation IDs into audit entries

- [x] Thread request ID from middleware into route context
- [x] Populate `correlation_id` in activity log writes

Files:
- `acme-api/crates/api/src/main.rs`
- `acme-api/crates/api/src/routes/admin/*.rs`
- `acme-api/crates/db/src/activity.rs`

### 3.3 Make non-idempotent migration step safe

- [x] Guard `media_current_version_fk` constraint creation with existence check
- [x] Validate rollback/re-apply behavior in dev reset flow

Files:
- `acme-api/migrations/202601301205__baseline_media.sql`

---

## Batch 4 - Jobs Reliability and Test Coverage

Priority: **High/Medium**

### 4.1 Enforce idempotency and failure semantics in jobs

- [x] Make batch handlers fail job when item failures occur (or mark degraded explicitly)
- [x] Add dedupe/idempotency key for due reminder fan-out
- [x] Preserve full execution policy on retry (timeout/priority/overlap metadata)
- [x] Add integration tests for reminder dedupe and retry-attempt policy
- [x] Add unit tests for retry config policy mapping

Files:
- `acme-api/crates/jobs/src/handlers/tasks.rs`
- `acme-api/crates/jobs/src/handlers/projects.rs`
- `acme-api/crates/api/src/routes/admin/jobs.rs`

### 4.2 Raise automated test coverage on critical paths

- [x] Add real integration tests for admin jobs APIs
- [x] Add real integration tests for scheduled-task APIs
- [x] Add app-router tests for critical auth/admin route boundaries
- [x] Add frontend test harness and critical flow tests for `acme-front`

Files:
- `acme-api/crates/jobs/`
- `acme-api/crates/api/tests/`
- `acme-admin/tests/`
- `acme-front/` (new test setup)

---

## Batch 5 - Frontend Consistency and Accessibility Hardening

Priority: **Medium/Low**

### 5.1 Centralize client configuration setup

- [x] Remove duplicate `configureAcmeClient(...)` calls from stores
- [x] Keep one canonical init path per runtime boundary

Files:
- `acme-admin/src/hooks.server.ts`
- `acme-admin/src/hooks.client.ts`
- `acme-admin/src/lib/stores/auth.ts`
- `acme-front/src/hooks.server.ts`
- `acme-front/src/hooks.client.ts`
- `acme-front/src/lib/stores/auth.ts`

### 5.2 Resolve remaining a11y keyboard/label issues

- [x] Add explicit accessible names for icon-only controls
- [x] Ensure action reveal behavior supports keyboard focus, not just hover
- [x] Remove fake-button wrappers where native controls can be used

Files:
- `acme-front/src/routes/(app)/projects/[projectId]/+page.svelte`
- `acme-admin/src/routes/(app)/system/jobs/+page.svelte`
- `acme-admin/src/routes/(app)/+layout.svelte`

### 5.3 Reduce styling drift from hardcoded colors

- [x] Migrate repeated semantic hex values to shared tokenized helpers
- [x] Start with `accents.ts` and system/media pages

Files:
- `acme-admin/src/lib/utils/accents.ts`
- `acme-admin/src/routes/(app)/system/+page.svelte`
- `acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte`

---

## Definition of Done

- [x] Critical + high findings resolved
- [x] Medium findings either resolved or accepted with documented exceptions
- [x] Regression tests added for all fixed high-impact flows
- [x] Sweep reports updated with resolution status

## Execution Order Recommendation

1. Phase 0 (critical authz boundary)
2. Phase 1 (security/privacy/versioning/contract)
3. Phase 2 (contract + query efficiency)
4. Batch 3 (audit + migration safety)
5. Batch 4 (jobs + test reliability)
6. Batch 5 (frontend consistency/a11y/style hygiene)
