# g01.007 Retained Underlay Surface Formalization

Status: paused during overlapping `g01.012` repository audit
Owner: repo maintainers
Updated: 2026-04-10
Governing refs: `docs/architecture/product-guardrails.md`, `docs/policy/001-working-rules.md`, `docs/specs/001-retained-underlay-surface-strict-lane.md`
Planning state: paused; card 001 preserved

## Goal

Turn the completed Poodle coexistence proof into an explicit retained-Underlay contract for the reference implementation so downstream app rollouts do not have to rediscover the boundary route by route.

## Why this matters now

`g01.006` proved that `acme-admin` can move its foundational UI layer onto Poodle while leaving selected Underlay surfaces in place. The remaining risk is no longer whether coexistence works. The remaining risk is drift: without a written retained-surface contract, future migrations may either keep too much in Underlay by habit or push domain-heavy/data-heavy surfaces into Poodle without a clear reusable boundary.

## Scope

- [ ] audit the meaningful retained Underlay surfaces still present in `acme-admin` after `g01.006`
- [ ] classify each retained surface as one of:
  - intentionally retained structural shell
  - intentionally retained data-heavy or workflow-heavy surface
  - reference-app/domain-specific surface
  - candidate for future Poodle review, but not part of `g01.007`
- [ ] record the approved retained-surface contract for the reference implementation in a durable doc artifact tied to this roadmap
- [ ] capture the practical migration rule for downstream apps:
  - use Poodle directly for primitives and simple composites
  - retain Underlay only where the approved contract says it still owns the surface
- [ ] do not reopen already migrated foundational primitives or widen this milestone back into another route-conversion sweep

## Deliverables

- [ ] one retained-surface contract artifact for `acme-admin` that names the major surviving Underlay surface groups and why they remain
- [ ] one roadmap-aligned execution log that records the contract-opening or completion batch
- [ ] updated roadmap indexes pointing to `g01.007` as the active milestone

## Validation

- [ ] run `effigy docs check-links README.md vision/README.md roadmaps/README.md logs/README.md` from `~/Dev/projects/underlay-reference/docs`
- [ ] confirm `docs/roadmaps/README.md` and `docs/roadmaps/g01/README.md` both show `g01.007` as the active next milestone
- [ ] ensure the retained-surface artifact uses absolute path references and stays aligned with the completed `g01.006` outcome

## Next

Execute `g01.012` card 002 first. Resume card 001 afterwards; the repository
audit must not classify or close the retained Underlay boundary.
