# Comprehensive Sweeps Remediation Closeout (001-018)

Date: 2026-02-13

Related roadmap:
- `acme-docs/roadmap/003-comprehensive-sweeps-remediation.md`

## Outcome

- Roadmap phases 0-5 are now completed with all checklist items marked done.
- Critical and high-severity findings from the sweep set were remediated.
- Medium findings were either remediated or explicitly accepted/documented as scoped exceptions.

## Completed in final pass

- Added admin jobs API integration coverage in `acme-api`:
  - route-level tests for list/get/cancel/retry/stats in `acme-api/crates/api/src/routes/admin/jobs.rs`.
- Added scheduled-task route coverage in `acme-api`:
  - list filtering, get-by-id, invalid-id rejection, toggle behavior in `acme-api/crates/api/src/routes/admin/scheduled_tasks.rs`.
- Added app-router auth boundary tests in `acme-api`:
  - missing bearer token, non-admin rejection, admin acceptance in `acme-api/crates/api/tests/api_tests.rs`.
- Added jobs handler integration coverage in `acme-jobs`:
  - reminder dedupe and retry-attempt policy in `acme-api/crates/jobs/tests/reminder_handler_tests.rs`.
- Added `acme-front` test harness and critical flow tests:
  - `vitest` scripts/config plus auth-store and route-load tests.
- Added `acme-client` command tests for admin batch task routes:
  - verifies `tasks/batch-delete` and `tasks/batch-update` paths.

## Verification status

Executed during remediation:
- `cargo test -p acme-api`
- `cargo test -p acme-api --test api_tests`
- `cargo test -p acme-jobs`
- `cargo fmt`

Not executed in this pass (per reference-app constraints):
- TypeScript install/build/test commands in `acme-front` and `acme-client` were not run in-repo.
- Added test harness and test files are ready for maintainer-run verification (`bun install`, then `bun test`).

## Report updates

- Updated status annotations in:
  - `acme-docs/reports/2026-02-13-002-underlay-reuse-sweep.md`
  - `acme-docs/reports/2026-02-13-003-frontend-consistency-sweep.md`
- Roadmap definition-of-done is now fully checked in:
  - `acme-docs/roadmap/003-comprehensive-sweeps-remediation.md`
