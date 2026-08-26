# g09.038 Underlay Reference Migration And Test Proof

Date: 2026-08-26
Roadmap: Underlay `g09.038`
Handoff: `docs/handoffs/20260826-181729-g09-038-migration-test-proof.md`

## Outcome

Cut Underlay Reference over to the baseline root-state and package-owned
`migration:*` interface, proved from-empty apply and reset/replay against the
approved local database, restored the API health check baseline, and converted
the state-free health route tests to `underlay_testing::TestServer`.

Target `g01.007` planning state was not changed.

## Worker Checkout

- worktree: `/Users/tom/.t3/worktrees/underlay-reference/t3code-ca4ef458`
- branch: `worker/g09-038-underlay-reference-migration-test-proof`
- planning base ancestor: `2fc07785`
- T3 launched this thread in the Underlay worktree, so the worker created this
  registered Underlay Reference worktree under the existing T3 container

## Local Target

Re-proved immediately before mutation:

- Effigy system / compose project: `underlay-reference-dev`
- PostgreSQL container: `underlay-reference-dev-postgres-1`
- database: `acme`
- user: `postgres`
- host binding: `127.0.0.1:19932`
- volume: `underlay-reference-dev-postgres-data`
- readiness: accepting connections
- runtime: Colima profile `effigy` / containerd
- this stack does not declare shared backing services

No database contents or credentials are recorded here.

## Proof

`effigy state plan` names `reset` (structure), `structure` (structure), and
`dev-overlay` (dev-overlay, `dev-only`) in application order.

`effigy state apply local --yes` executed all three layers. Package
`acme-api/migration:reset` then replayed the same drop + schema + overlay
sequence. Schema list, table list, and `_sqlx_migrations` versions matched:

- 6 schemas (`account`, `acme`, `auth`, `media`, `platform`, `public`)
- 31 tables
- 8 structural migrations

Overlay remains a separate package task. Seed failure exits non-zero, so both
state apply and reset stop instead of continuing.

## Changed Surfaces

- root `[state.local]` stack behind `effigy state plan` / `effigy state apply local --yes`
- `apps/acme-api` `migration:reset:schema`, `migration:apply`,
  `migration:apply:overlay`, `migration:reset`
- retired `db:drop`, `db:migrate`, `db:reset` with no aliases
- API `health` now includes `cargo check --workspace --all-features`
- `migrate_dev_db` takes `schema|overlay`
- `health_tests` uses `underlay_testing::TestServer` (`v0.9.4`, `server` feature)
- active README / architecture / agent notes; postgres named-volume wording

## Validation

- `effigy tasks` — `migration:*` present, no `db:*`
- `effigy state plan` — passed
- `effigy state apply local --yes` — passed against the proved local target
- `effigy acme-api/migration:apply` and `effigy acme-api/migration:reset` — passed
- `effigy acme-api/health` — passed
- targeted `api_tests` `health_tests` — passed
- `effigy workspace:js:prepare` then `effigy health` — passed
- `effigy acme-docs/qa:docs` — passed
- `effigy acme-docs/qa:northstar` — passed
- retired-selector search — no active `db:migrate` / `db:reset` / `db:drop`
- `git diff --check` — passed

`effigy validate` and `effigy qa` ran the API, admin, and client suites
successfully, then failed on `acme-front` because Vitest exits 1 when no test
files exist. That empty front suite is the accepted minimum posture and was
left unchanged.

## Residual Risk

- root built-in test auto-detects `acme-front` Vitest and fails on an empty
  suite
- unprepared host `bun x tsc` can fetch TypeScript 7 and reject `baseUrl`;
  `workspace:js:prepare` restores the pinned 5.9.3 path
- host `migration:*` tasks need `DATABASE_URL` pointing at
  `127.0.0.1:19932/acme`; the binaries do not invent that URL

## Next Task

Open the reviewable PR for this worker branch. Do not merge. After an
authorised merge, the orchestrator closes Underlay `g09.038` and promotes
`g09.039`–`g09.043`.
