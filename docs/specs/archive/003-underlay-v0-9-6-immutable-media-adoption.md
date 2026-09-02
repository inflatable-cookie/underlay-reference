# Underlay v0.9.7 Owned Media Recovery

Status: complete — PR 14 merged as `0585c926`
Owner: repo maintainers
Created: 2026-09-02
Roadmap: `g01.013`
Released authority: Underlay tag `v0.9.7`, release commit
`8a7ce84b0501f6902da3ec1daf03f67ef0f42d4f`, Contract 040

## Outcome

Make the reference app the first clear consumer proof for Underlay v0.9.7.
Every Cargo and JavaScript declaration resolves the same released tag, and the
live media finalisation path uses token-bound owned promotion and recovery.

The filename is retained because this is a revision of the existing Card 003 / PR
14 lane, not a replacement lane.

## Required Boundary

- capture source bytes once with a bounded read; digest, size, MIME validation,
  and publication describe those bytes;
- publish to a distinct immutable destination with exclusive create;
- refuse every collision while preserving the incumbent object;
- derive persisted digest, size, MIME, provider, bucket, and destination key
  from server-owned evidence rather than the client request;
- commit ready state and `current_version_id` atomically;
- generate a fresh server-owned ownership token of at least 32 random bytes for
  each publication and persist it with the exact provider, bucket, and destination
  key before exclusive create;
- recover after process loss only through `recover_owned_publication` using that
  durable token and immutable authority; intent, key secrecy, ETag, staging, and
  byte equality are never ownership evidence;
- retain staging, owned destination, and token identity until required blob
  cleanup succeeds; delete and purge propagate cleanup failure and retain the row
  for retry;
- preserve the existing public DTO and successful response unless a stop
  condition requires new authority.

## Authorized Private Migration

Add the smallest app-owned migration needed to persist owned-publication state:

- a private `bytea` ownership-token column, never selected into public DTOs,
  rendered, logged, or returned;
- a private destination-key column that remains distinct from the staging
  `object_key` until activation;
- constraints making token and destination authority either complete or absent.

The existing `storage_provider` and `bucket` columns may carry the remaining
immutable authority. Keep the token and destination identity through ready state
so delete and purge can retry cleanup. This migration is authorized; any public
DTO change, token exposure, retention threshold, or unrelated schema redesign is
not.

## Dependency And Lock Contract

- all Underlay JavaScript declarations use
  `git+ssh://git@github.com/inflatable-cookie/underlay.git#v0.9.7`;
- all Underlay Cargo declarations use the SSH Git source with `tag = "v0.9.7"`;
- root `bun.lock` and API `Cargo.lock` resolve the released tag/commit with no
  path, branch, or revision override;
- Underlay and other sibling repositories remain read-only.

## Review Oracle

| Counterexample | Required result |
| --- | --- |
| Caller path changes after capture. | Published bytes and metadata still match the captured object. |
| Source is oversized, non-regular, or unreadable. | Refuse before publication; no ready/current mutation. |
| Destination already exists, including identical bytes or forged metadata. | Refuse; incumbent bytes and metadata remain unchanged. |
| Process stops after intent but before create, then a foreign object occupies the destination. | Refuse; intent alone cannot authorize adoption. |
| Process stops after owned create but before facts/activation; staging is then missing or hostile. | Recover from token-bound object metadata and immutable authority without reading staging. |
| Token, provider, bucket, or destination authority mismatches. | Refuse recovery; do not mutate the incumbent or database lifecycle state. |
| Database activation fails after promotion. | Exact promoted/staging identity remains durably recoverable; retry cannot duplicate the object. |
| Blob deletion fails during version delete or media purge. | Return failure and retain the row plus every cleanup identity for retry. |
| Client supplies a false digest, size, MIME, provider, bucket, or final key. | Client value is rejected or ignored as authority; persisted facts are server-derived. |
| Successful retry or first completion. | Exactly one immutable object, one ready version, and the exact current pointer commit. |

Tests must drive the real handler/service composition with failure-capable blob
and database seams. Helper-only assertions are insufficient.

## Stop Conditions

Stop for a public DTO change, schema work beyond the authorized private migration,
retention threshold, cleanup policy choice, unsupported production adapter, or a
database/storage oracle that cannot execute. Do not weaken the invariant or
mutate Underlay to continue.

## Validation

Use repository-owned Effigy routes: task inventory and test plan first, frozen
root JS preparation, focused media/blob and database oracles, package checks,
`effigy validate`, docs/Northstar QA, lock-source inspection, and
`git diff --check origin/main...HEAD`.

## Next Task

Complete. PR 14 merged as `0585c926db9678e5d64b3ca3e29071a37e964c8c`.
Resume `g01.007` Card 001.
