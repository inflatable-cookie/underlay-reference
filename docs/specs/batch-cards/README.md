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

- `002-northstar-agents-rust-typescript-audit.md`

Paused: `001-audit-retained-acme-admin-underlay-surface.md` until card 002
closes.

## Next Task

Execute card 002 under `g01.012` and stop at its PR for orchestrator review.
