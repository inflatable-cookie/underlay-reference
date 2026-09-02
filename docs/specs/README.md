# Specs

Use this folder for strict-lane planning and batch cards while an active
reference-app milestone needs tighter execution control.

## Artifact types

- `NNN-<slug>.md`
  active strict-lane specs
- `batch-cards/NNN-<slug>.md`
  bounded execution cards derived from the active spec

## Rules

- Use specs when the roadmap needs a tighter execution wrapper than summary
  prose can provide.
- Keep one active strict-lane spec per live owner unless a broader planning
  fork forces a new surface.
- Use batch cards for work that should run from a bounded owner, not from fresh
  planning judgment.
- If there is no ready card, re-enter planning instead of guessing.
- Archive or delete stale strict-lane specs and batch cards once their owning
  roadmap work is fully implemented or otherwise closed.
- Do not roll a roadmap generation while stale generation-specific strict-lane
  material is still sitting in the active `specs/` tree.

## Archive

- Completed or superseded strict-lane material belongs in `archive/`.
- Completed or superseded batch cards belong in `archive/batch-cards/`.

## Active spec

- `003-underlay-v0-9-6-immutable-media-adoption.md` (v0.9.7 owned-recovery revision)

Spec 002 is complete and merged as PR 13. Spec 001 remains paused during this
rollout.

## Next Task

Resume Card 003 on existing PR 14 and stop at a new exact head for orchestrator
review. Resume spec 001 after merge.
