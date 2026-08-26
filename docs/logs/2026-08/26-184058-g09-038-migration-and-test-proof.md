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

`effigy state apply local --yes` executed all three layers with no host
`DATABASE_URL`. Package `acme-api/migration:reset` then replayed the same
drop + schema + overlay sequence. Schema list, table list, and
`_sqlx_migrations` versions matched:

- 6 schemas (`account`, `acme`, `auth`, `media`, `platform`, `public`)
- 31 tables
- 8 structural migrations

Migration tasks run in the Effigy workspace container and load
`DATABASE_URL` from the app config stack when it is unset. Overlay remains a
separate package task. Seed failure exits non-zero, so both state apply and
reset stop instead of continuing.

## Changed Surfaces

- root `[state.local]` stack behind `effigy state plan` / `effigy state apply local --yes`
- `apps/acme-api` `migration:reset:schema`, `migration:apply`,
  `migration:apply:overlay`, `migration:reset`
- retired `db:drop`, `db:migrate`, `db:reset` with no aliases
- API `health` now includes `cargo check --workspace --all-features`
- `migrate_dev_db` takes `schema|overlay`
- `health_tests` uses `underlay_testing::TestServer` (`v0.9.4`, `server` feature)
- `acme-front` Vitest suite is configured `default = false` so the empty
  suite stays off the root test board
- active README / architecture / agent notes; postgres named-volume wording

## Validation

- `effigy tasks` — `migration:*` present, no `db:*`
- `effigy state plan` — passed
- `effigy state apply local --yes` — passed with no host `DATABASE_URL`
- `effigy acme-api/migration:reset` — passed with no host `DATABASE_URL`
- `effigy acme-api/health` — passed
- targeted `api_tests` `health_tests` — passed
- `effigy workspace:js:prepare` then `effigy health` — passed
- `effigy validate` — passed
- `effigy qa` — passed
- `effigy acme-docs/qa:docs` — passed
- `effigy acme-docs/qa:northstar` — passed
- retired-selector search — no active `db:migrate` / `db:reset` / `db:drop`
- `git diff --check` — passed

Review round 2 also re-ran from-empty apply and package reset without
undocumented shell state.

## Residual Risk

- unprepared host `bun x tsc` can fetch TypeScript 7 and reject `baseUrl`;
  `workspace:js:prepare` restores the pinned 5.9.3 path

## Next Task

PR4 is updated for orchestrator re-review. Do not merge. After an authorised
merge, the orchestrator closes Underlay `g09.038` and promotes
`g09.039`–`g09.043`.
