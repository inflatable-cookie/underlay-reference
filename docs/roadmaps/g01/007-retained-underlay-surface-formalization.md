# g01.007 Retained Underlay Surface Formalization

Status: active
Owner: repo maintainers
Updated: 2026-09-03
Governing refs: `docs/architecture/product-guardrails.md`, `docs/policy/001-working-rules.md`, `docs/specs/001-retained-underlay-surface-strict-lane.md`
Planning state: Card 001 executed — contract frozen at `docs/architecture/004-retained-underlay-surface-contract.md`, PR pending review

## Goal

Turn the completed Poodle coexistence proof into an explicit retained-Underlay contract for the reference implementation so downstream app rollouts do not have to rediscover the boundary route by route.

## Why this matters now

`g01.006` proved that `acme-admin` can move its foundational UI layer onto Poodle while leaving selected Underlay surfaces in place. The remaining risk is no longer whether coexistence works. The remaining risk is drift: without a written retained-surface contract, future migrations may either keep too much in Underlay by habit or push domain-heavy/data-heavy surfaces into Poodle without a clear reusable boundary.

## Scope

- [x] audit the meaningful retained Underlay surfaces still present in `acme-admin` after `g01.006`
- [x] classify each retained surface as one of:
  - intentionally retained structural shell
  - intentionally retained data-heavy or workflow-heavy surface
  - reference-app/domain-specific surface
  - candidate for future Poodle review, but not part of `g01.007`
- [x] record the approved retained-surface contract for the reference implementation in a durable doc artifact tied to this roadmap
- [x] capture the practical migration rule for downstream apps:
  - use Poodle directly for primitives and simple composites
  - retain Underlay only where the approved contract says it still owns the surface
- [x] do not reopen already migrated foundational primitives or widen this milestone back into another route-conversion sweep

## Deliverables

- [x] one retained-surface contract artifact for `acme-admin` that names the major surviving Underlay surface groups and why they remain — `docs/architecture/004-retained-underlay-surface-contract.md`
- [x] one roadmap-aligned execution log that records the contract-opening or completion batch — `docs/logs/2026-09/03-003654-g01-007-retained-surface-contract.md`
- [x] updated roadmap indexes pointing to `g01.007` as the active milestone

## Validation

- [x] run `effigy docs check-links README.md vision/README.md roadmaps/README.md logs/README.md` from `docs/`
- [x] confirm `docs/roadmaps/README.md` and `docs/roadmaps/g01/README.md` both show `g01.007` as the active milestone
- [x] ensure the retained-surface artifact uses repo-root path references and stays aligned with the completed `g01.006` outcome

## Next

Card 001 is executed. Review the retained-surface contract PR at its exact
head; downstream rollout planning starts only after that review.
