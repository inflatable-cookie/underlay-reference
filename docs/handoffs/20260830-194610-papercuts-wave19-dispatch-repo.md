---
title: Papercuts wave 19 owning-repo dispatch worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: awaiting-review
owner: Tom / papercuts orchestrator
created: 2026-08-30
updated: 2026-08-30
handoff_path: /Users/tom/Dev/projects/underlay-reference/docs/handoffs/20260830-194610-papercuts-wave19-dispatch-repo.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts]
---

## What This Thread Was Doing

The `g09.038` worker handoff lives in Underlay Reference, but T3 started
that thread in a clean Underlay worktree. `.agents.local.env` was also
absent, so the worker created a second worktree under the T3 Underlay
Reference container.

Northstar PR 8 already binds operator-facing dispatch to the owning
repo's **absolute** handoff path. Proved that protocol and closed the
copy. Did not start in Underlay.

## Why It Matters

The first preflight was spent proving the current root is the wrong repo.

## Current State

- **Repository:** `/Users/tom/Dev/projects/underlay-reference`
- **Planning branch:** `main`
- **Planning base commit:** `4dae06c1eb45cb7b34ea2d7ef6529d793a38f999`
- **Worker mode:** implementation complete; awaiting review.
- **Worker branch:** `t3code/fix-owning-repo-dispatch`
- **Worker worktree:** `/Users/tom/.t3/worktrees/underlay-reference/t3code-8b2ef9df`
  (launcher supplied clean non-`main` worktree; accepted)
- **Required sibling worktree links:** `none`
- **Done:**
  1. Cited Northstar `1840c9f6d4f7127240622a09e462b06adc094971` (PR 8);
     absolute owning-repo dispatch is the operator-facing artifact.
  2. `AGENTS.md` states that the operator-facing path is absolute and
     names this owning repo. Do not treat an Underlay-relative
     `docs/handoffs/…` lookup as the dispatch artifact.
  3. Added `.agents.local.env` to `.gitignore`. Did not create or commit
     the env file.
  4. Closed the matching papercut in `PAPERCUTS.md` under `## Closed`.
- **Out of scope left open:** editing Underlay or Northstar; T3 launcher
  code; GitHub workflows; unresolved Open papercuts (sibling mounts,
  stale Bun `file:` metadata, misaimed Vitest suites).
- **Validation evidence:**
  - `AGENTS.md` names absolute owning-repo dispatch.
  - `git check-ignore` covers `.agents.local.env`.
  - Reviewer at `b257f4a27b6ca0593985def948b4f0bfc68fbd2c`:
    `git diff --check` passed; `git check-ignore` proves ignore coverage;
    `effigy test --plan` resolved the expected three targets;
    `effigy acme-docs/qa:docs` passed; Northstar PR 8 SHA supports the
    absolute owning-repo dispatch rule.
- **PR URL:** https://github.com/inflatable-cookie/underlay-reference/pull/11
- **Merge authorisation:** absent; do not merge

## Boundaries

- Copy closeout is done (see Current State). Do not dispatch from
  Underlay. Review-only from here; merge is operator-authorised only.

## Important Context

- Nucleus wave 18 closed the same absolute-path copy.
- **Report to:** the operator.

## Suggested Next Move

Orchestrator review of
https://github.com/inflatable-cookie/underlay-reference/pull/11. Do not
relaunch implementation; evidence and PR URL are already recorded above.
Merge only with operator authorisation.

## Completion Protocol

### Review and merge path

Awaiting orchestrator review. Merge is operator-authorised only.

- **Closeout refs:** `PAPERCUTS.md`; this handoff; the PR.

### Handoff closeout

Northstar PR 8 (`1840c9f6d4f7127240622a09e462b06adc094971`) is the
governing rule; papercut closed. Do not commit `.agents.local.env`.
