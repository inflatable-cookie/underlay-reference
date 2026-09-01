# Batch Cards

Batch cards hold the bounded execution steps for active strict-lane work in
the reference implementation.

## Rules

- Keep only ready or still-governing cards here.
- A card should be ready only when it can be executed without fresh planning
  judgment.
- When a card closes, refresh the roadmap and front-door/currentness surfaces
  if they still advertise it as the active next move.

## Active batch card

None. `002-northstar-agents-rust-typescript-audit.md` is complete with its PR
open for orchestrator review.

Paused: `001-audit-retained-acme-admin-underlay-surface.md` until that PR
merges.

## Next Task

Orchestrator: review the card 002 PR at its exact head and merge, then resume
card 001.
