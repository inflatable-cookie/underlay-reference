# 001 - Audit Retained Acme-Admin Underlay Surface

Status: ready
Owner: repo maintainers
Updated: 2026-04-10
Roadmap: g01.007
Spec: docs/specs/001-retained-underlay-surface-strict-lane.md
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

- [ ] audit the meaningful retained Underlay surfaces still present in
      `acme-admin`
- [ ] classify retained surfaces into the approved groups already named by
      `g01.007`
- [ ] write the durable retained-surface contract artifact for the reference
      app
- [ ] refresh roadmap/front-door surfaces if the active next move changes
- [ ] write the execution log with validation actually run

## Acceptance Criteria

- [ ] the major retained Underlay surface groups in `acme-admin` are named and
      justified
- [ ] the retained boundary is frozen in a durable doc artifact
- [ ] `docs/roadmaps/README.md`, `docs/roadmaps/g01/README.md`, and
      `docs/logs/README.md` agree about the active lane state

## Validation

- [ ] `effigy acme-docs/qa:docs --repo ~/Dev/projects/underlay-reference`
- [ ] `effigy acme-docs/qa:northstar --repo ~/Dev/projects/underlay-reference`

## Stop Conditions

- the audit reveals a wider migration wave is needed before the retained
  boundary can be frozen honestly
- retained surfaces cannot be classified without fresh product/planning intent

## Next Task

Execute this card in one docs-only worker and stop at its PR for review.
