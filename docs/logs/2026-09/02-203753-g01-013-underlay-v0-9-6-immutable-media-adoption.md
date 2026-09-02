# g01.013 Underlay v0.9.7 Owned Media Recovery

Status: delivered-at-pr
Owner: media worker
Date: 2026-09-02
Roadmap: `docs/roadmaps/g01/013-underlay-v0-9-6-immutable-media-adoption.md`
Spec: `docs/specs/003-underlay-v0-9-6-immutable-media-adoption.md`
Card: `docs/specs/batch-cards/003-underlay-v0-9-6-immutable-media-adoption.md`
Handoff: `docs/handoffs/20260902-235000-underlay-v0-9-7-owned-recovery-resume.md`

## Run Identity

- Worker branch: `worker/underlay-v0-9-6-immutable-media-adoption`
- Worker worktree: `/Users/tom/.paseo/worktrees/119ajruu/underlay-v0-9-6-media`
- Existing lane: PR 14, agent `0bc493dc-7044-46f0-bd6f-a6d35ccfbe67`,
  workspace `wks_c08140ce3908419e`
- Prior head `bfb6a41b` plus `origin/main@ec0c77da` merged without rewrite
- Underlay tag `v0.9.7` peels to
  `8a7ce84b0501f6902da3ec1daf03f67ef0f42d4f`

## What Changed

Every Underlay Cargo and JavaScript declaration, plus both root-owned locks,
now resolve released tag `v0.9.7` at commit `8a7ce84b`. No path, branch, or
revision override remains.

Live Acme finalisation generates a fresh ≥32-byte ownership token, persists
token plus provider/bucket/destination before exclusive create, publishes
through `promote_verified_owned`, and recovers after process loss only through
`recover_owned_publication`. Intent, staging bytes, ETag, and byte equality
are not ownership evidence. Ready/current activation stays one transaction.
Delete and purge remove staging and owned destination before the row, return
blob cleanup failure, and retain token plus identities for retry.

Public `FinaliseUploadRequest` / `FinaliseUploadResponse` are unchanged. The
authorized private migration adds `ownership_token bytea` and
`published_object_key text` with complete-or-absent constraints. Token Debug
is redacted and never selected into DTOs.

## Oracle Evidence

Composition tests in
`apps/acme-api/crates/api/src/tests/routes/admin/media_finalise_promotion_tests.rs`
drive the real `finalise_upload` / `delete_version` / `purge_media` handlers
against a failure-capable in-memory blob adapter and Postgres:

| Oracle | Test |
| --- | --- |
| Staging mutates after capture | `captured_bytes_are_published_when_staging_mutates_after_capture` |
| Oversized / unreadable source | `oversized_or_unreadable_source_refuses_before_publication` |
| Occupied destination, forged or identical bytes | `occupied_destination_preserves_incumbent_even_with_identical_or_forged_bytes` |
| Occupied destination plus post-capture staging swap | `occupied_destination_refuses_after_staging_mutates_post_capture` |
| Forged client digest / identity | `forged_client_metadata_is_ignored_and_persisted_facts_are_server_derived` |
| In-transaction ready/current rollback + durable retry | `activation_failure_keeps_identities_and_retry_does_not_duplicate` |
| Pre-create crash plus foreign incumbent | `pre_create_crash_plus_foreign_incumbent_refuses` |
| Post-owned-create crash, staging deleted/mutated/hostile | `post_owned_create_crash_recovers_without_staging` |
| Wrong token / provider / bucket / destination | `wrong_token_provider_bucket_or_destination_refuses_without_mutation` |
| Delete and purge blob failure retain row | `delete_and_purge_blob_failure_retains_row_and_retry_converges` |
| Declared MIME vs bytes | `mismatched_declared_mime_refuses_before_publication` |

Initiate policy tests remain (3). Finalisation composition tests: 11. Total focused suite: 14.

## Lock Sources

- JS declarations: `git+ssh://git@github.com/inflatable-cookie/underlay.git#v0.9.7`
- `bun.lock` package: `#8a7ce84b0501f6902da3ec1daf03f67ef0f42d4f`
- Cargo declarations: `tag = "v0.9.7"`
- `apps/acme-api/Cargo.lock`:
  `git+ssh://git@github.com/inflatable-cookie/underlay.git?tag=v0.9.7#8a7ce84b0501f6902da3ec1daf03f67ef0f42d4f`

## Validation

Recorded after the focused oracle, fmt, clippy, and lock inspection.

`effigy container up` could not start the Effigy-owned Postgres (vault TTY).
Oracle tests used a throwaway `postgres:16` plus host `migrate_dev_db`.
Root `effigy validate` still fans into sibling Underlay and is not this
lane's gate (open papercut).

## Review History

Exact-head review at `210e954f` required three blockers (collision
convergence, non-durable recovery, mocked DB failure). Repair `ed5db4ab`.

Re-review at `ed5db4ab` required crash-window recovery and evidence fixes.
Repair `bfb6a41b` used publication intent.

Re-review at `bfb6a41b` required token-bound ownership: intent cannot
authorize a foreign incumbent, and delete/purge must not swallow blob
cleanup failure. v0.9.6 could not bind a consumer token to both Postgres
and the created object. This revision moves the same PR 14 lane to v0.9.7
owned promotion/recovery.

Re-review at `b2e3c357` required the test fault injection to leave the
production surface. `activate_ready_current_failing_after_version_ready`
now lives behind acme-db feature `test-faults`, enabled only by
`acme-test-utils`, and is re-exported from that crate. `FinaliseFault`,
the `fault` parameter, and `injected_crash` are `#[cfg(test)]`. Production
`activate_ready_current` has no fail arm. A `--release` `acme-api` binary
contains none of those symbols. The 11 finalisation tests still drive the
real handler, the shared ready/current writes, and the in-transaction
rollback. Non-blocking follow-ups were left untouched.

## Boundaries Held

- no Underlay or Poodle edits
- no public DTO change
- schema work limited to the two authorized private fields and constraints
- `g01.012` and `g01.007` untouched
- no workflow, release, deploy, or merge

## Next Task

Orchestrator exact-head review of this PR. Do not merge from this worker.
Resume `g01.007` after merge.
