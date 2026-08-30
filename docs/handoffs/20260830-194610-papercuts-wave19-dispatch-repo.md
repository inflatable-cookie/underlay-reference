---
title: Papercuts wave 19 owning-repo dispatch worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
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
repo's **absolute** handoff path. You are the Underlay Reference
implementation worker. Prove that and close the copy. Do not start in
Underlay.

## Why It Matters

The first preflight is spent proving the current root is the wrong repo.

## Current State

- **Repository:** `/Users/tom/Dev/projects/underlay-reference`
- **Planning branch:** `main`
- **Planning base commit:** `4dae06c1eb45cb7b34ea2d7ef6529d793a38f999`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved
  to that SHA before this handoff was created.
- **Planning checkout:** has unrelated dirty vendored Effigy skill
  files; they are not in this handoff. Worker uses pushed `origin/main`.
- **Worker mode:** implementation worker dispatched by the orchestrator.
- **Worker branch:** `worker/papercuts-wave19-dispatch-repo`
- **Worker worktree:** launcher first. `.agents.local.env` is absent in
  the planning checkout; if the launcher did not supply a clean
  dedicated non-`main` worktree, ask the operator for
  `AGENTS_WORKTREE_CONTAINER_DIR` before creating a fallback. Never use
  `/tmp`.
- **Required sibling worktree links:** `none`
- **Ready work items, in order:**
  1. T3 worker launch used the Underlay worktree for an Underlay
     Reference handoff — close if Northstar
     `1840c9f6d4f7127240622a09e462b06adc094971` (PR 8) requires the
     owning repo's absolute handoff path. Cite that SHA. Put a short
     note on `AGENTS.md`: operator-facing dispatch is this repo's
     absolute `docs/handoffs/…` path, not an Underlay-relative lookup.
     If fallback worktrees are needed, seed gitignored
     `.agents.local.env` with `AGENTS_WORKTREE_CONTAINER_DIR` after
     asking; do not commit the env file. Add `.agents.local.env` to
     `.gitignore` if it is missing there.
- **Out of scope:** editing Underlay or Northstar; T3 launcher code;
  GitHub workflows; the dirty vendored Effigy skill files in the
  planning checkout.
- **Canonical refs:** `PAPERCUTS.md`; `AGENTS.md`; Northstar PR 8
  (`1840c9f6d4f7127240622a09e462b06adc094971`).
- **Required validation:** `AGENTS.md` names absolute owning-repo
  dispatch. `.agents.local.env` is gitignored. Do not stage the env
  file.
- **PR URL:** pending
- **Merge authorisation:** absent; do not merge

## Boundaries

- Close the copy. Do not dispatch from Underlay. Do not merge.

## Important Context

- Nucleus wave 18 closed the same absolute-path copy. Match that
  language.
- **Report to:** the operator.

## Suggested Next Move

Read this file from the top. Run the worktree-safety preflight. After
the committed `HEAD` handoff checks out, skip sibling links (`none`),
then close the copy.

## Completion Protocol

### Before you start

1. Read this handoff path. Its `worker_mode: implementation` and
   `dispatch_authority: orchestrator` metadata activate worker mode. Then
   run `git rev-parse --show-toplevel`, `git branch --show-current`,
   `git status --porcelain`, and `git worktree list --porcelain`.
2. If the current root is a registered worktree, status is empty, and the
   branch is not `main`, accept it. Record the actual path/branch.
3. If the launcher supplied a dirty or `main` worktree, stop and report
   it. `.agents.local.env` was absent; ask before creating a fallback.
   Never use `/tmp`.
4. From the selected worktree, record the repository-relative path
   `docs/handoffs/20260830-194610-papercuts-wave19-dispatch-repo.md`.
   Confirm `HEAD == origin/main`, ancestor
   `4dae06c1eb45cb7b34ea2d7ef6529d793a38f999`, and that relative path in
   `HEAD`. Load
   `git show HEAD:docs/handoffs/20260830-194610-papercuts-wave19-dispatch-repo.md`.
   If the absolute dispatch file differs, stop. The `HEAD` copy is
   canonical.
5. Required sibling list is `none`. Skip link setup.
6. Read `AGENTS.md` and `PAPERCUTS.md`.

### When the assigned runway is complete

1. Update `PAPERCUTS.md`. Push a PR. Do not merge.

### Review and merge path

Awaiting orchestrator review. Merge is operator-authorised only.

- **Closeout refs:** `PAPERCUTS.md`; this handoff; the PR.

### Handoff closeout

Do not commit `.agents.local.env`.
