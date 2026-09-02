# 003 - Underlay v0.9.7 Owned Media Recovery

Status: revision-ready
Owner: media worker
Created: 2026-09-02
Roadmap: `g01.013`
Spec: `docs/specs/003-underlay-v0-9-6-immutable-media-adoption.md`
Auto-start next card: no

## Objective

Pin the full workspace to released Underlay `v0.9.7` and make the live Acme
media finalisation path the canonical consumer proof of owned promotion and
restart recovery.

## Scope

- root and package manifests plus root `bun.lock` and API `Cargo.lock`;
- `apps/acme-api` media upload/finalisation, blob adapter use, version/current
  persistence, failure-capable composition tests, and one lane log;
- documentation needed to record the delivered contract and exact evidence.

## Ordered Work

1. Preflight the clean pushed base, applicable instructions, released tag and
   `promote_verified` signature. Reproduce the mutable/client-described path.
2. Integrate current pushed `main` into the existing PR 14 branch without
   rewriting history. Update every Underlay declaration and both locks to exact
   `v0.9.7`; prove no
   mixed, path, branch, or revision source remains.
3. Add the spec-authorized private migration. Generate and persist a fresh
   ownership token plus exact provider/bucket/destination authority before create.
4. Promote through `promote_verified_owned`; on restart recover only through
   `recover_owned_publication`. Persist returned/server-derived facts and commit
   ready/current atomically.
5. Make version delete and media purge remove required staging and destination
   blobs before deleting the durable row. Propagate cleanup failure for retry.
6. Drive every spec oracle row through real composition, run proportional
   Effigy validation, add one log, push, and open one PR.

## Acceptance And Review

Use the governing spec verbatim. Review must inspect the lock sources and try
source mutation, non-regular/oversized input, occupied destinations, forged
client metadata, post-promotion DB failure, and retry convergence.

## Stop Conditions

Use the governing spec. The narrow private migration is authorized. Stop rather
than choosing a DTO, broader schema, retention, external-service, or unsupported
adapter policy. Never edit Underlay or Poodle.

## Next Task

Resume the existing worker and PR 14. Do not create a replacement lane or merge.
