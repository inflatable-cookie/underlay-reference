---
title: Underlay Reference v0.9.6 immutable media adoption
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / Northstar orchestrator
created: 2026-09-02
updated: 2026-09-02
handoff_path: /Users/tom/Dev/projects/underlay-reference/docs/handoffs/20260902-200928-underlay-v0-9-6-immutable-media-adoption.md
base_required: pushed-main
tags: [coordination, handoff, worker, underlay, media, integrity]
---

## Assignment

Implement `g01.013` card 003 as one PR. Pin every Underlay Cargo/JavaScript
declaration and root lock to released tag `v0.9.6`, then adopt
`BlobAdapterPromotionExt::promote_verified` in the live Acme media
finalisation path under spec 003's full oracle.

## Current State

- repository: `/Users/tom/Dev/projects/underlay-reference`;
- planning base: `135fab451a90ad28ea538422c9e435cbb164d326` on pushed `main`;
- worker branch: `worker/underlay-v0-9-6-immutable-media-adoption`;
- Underlay release: tag `v0.9.6`, commit
  `4f6d75522c553fa9279b1ce36871ccc1cc1ce99d`;
- authority: roadmap `g01.013`, spec 003, card 003, applicable `AGENTS.md`;
- overlapping `g01.012` audit and `g01.007` are paused; do not absorb them;
- integration and exact-head review belong to the orchestrator.

## Boundaries

The worker owns manifests, root locks, Acme media finalisation and persistence,
focused failure-capable tests, and one log. Underlay and Poodle are read-only.
Preserve the public DTO and successful response. Stop for migration, retention,
cleanup-policy, unsupported-adapter, missing-API, or unavailable DB/storage
oracle decisions. Do not edit workflows, release, deploy, resolve other lanes,
or merge.

## Preflight And Proof

Read the tracked handoff, all applicable instructions, spec 003, roadmap 013,
and card 003. Fetch origin; require a clean non-main worker with `HEAD ==
origin/main`, the planning base as ancestor, and byte-identical tracked/absolute
handoff. Use Effigy task inventory, graph where useful, and test planning before
commands. Meet every spec oracle row through real handler/blob/DB composition,
inspect both locks for exact released sources, validate proportionally, push,
open one PR, and report URL plus exact head. Revisions return to this worker.
