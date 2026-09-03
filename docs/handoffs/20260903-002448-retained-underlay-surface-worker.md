---
title: Audit the retained acme-admin Underlay surface
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
status: merged-pr-15
owner: Tom / Northstar orchestrator
created: 2026-09-03
updated: 2026-09-03
base_required: pushed-main
tags: [coordination, handoff, worker, docs, underlay, poodle]
---

## Assignment

Execute `g01.007` Card 001 as one docs-only PR. Audit the meaningful retained
Underlay surfaces in `acme-admin`, classify them using the four categories in
the roadmap, and freeze the approved reference-app boundary in one durable
contract artifact.

## Boundaries

- repository: `/Users/tom/Dev/projects/underlay-reference`;
- authority: `docs/roadmaps/g01/007-retained-underlay-surface-formalization.md`,
  `docs/specs/archive/001-retained-underlay-surface-strict-lane.md`, archived
  Card 001,
  product guardrails, and working rules;
- inspect live source as evidence, but do not edit production code;
- do not reopen migrated Poodle primitives, perform a route conversion, invent
  downstream rollout work, edit Underlay/Poodle, or touch Bughunt state;
- if a meaningful surface cannot be classified without product intent, stop
  rather than guessing.

## Required Output

- one durable retained-surface contract naming each major surviving group,
  representative source paths, category, owner, and why it remains;
- explicit rules for downstream apps: Poodle for primitives/simple composites;
  Underlay only where the retained contract names structural, workflow-heavy,
  or data-heavy ownership;
- Card 001, g01.007, front-door, and execution-log closeout consistent with the
  evidence;
- repository-owned docs/Northstar QA and range diff check.

Push one branch, open one PR, and stop at its exact head for orchestrator review.
