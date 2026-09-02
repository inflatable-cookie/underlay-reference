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

- `003-underlay-v0-9-6-immutable-media-adoption.md`

Card 002 is complete and merged as PR 13. Card 001 remains paused during this
rollout.

## Next Task

Orchestrator exact-head review of the card 003 PR under `g01.013`.
