# Roadmaps

Roadmaps hold executable work for the reference implementation.

## Rules

- Active roadmap files live in generation folders such as `g01/`.
- File names use `NNN-slug.md` with numbering local to the generation.
- References should use roadmap IDs such as `g01.003`.
- Generation rollover is manual only.
- Treat a generation as a substantial run of roadmap work, normally around 20 to 40 milestones rather than a short convenience bucket.
- Do not roll to a new generation until every roadmap in the current one is explicitly closed, superseded, or rehomed and the stale strict-lane material for that generation has been purged from `specs/`.
- Backlog items that are not active milestones belong in `backlog/`.

## Current generation

- Active generation: `g01`
- Next roadmap ID: `g01.014`
- `g01.013` is the active Underlay v0.9.6 immutable media adoption with ready
  card 003
- `g01.012` card 002 is paused intact during the overlapping rollout
- `g01.007` and card 001 are paused during the overlapping audit
- `g01.008`–`g01.011`, the g08 consumer-audit tranche, are complete

## Index

- [generation-index.md](generation-index.md)
- [g01/README.md](g01/README.md)
- [backlog/README.md](backlog/README.md)

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

Execute `g01.013` card 003 and stop at its PR for orchestrator review. Resume
`g01.012` afterwards; `g01.007` remains paused behind it.
