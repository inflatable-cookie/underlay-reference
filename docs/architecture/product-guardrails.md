# Product Guardrails

Status: active
Owner: repo maintainers
Updated: 2026-04-10

## Purpose

Keep the reference implementation honest as a reusable example during the
Poodle-era coexistence and retained-Underlay normalization work.

## Guardrails

- Prefer reusable reference patterns over one-off app-local customization.
- Prefer explicit retained-Underlay boundaries over habitual retention.
- Prefer Poodle primitives and simple composites directly when the approved
  retained-surface contract does not say Underlay still owns the surface.
- Prefer documented reference-app rationale for retained heavy surfaces over
  vague “not migrated yet” explanations.
- Do not reopen already migrated foundational primitives just because retained
  routes still exist.
- Do not call a surface “retained” unless its structural, workflow, or
  data-heavy reason is explicit.
- Do not widen the retained-surface audit into another broad route-conversion
  sweep.
- Do not let the reference app drift into a mixed surface posture that
  downstream apps would have to rediscover by inspection.

## Next Task

Apply these guardrails against the frozen retained-surface contract in
`docs/architecture/004-retained-underlay-surface-contract.md`. Re-enter
planning before any downstream rollout wave.
