# g01.013 Underlay v0.9.7 Owned Media Recovery

Status: complete — PR 14 merged as `0585c926`
Owner: repo maintainers
Created: 2026-09-02
Governing spec: `docs/specs/archive/003-underlay-v0-9-6-immutable-media-adoption.md`
Planning state: complete

## Purpose

Prove released owned promotion/recovery in the canonical Underlay consumer and
move the workspace to one `v0.9.7` dependency identity.

## Scope

- every Cargo and JavaScript Underlay declaration and both root-owned locks;
- the Acme API media upload/finalisation handler, blob seam, persistence
  activation boundary, focused tests, and one delivery log;
- one narrow private migration for token and destination identity;
- no Underlay/Poodle edits, public DTO redesign, broader migration, retention policy,
  workflow, deployment, or release work.

## Acceptance

- [x] all declarations and locks resolve exact released tag `v0.9.7`;
- [x] live finalisation uses `promote_verified_owned` and token-bound recovery;
- [x] collisions preserve the incumbent and every persisted blob fact is
      server-derived;
- [x] ready/current activation is atomic; delete/purge retain durable recovery
      identity until required blob cleanup succeeds;
- [x] the spec review oracle and repository validation pass at one exact PR
      head.

## Dependencies And Parallelism

Underlay `v0.9.7` is released. Other consumer repositories may implement in
parallel. The broad `g01.012` audit completed and merged before dispatch.
`g01.007` resumed after this PR merged.

## Next Task

Complete. Resume `g01.007` Card 001.
