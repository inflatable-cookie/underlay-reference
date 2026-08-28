---
title: Papercuts wave 5 doctor docs and runtime-docs closeout worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-for-review
owner: Tom / papercuts orchestrator
created: 2026-08-28
updated: 2026-08-28
handoff_path: /Users/tom/Dev/projects/underlay-reference/docs/handoffs/20260828-164850-papercuts-wave5-docs-closeout.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts]
---

## What This Thread Was Doing

Wave 2 closed the vitest bin-link race and left Doctor's built-in `docs`
as an Effigy follow-up. Effigy already treats `docs` as a built-in in
task-reference checks. README already names
`underlay-reference-dev-postgres-data`. This repo still lists the Doctor
copy; Underlay still lists the postgres-docs copy.

You are the Underlay Reference implementation worker. Prove Doctor
against current Effigy and close that copy. Hunt remaining *active*
runtime docs that still claim a repo-local
`.effigy/runtime/data/postgres` bind-mount. Do not rewrite historical
g09.038 logs.

## Why It Matters

`effigy doctor` still looks broken on valid `docs check ...` steps, and
agents can still misidentify the destructive postgres boundary if any
live runtime wording lagged the named volume.

## Current State

- **Repository:** `/Users/tom/Dev/projects/underlay-reference`
- **Planning branch:** `main`
- **Planning base commit:** `0109b906272c7ea39e5e84bb4034ff08d0043f48`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved
  to that SHA before this handoff was created.
- **Planning checkout:** clean before this handoff file was created.
- **Worker mode:** implementation worker dispatched by the orchestrator.
- **Worker branch:** `t3code/papercuts-wave5-docs-closeout`
- **Worker worktree:** `/Users/tom/.t3/worktrees/underlay-reference/t3code-3b5034b1`
  (launcher worktree). `.agents.local.env` absent; no manual fallback created.
- **Ready work items, in order:**
  1. Doctor rejects built-in `docs` steps as unresolved task references
     — **closed.** Effigy `v0.12.1+local.834a4bd` doctor run emits no
     task-reference findings for `docs/effigy.toml` `docs check ...`
     steps. No migration-only workaround was present.
  2. Active runtime docs vs named postgres volume — **closed.** Canonical
     live wording is README: Postgres persists in
     `underlay-reference-dev-postgres-data`; older `.effigy/runtime/data/`
     bind-mounts are not migrated. No other active usage/runtime guide
     still claims the bind-mount path.
- **Out of scope:** T3 launching an Underlay worktree for this handoff;
  rewriting historical `docs/handoffs/` or `docs/logs/` entries;
  editing Underlay or Effigy.
- **Canonical refs:** `PAPERCUTS.md`; `docs/effigy.toml`; README runtime
  notes around the named volumes; sibling Effigy closed Doctor `docs`
  built-in (2026-08-27).
- **Required validation:** `effigy doctor` on this checkout against
  Effigy `v0.12.1+local.834a4bd`. Remaining doctor errors are unrelated
  (vault/health TTY, unsupported keys, scan markers/god-files).
- **PR URL:** pending
- **Merge authorisation:** absent; do not merge

## Boundaries

- Prove against the current pin. Do not add a Doctor workaround that
  hides a still-broken built-in resolver.
- Do not merge.

## Important Context

- README already says Postgres persists in
  `underlay-reference-dev-postgres-data` and that older
  `.effigy/runtime/data/` bind-mounts are not migrated.
- **Report to:** the operator.

## Suggested Next Move

Read this file, run the worktree preflight, then run `effigy doctor` and
hunt remaining active runtime wording.

## Completion Protocol

### Before you start

1. Read this handoff. Run `git rev-parse --show-toplevel`,
   `git branch --show-current`, `git status --porcelain`, and
   `git worktree list --porcelain`.
2. Accept a clean dedicated non-`main` registered worktree. Record the
   actual path/branch.
3. Confirm `HEAD == origin/main` and ancestor
   `0109b906272c7ea39e5e84bb4034ff08d0043f48`.
4. Confirm this handoff exists in `HEAD`.

### When the assigned runway is complete

1. Update `PAPERCUTS.md`. Push a PR. Do not merge.

### Review and merge path

Awaiting orchestrator review. Merge is operator-authorised only.

- **Closeout refs:** `PAPERCUTS.md`; this handoff; the PR.

### Handoff closeout

If Doctor is already clean on this SHA, close with that evidence. Leave
the T3 wrong-repo launch papercut open.
