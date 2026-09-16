---
kind: northstar-handoff
title: "g01.016 — Adopt published Poodle 0.4.2"
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom
created: 2026-09-16
updated: 2026-09-16
base_required: pushed-main
queue_dispatch: northstar-queue
queue_approval: "Operator direction in the Longhorn thread on 2026-09-16: sweep every consumer for stale Poodle pins and dispatch an update per repository."
roadmap: docs/roadmaps/g01/016-adopt-published-poodle-0-4-2.md
tags: [coordination, handoff, worker, pr, poodle-adoption]
---

## What This Thread Was Doing

A cross-project sweep found this repository pinning Poodle below the published
`0.4.2`. This lane moves it current.

## Why It Matters

The pinned Poodle is a published release behind. Moving to `0.4.2` picks up the
fixes on the public registry and keeps this repository on the same Poodle the
sibling projects now use.

## Current State

- **Done:** the release is published; this task and handoff are committed.
- **Still open:** the pin bump and any 0.3 → 0.4 call-site migration.
- **Current task:** `docs/roadmaps/g01/016-adopt-published-poodle-0-4-2.md` (`g01.016`).
- **Canonical refs:** Poodle CHANGELOG at tag `v0.4.2`; the canonical task.
- **Remaining continuation envelope:** none.
- **Lane budget / pause signal:** normal; stop on the task's stop conditions.
- **Required sibling worktree links:** none.
- **Key files:** `apps/acme-admin/package.json`, `apps/acme-front/package.json`, and `packages/acme-ui/package.json`

## Boundaries

- **In scope:** the pin bump from `0.3.0` to `0.4.2` and the call-site
  migration the release forces.
- **Out of scope:** product behaviour, other repositories, and any new feature.
- **Repo constraints:** follow this repository's `AGENTS.md`; run its checks.

## Important Context

The 0.3 → 0.4 breaking changes are listed in the canonical task. No shims: if a
removed API has no direct replacement, stop and report rather than vendoring or
patching Poodle.

### UI Design Brief

Not applicable.

## Suggested Next Move

Read the canonical task, bump the pins, run the checks, and fix whatever the
0.4 release broke.

## Completion Protocol

Work on the queue-owned branch, not `main`. Bump the pins, absorb the breaking
changes, run `effigy qa`, open a PR, and report `ready_for_review` through the
Queue callback helper. Fix review findings on the same branch. Stop and report
if a change needs a product decision the operator has not made.
