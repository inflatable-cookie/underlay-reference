# Roadmaps

Roadmaps hold only promoted executable work for the reference implementation.
Unresolved or deferred candidates live in `../triage/` until planning promotes
them into a roadmap milestone. Triage is never execution authority.

## Rules

- Active roadmap files live in generation folders such as `g01/`.
- File names use `NNN-slug.md` with numbering local to the generation.
- References should use roadmap IDs such as `g01.003`.
- Generation rollover is manual only.
- Treat a generation as a substantial run of roadmap work, normally around 20 to 40 milestones rather than a short convenience bucket.
- Do not roll to a new generation until every roadmap in the current one is explicitly closed, superseded, or rehomed and the stale strict-lane material for that generation has been purged from `specs/`.
- Unresolved or deferred candidates belong in `../triage/`, never in a second
  intake queue under `roadmaps/`. Migration into a roadmap is not approval:
  only planning promotes a candidate into an executable milestone.

## Current generation

- Active generation: `g01`
- Next roadmap ID: `g01.014`
- `g01.013` completed in merged PR 14
- `g01.012` card 002 is complete and merged as PR 13
- `g01.007` is complete in merged PR 15; the retained-surface contract is frozen
- `g01.008`–`g01.011`, the g08 consumer-audit tranche, are complete

## Index

- [generation-index.md](generation-index.md)
- [g01/README.md](g01/README.md)
- [../triage/README.md](../triage/README.md)

## Historical language boundary

- New roadmaps and actively maintained roadmap updates must use roadmap IDs and batch language.
- Imported roadmap content may retain phase-era wording when it is recording past implementation work.
- Normalize local historical wording only when a roadmap is reopened for active work or when an old label causes live path or reference drift.

## Rollover guardrail

Generation rollover is a closeout event, not a queue reset.

Before opening `gNN+1`:

- close, supersede, or rehome every roadmap still sitting in `gNN`
- refresh the roadmap front doors so they no longer advertise stale active work
- archive or delete stale generation-specific strict-lane specs and batch cards from `specs/`

## Next Task

No implementation card is ready. Re-enter planning before opening the next
task.
<!-- northstar:lifecycle:begin schema=northstar.lifecycle.projection.v2 digest=sha256:7f4e6c2a5f1f9a1985b068fc98f098daab81ea7a0557e28111b760b8a9bbf2bd -->
| Generation | Disposition | Runway state |
| --- | --- | --- |
| g01 | open | planning_required |
| Task | Status | Stage | Revision | Record digest |
| --- | --- | --- | --- | --- |
| g01.014 | complete | none | 8 | sha256:4dee75213a769e2735b3c74b73851aac34ad5ff43840b54b9fa902f332939473 |
| g01.015 | complete | none | 8 | sha256:943f2d28a6f065768a9174720e1b0111ff4a7a6f09b0a9742b02eddf7f58a34d |
| g01.016 | complete | none | 8 | sha256:22f379a3a2999a2ff95e5b10c116dfed358a527763875c71753eab276509a2ed |
<!-- northstar:lifecycle:end -->
