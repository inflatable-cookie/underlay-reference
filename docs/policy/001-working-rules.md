# 001 - Working Rules

Status: active
Owner: repo maintainers
Updated: 2026-04-10
Depends on: docs/architecture/000-overview.md
Affects: acme-docs, acme-admin, acme-front, acme-ui, acme-client, acme-api

## Contract

### Delivery grammar

- Material work should follow this chain:
  `vision -> architecture + policy -> roadmap milestone -> spec/batch card -> execution -> evidence -> closeout`.
- In this repo, `policy/` is the compact contract surface that carries working
  rules and execution posture.
- In a strict lane, a bare `continue` should resolve through the previous
  closeout's `Next Task`, which should normally point at the current ready card
  or an explicit stop/reassessment step.
- When planning is needed and the next direction is materially ambiguous, stop
  and ask for intent instead of inventing the next wave.

### Strict retained-surface lane rules

- Execute from the current ready card, not from roadmap summary alone.
- Card 003 under `g01.013` merged as PR 14. The retained-surface owner in
  `g01.007` and Card 001 is now active.
- Treat retained Underlay surfaces as a contract-definition problem, not a
  fresh route-migration sweep.
- If the retained-surface audit shows a wider migration wave is needed, stop
  and re-enter planning instead of widening execution implicitly.

### Definition of done

Work in the strict lane is not done unless:

- the card, roadmap, and front-door currentness surfaces agree
- the retained-surface outcome is captured in a durable doc artifact
- required docs validation has actually run
- one explicit next task remains unless the lane is genuinely complete

### Closeout pattern

- update the current batch card first
- update the active roadmap milestone if progress or readiness changed
- refresh front-door/currentness surfaces that still name the active lane or
  ready card
- write the batch log with evidence and validation actually run
- leave one explicit next task in the highest-authority active surface

## Next Task

Execute `g01.007` Card 001 and stop at its PR for exact-head review.
