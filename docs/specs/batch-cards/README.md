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

None. Cards 001, 002, and 003 are archived after merged PRs 15, 13, and 14.

## Next Task

No execution card is ready. Re-enter planning before compiling another card.
