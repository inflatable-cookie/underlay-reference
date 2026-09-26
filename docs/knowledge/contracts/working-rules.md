# Working rules

Owner: repo maintainers
Depends on: [architecture overview](../architecture/000-overview.md)

## Execution

- Start through `docs/README.md`, `docs/plan.md`, `effigy tasks`, and
  `effigy test --plan`.
- Keep a change bounded to one coherent outcome.
- When the next direction is materially ambiguous, stop and ask instead of
  inventing the next wave.
- Update code, knowledge, tests, and indexes together when they form one
  observable change.
- Record process friction in `PAPERCUTS.md`.

## Retained-surface rule

The [retained-surface contract](../architecture/004-retained-underlay-surface-contract.md)
is frozen. Treat retained Underlay surfaces as a contract-definition problem,
not a fresh route-migration sweep. If an audit shows a wider migration wave is
needed, stop and re-enter planning. Changing the contract needs a plan item;
see Q-001.

## Definition of done

A change is done when its scoped behavior is implemented, the relevant Effigy
checks pass, the owning knowledge files match the result, and any remaining
gate is named rather than implied complete.
