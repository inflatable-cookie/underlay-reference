# g01.013 Underlay v0.9.6 Immutable Media Adoption

Status: delivered-at-pr
Owner: media worker
Date: 2026-09-02
Roadmap: `docs/roadmaps/g01/013-underlay-v0-9-6-immutable-media-adoption.md`
Spec: `docs/specs/003-underlay-v0-9-6-immutable-media-adoption.md`
Card: `docs/specs/batch-cards/003-underlay-v0-9-6-immutable-media-adoption.md`
Handoff: `docs/handoffs/20260902-200928-underlay-v0-9-6-immutable-media-adoption.md`

## Run Identity

- Worker branch: `worker/underlay-v0-9-6-immutable-media-adoption`
- Worker worktree: `/Users/tom/.paseo/worktrees/119ajruu/underlay-v0-9-6-media`
- Planning base `135fab451a90ad28ea538422c9e435cbb164d326` is an ancestor of
  launch `HEAD` `e60354adad62a109e7e148999cf45c55bb96738f`, which equalled
  `origin/main` at preflight.
- Tracked handoff in that `HEAD` is byte-identical to the dispatch file.
- Underlay tag `v0.9.6` peels to
  `4f6d75522c553fa9279b1ce36871ccc1cc1ce99d`.
- Sibling links in the worktree container: `underlay ->
  /Users/tom/Dev/projects/underlay`, `poodle -> /Users/tom/Dev/projects/poodle`.

## What Changed

Every Underlay Cargo and JavaScript declaration, plus both root-owned locks,
now resolve released tag `v0.9.6` at commit `4f6d7552`. No path, branch, or
revision override remains.

Live Acme finalisation no longer mutates the client upload key in place or
persist client `sha256` / hardcoded provider and bucket. It captures staging
once through `BlobAdapterPromotionExt::promote_verified`, publishes to a
distinct `…/published/…` destination with exclusive create, and commits
ready state plus `current_version_id` in one transaction. Every destination
collision, including identical bytes, is refused. Staging identity is
persisted at initiate; promotion facts are committed before activation so a
DB failure cannot strand recovery. Retry reloads that identity from Postgres
and does not reread mutable staging.

Public `FinaliseUploadRequest` / `FinaliseUploadResponse` are unchanged.

## Oracle Evidence

Composition tests in
`apps/acme-api/crates/api/src/tests/routes/admin/media_finalise_promotion_tests.rs`
drive the real `finalise_upload` handler (and `finalise_upload_with` for the
injected activation failure) against a failure-capable in-memory blob adapter
and Postgres:

| Oracle | Test |
| --- | --- |
| Staging mutates after capture | `captured_bytes_are_published_when_staging_mutates_after_capture` |
| Oversized / unreadable source | `oversized_or_unreadable_source_refuses_before_publication` |
| Occupied destination, forged or identical bytes | `occupied_destination_preserves_incumbent_even_with_identical_or_forged_bytes` |
| Occupied destination plus post-capture staging swap | `occupied_destination_refuses_after_staging_mutates_post_capture` |
| Forged client digest / identity | `forged_client_metadata_is_ignored_and_persisted_facts_are_server_derived` |
| In-transaction ready/current rollback + durable retry | `activation_failure_keeps_identities_and_retry_does_not_duplicate` |
| Crash after exclusive create, before fact recording | `crash_after_promote_recovers_from_destination_and_delete_cleans_it` |
| Declared MIME vs bytes | `mismatched_declared_mime_refuses_before_publication` |

Initiate policy tests remain (3). Finalisation composition tests: 8. Total focused suite: 11.

## Lock Sources

- JS declarations: `git+ssh://git@github.com/inflatable-cookie/underlay.git#v0.9.6`
- `bun.lock` package: `#4f6d75522c553fa9279b1ce36871ccc1cc1ce99d`
- Cargo declarations: `tag = "v0.9.6"`
- `apps/acme-api/Cargo.lock`:
  `git+ssh://git@github.com/inflatable-cookie/underlay.git?tag=v0.9.6#4f6d75522c553fa9279b1ce36871ccc1cc1ce99d`

## Validation

- focused acme-api oracle + initiate tests: 11 passed against Postgres 16
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `effigy workspace:js:prepare`
- `effigy acme-admin/check`, `acme-client/check`, `acme-ui/check`,
  `acme-front/check`
- `effigy acme-docs/qa:docs` and `qa:northstar` (after this log)
- `git diff --check origin/main...HEAD`

`effigy container up` could not start the Effigy-owned Postgres (vault TTY).
Oracle tests used a throwaway `postgres:16` plus host `migrate_dev_db`.
Root `effigy validate` still fans into sibling Underlay and is not this
lane's gate (open papercut).

## Review Repair

Exact-head review at `210e954f` required three blockers. This revision:

1. Removed `promote_or_converge` / `converge_existing_destination`. Every
   `DestinationExists` is a 409; identical-byte incumbents stay unchanged.
2. Persisted the staging key at version insert and committed server-derived
   promotion facts before activation. Retry reloads that row after dropping
   in-memory keys and after a filename rename.
3. Replaced `FailingStore` with `activate_ready_current_failing_after_version_ready`,
   which raises `SELECT 1 / 0` inside the real transaction after the version-ready
   write. Fresh queries still show `uploading` and a null current pointer.

Re-review at `ed5db4ab` required two further blockers:

1. Publication intent (adapter provider/bucket) is committed before
   `promote_verified`. A crash after exclusive create and before digest
   recording recovers from the immutable destination without rereading
   staging. Version delete/purge removes both staging and published keys.
2. This log and the PR body now state universal collision refusal and the
   actual focused suite count (11).

## Boundaries Held

- no Underlay or Poodle edits
- no public DTO change, migration, retention, or cleanup policy
- `g01.012` and `g01.007` untouched
- no workflow, release, deploy, or merge

## Next Task

Orchestrator exact-head review of this PR. Do not merge from this worker.
Resume `g01.007` after merge.
