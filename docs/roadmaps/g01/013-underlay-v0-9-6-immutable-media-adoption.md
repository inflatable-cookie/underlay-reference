# g01.013 Underlay v0.9.6 Immutable Media Adoption

Status: active
Owner: repo maintainers
Created: 2026-09-02
Governing spec: `docs/specs/003-underlay-v0-9-6-immutable-media-adoption.md`
Planning state: card 003 ready; `g01.012` card 002 complete and merged as PR 13

## Purpose

Prove the released immutable blob-promotion contract in the canonical Underlay
consumer and move the whole workspace to one `v0.9.6` dependency identity.

## Scope

- every Cargo and JavaScript Underlay declaration and both root-owned locks;
- the Acme API media upload/finalisation handler, blob seam, persistence
  activation boundary, focused tests, and one delivery log;
- no Underlay/Poodle edits, public DTO redesign, migration, retention policy,
  workflow, deployment, or release work.

## Acceptance

- [ ] all declarations and locks resolve exact released tag `v0.9.6`;
- [ ] live finalisation uses `promote_verified` over bounded captured bytes and
      an immutable create-only destination;
- [ ] collisions preserve the incumbent and every persisted blob fact is
      server-derived;
- [ ] ready/current activation is atomic and failure leaves durable recovery
      identity;
- [ ] the spec review oracle and repository validation pass at one exact PR
      head.

## Dependencies And Parallelism

Underlay `v0.9.6` is released. Other consumer repositories may implement in
parallel. The broad `g01.012` audit completed and merged before dispatch.
`g01.007` remains paused during this dependency/media rollout and resumes after
its PR merges.

## Next Task

Orchestrator exact-head review of the card 003 PR. Resume `g01.007` after
merge.
