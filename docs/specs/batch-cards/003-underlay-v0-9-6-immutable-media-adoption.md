# 003 - Underlay v0.9.6 Immutable Media Adoption

Status: ready
Owner: media worker
Created: 2026-09-02
Roadmap: `g01.013`
Spec: `docs/specs/003-underlay-v0-9-6-immutable-media-adoption.md`
Auto-start next card: no

## Objective

Pin the full workspace to released Underlay `v0.9.6` and make the live Acme
media finalisation path the canonical consumer proof of immutable verified blob
promotion.

## Scope

- root and package manifests plus root `bun.lock` and API `Cargo.lock`;
- `apps/acme-api` media upload/finalisation, blob adapter use, version/current
  persistence, failure-capable composition tests, and one lane log;
- documentation needed to record the delivered contract and exact evidence.

## Ordered Work

1. Preflight the clean pushed base, applicable instructions, released tag and
   `promote_verified` signature. Reproduce the mutable/client-described path.
2. Update every Underlay declaration and both locks to exact `v0.9.6`; prove no
   mixed, path, branch, or revision source remains.
3. Capture source bytes once within a limit, validate them, and promote to a
   distinct immutable destination through `promote_verified`.
4. Persist only returned/server-derived metadata. Make ready/current activation
   atomic and preserve exact recovery identity across each failure edge.
5. Drive every spec oracle row through real composition, run proportional
   Effigy validation, add one log, push, and open one PR.

## Acceptance And Review

Use the governing spec verbatim. Review must inspect the lock sources and try
source mutation, non-regular/oversized input, occupied destinations, forged
client metadata, post-promotion DB failure, and successful retry.

## Stop Conditions

Use the governing spec. Stop rather than choosing a DTO, migration, retention,
cleanup, adapter, or external-service policy. Never edit Underlay or Poodle.

## Next Task

Orchestrator exact-head review of the card 003 PR. Do not merge from the
worker. Do not start card 002.
