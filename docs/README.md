# Acme Docs

Northstar-aligned documentation authority for the Underlay reference
implementation. This authority lives at root `docs/` and is addressed through
the `acme-docs` Effigy catalog alias.

Use this repo section for four things:
- `vision/` for the long-term purpose of the reference app
- `architecture/` for structural and technical decisions
- `policy/` for compact execution and authority rules
- `specs/` for active strict-lane execution wrappers when a milestone needs
  tighter control
- `roadmaps/` for executable work queues
- `logs/` for timestamped execution history tied to roadmap work

## Core structure

- `vision/`
- `architecture/`
- `policy/`
- `processes/`
- `specs/`
- `roadmaps/`
- `logs/`

## Reference app scope

The reference implementation covers:
- `apps/acme-api` for the Rust backend and jobs runtime
- `packages/acme-client` for the typed TypeScript API boundary
- `apps/acme-admin` for the admin frontend
- `apps/acme-front` for the public frontend
- `packages/acme-ui` for app-local shared UI pieces

## Effigy-First Repo Loop

Use Effigy as the default command surface from the workspace root, with
explicit `acme-docs/...` selectors for docs-authority work:

```bash
effigy tasks
effigy acme-docs/health
effigy test --plan
```

Repo-owned rollout checks:

```bash
effigy acme-docs/check:rollout admin-freshness
effigy acme-docs/check:rollout auth-security-alerting
effigy acme-docs/check:rollout reorder-conflict
effigy acme-docs/qa
effigy acme-docs/qa:docs
effigy acme-docs/qa:northstar
```

These tasks wrap the active rollout audit scripts in the workspace root so agents and contributors can use a consistent repo-local surface instead of direct shell invocations.

## How to use this docs set

- Start with [vision/001-acme-reference-implementation-vision.md](vision/001-acme-reference-implementation-vision.md).
- Use [architecture/000-overview.md](architecture/000-overview.md) for the package map and system layout.
- Use [architecture/product-guardrails.md](architecture/product-guardrails.md) for the current retained-surface guardrails.
- Use [policy/001-working-rules.md](policy/001-working-rules.md) for the active strict execution contract.
- Use [specs/README.md](specs/README.md) for the current ready-card surface.
- Read [processes/210-reference-implementation-notes.md](processes/210-reference-implementation-notes.md) for implementation notes and validation commands.
- Track active execution in [roadmaps/README.md](roadmaps/README.md).
- Record meaningful work batches in [logs/README.md](logs/README.md).

## Next Task

Orchestrator exact-head review of card 003, the Underlay v0.9.6 immutable
media adoption under `g01.013`. Resume the retained-surface card after this
rollout merges.
