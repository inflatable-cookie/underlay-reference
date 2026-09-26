# Agents Guide: Acme Docs

## Scope

Root `docs/` is the documentation authority for the Underlay reference
implementation, addressed through the `acme-docs` Effigy catalog alias. Keep
knowledge and intent here rather than in package-local docs.

## Hard Rules

- Current truth lives in `knowledge/`, one owner per fact. See `knowledge/README.md`.
- Intent lives in `plan.md`. Unresolved leads live in `triage/` and are never
  authority.
- Do not leave compatibility shim docs behind after moves; update links in place.
- Prefer Underlay source docs for shared framework doctrine and root `docs/`
  for reference-app-specific application.

## Effigy-First Execution

Root `AGENTS.md` owns the runtime stance; this section only adds what is local
to this scope.

Docs work is addressed through explicit `acme-docs/...` selectors — `effigy
acme-docs/health`, `effigy acme-docs/validate`, `effigy acme-docs/qa:docs`,
`effigy acme-docs/qa:northstar` — so a docs change does not silently trigger
whole-workspace validation. Run `effigy validate` only when the broader
workspace genuinely needs it. Prefer the docs-authority rollout tasks over
calling the shell scripts directly.

Repo notes:
- `acme-docs/health` is the stable baseline for day-to-day docs validation
- `acme-docs/validate` runs the full current rollout-check set
- direct script execution is fallback only when debugging a specific check

## Validation

- `effigy acme-docs/health`
- `effigy acme-docs/validate`
- `effigy acme-docs/qa:docs`

## Reference Docs

- `knowledge/README.md`
- `knowledge/vision.md`
- `knowledge/architecture/000-overview.md`
- `knowledge/operations/reference-implementation-notes.md`
- `plan.md`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `knowledge/contracts/writing-style.md`
