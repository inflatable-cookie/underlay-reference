# Underlay v0.9.6 Immutable Media Adoption

Status: active
Owner: repo maintainers
Created: 2026-09-02
Roadmap: `g01.013`
Released authority: Underlay tag `v0.9.6`, release commit
`4f6d75522c553fa9279b1ce36871ccc1cc1ce99d`, Contract 040

## Outcome

Make the reference app the first clear consumer proof for Underlay v0.9.6.
Every Cargo and JavaScript declaration resolves the same released tag, and the
live media finalisation path promotes verified captured bytes to a new immutable
object through `BlobAdapterPromotionExt::promote_verified`.

## Required Boundary

- capture source bytes once with a bounded read; digest, size, MIME validation,
  and publication describe those bytes;
- publish to a distinct immutable destination with exclusive create;
- refuse every collision while preserving the incumbent object;
- derive persisted digest, size, MIME, provider, bucket, and destination key
  from server-owned evidence rather than the client request;
- commit ready state and `current_version_id` atomically;
- retain exact staging identity after post-promotion failure so retry or cleanup
  remains possible and explicit;
- preserve the existing public DTO and successful response unless a stop
  condition requires new authority.

## Dependency And Lock Contract

- all Underlay JavaScript declarations use
  `git+ssh://git@github.com/inflatable-cookie/underlay.git#v0.9.6`;
- all Underlay Cargo declarations use the SSH Git source with `tag = "v0.9.6"`;
- root `bun.lock` and API `Cargo.lock` resolve the released tag/commit with no
  path, branch, or revision override;
- Underlay and other sibling repositories remain read-only.

## Review Oracle

| Counterexample | Required result |
| --- | --- |
| Caller path changes after capture. | Published bytes and metadata still match the captured object. |
| Source is oversized, non-regular, or unreadable. | Refuse before publication; no ready/current mutation. |
| Destination already exists, including identical bytes or forged metadata. | Refuse; incumbent bytes and metadata remain unchanged. |
| Database activation fails after promotion. | Exact promoted/staging identity remains durably recoverable; retry cannot duplicate the object. |
| Client supplies a false digest, size, MIME, provider, bucket, or final key. | Client value is rejected or ignored as authority; persisted facts are server-derived. |
| Successful retry or first completion. | Exactly one immutable object, one ready version, and the exact current pointer commit. |

Tests must drive the real handler/service composition with failure-capable blob
and database seams. Helper-only assertions are insufficient.

## Stop Conditions

Stop for a public DTO change, schema migration, retention threshold, cleanup
policy choice, unsupported production adapter, missing released API, or a
database/storage oracle that cannot execute. Do not weaken the invariant or
mutate Underlay to continue.

## Validation

Use repository-owned Effigy routes: task inventory and test plan first, frozen
root JS preparation, focused media/blob and database oracles, package checks,
`effigy validate`, docs/Northstar QA, lock-source inspection, and
`git diff --check origin/main...HEAD`.

## Next Task

Orchestrator exact-head review of the card 003 PR. Resume `g01.007` after
merge.
