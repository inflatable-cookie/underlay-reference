# 001 - Audit Retained Acme-Admin Underlay Surface

Status: complete — merged as PR 15
Owner: retained-surface worker
Updated: 2026-09-03
Roadmap: g01.007
Spec: docs/specs/archive/001-retained-underlay-surface-strict-lane.md
Governing refs: docs/architecture/product-guardrails.md, docs/policy/001-working-rules.md

## Objective

Audit the meaningful retained Underlay surfaces still present in `acme-admin`
after the completed Poodle coexistence proof and freeze the approved retained
boundary as a durable contract artifact.

## Scope

- inspect the surviving retained Underlay surface groups in `acme-admin`
- classify each meaningful retained surface
- write the retained-surface contract artifact tied to `g01.007`
- refresh roadmap/currentness surfaces and log the batch closeout

## Out of Scope

- reopening migrated foundational primitives
- broad route-conversion work
- speculative downstream app rollout work

## Steps

- [x] audit the meaningful retained Underlay surfaces still present in
      `acme-admin`
- [x] classify retained surfaces into the approved groups already named by
      `g01.007`
- [x] write the durable retained-surface contract artifact for the reference
      app
- [x] refresh roadmap/front-door surfaces if the active next move changes
- [x] write the execution log with validation actually run

## Acceptance Criteria

- [x] the major retained Underlay surface groups in `acme-admin` are named and
      justified
- [x] the retained boundary is frozen in a durable doc artifact
- [x] `docs/roadmaps/README.md`, `docs/roadmaps/g01/README.md`, and
      `docs/logs/README.md` agree about the active lane state

## Validation

- [x] `effigy acme-docs/qa:docs`
- [x] `effigy acme-docs/qa:northstar`

## Stop Conditions

- the audit reveals a wider migration wave is needed before the retained
  boundary can be frozen honestly
- retained surfaces cannot be classified without fresh product/planning intent

## Next Task

Complete in PR 15. The frozen contract lives at
`docs/architecture/004-retained-underlay-surface-contract.md`; re-enter planning
before opening another execution card.
