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

Paused: card 002 during the overlapping rollout; card 001 remains paused behind
card 002.

## Next Task

Execute card 003 under `g01.013` and stop at its PR for orchestrator review.
