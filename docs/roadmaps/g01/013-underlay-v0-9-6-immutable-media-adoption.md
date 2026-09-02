# g01.013 Underlay v0.9.6 Immutable Media Adoption

Status: active
Owner: repo maintainers
Created: 2026-09-02
Governing spec: `docs/specs/003-underlay-v0-9-6-immutable-media-adoption.md`
Planning state: card 003 ready; `g01.012` card 002 paused intact for overlap

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
parallel. Within this repository, card 003 runs alone because the broad
`g01.012` audit owns overlapping media and dependency surfaces. Resume that
audit after this PR merges; `g01.007` remains paused behind it.

## Next Task

Execute card 003 and stop at its PR for orchestrator exact-head review.
