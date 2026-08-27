# g09.059 Underlay Reference Task Batch-Delete Colon Cutover

Date: 2026-08-27
Roadmap: Underlay `g09.059`
Handoff: `docs/handoffs/20260827-181956-g09-059-task-batch-delete-colon.md`

## Scope

Atomic cutover of nested admin task batch-delete from slash to colon
grammar. Collection semantics, `POST`, payload, audit, and handler body
unchanged.

- new: `POST /v1/admin/projects/{project_id}/tasks:batch-delete`
- retired: `POST /v1/admin/projects/{project_id}/tasks/batch-delete`

## Compatibility Decision

Closed-world caller set. No slash alias. Compatibility window: none.

Supported callers before cutover: admin router mount, Acme Client
`batchDeleteTasks`, and its focused path test. Admin UI calls the client
command only. No external fleet callers in this repo.

## Changed Surfaces

- `apps/acme-api/crates/api/src/routes/admin/router.rs` — colon mount plus
  source inventory proof that the slash path is absent
- `packages/acme-client/src/commands/admin/task-commands.ts` — colon path
- `packages/acme-client/tests/commands/admin/task-commands.test.ts` —
  positive colon call and negative slash-path check

Handler comment in `tasks.rs` already stated the colon form. Historical
logs and completed roadmaps left intact.

## Validation

- `cargo test -p acme-api nested_task_batch_delete_uses_colon_not_slash`
- `bun x vitest run tests/commands/admin/task-commands.test.ts` in
  `packages/acme-client` after `effigy workspace:js:prepare`
- `effigy acme-api/check`
- `effigy acme-client/check`
- `effigy acme-docs/qa:docs`
- `effigy acme-docs/qa:northstar`
- root `effigy validate` — passed after rustfmt on the grammar test
- `git diff --check`

## Residual Risk

- `tasks/batch-update` remains slash-mounted while its handler comment
  already shows colon; out of scope for this lane
- parallel papercuts PR (#8) may still race root Vitest via `bun x vitest`;
  not absorbed here
- external `g09.059` stays open until both target PRs merge and fleet
  closeout records tips

## Next Task

Orchestrator exact-head review of this PR. Merge only with explicit
operator authorisation. Do not mark Underlay `g09.059` complete from this
lane alone.
