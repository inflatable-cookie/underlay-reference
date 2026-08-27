---
title: g09.059 Underlay Reference task batch-delete colon worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / Underlay orchestrator
created: 2026-08-27
updated: 2026-08-27
handoff_path: /Users/tom/Dev/projects/underlay-reference/docs/handoffs/20260827-181956-g09-059-task-batch-delete-colon.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, g09, batch-delete, routes]
---

## What This Thread Was Doing

Implement external Underlay roadmap `g09.059` as one bounded Underlay Reference
PR. Move nested admin task batch deletion from slash grammar to the app's
existing canonical colon grammar.

This worker owns the API route, Acme Client caller, focused route/client proof,
and current route documentation for that one action. Do not advance or
reinterpret Underlay Reference's independent planning queue.

## Why It Matters

Categories, projects, and media already use `:batch-delete`; nested tasks alone
use `/batch-delete`. One API should expose one collection-action grammar.

The operator declared the supported caller set closed-world, chose
`:batch-delete`, and authorised an atomic cutover with no compatibility window.

## Current State

- **Repository:** `git@github.com:inflatable-cookie/underlay-reference.git`
- **Planning branch:** `main`
- **Planning base commit:** `2e67d297a74c469d5495da587d9f2913367e5ef2`
- **Pushed main verification:** local `HEAD` and `origin/main` matched that SHA
  immediately before this handoff was created
- **Planning checkout:** clean
- **Worker mode:** implementation worker dispatched by the orchestrator
- **External authority:** Underlay commit
  `872adb2205c3c400ceb9cadee361a9c1eb5421f6`; roadmap
  `docs/roadmaps/g09/059-batch-delete-action-grammar-convergence.md`
- **Worker branch:** `worker/g09-059-task-batch-delete-colon`
- **Worker worktree:** launcher-provided; accept its actual clean registered
  non-`main` path and branch
- **Manual fallback:** only through operator-configured
  `AGENTS_WORKTREE_CONTAINER_DIR`; never use `/tmp` or guess a path
- **Active spec lane:** none
- **Ready work, in order:** API route, client caller, route/client proof,
  current docs/inventory, one target execution log, one PR
- **Allowed runway:** nested admin task batch-delete grammar only
- **Remaining card budget:** one external roadmap lane
- **Dispatch topology:** parallel with the other four `g09.058`/`g09.059`
  target lanes
- **Parallel safety check:** main also carries a papercuts Vitest-race handoff;
  its task-runner wiring is disjoint. Stop on a new PR or worktree edit touching
  the task route, task command, or their tests
- **Canonical refs:** root, API, client, and docs `AGENTS.md`; this handoff;
  external Underlay contracts `027` and `029`; external roadmap `g09.059`
- **Tool/runtime restrictions:** Effigy-first; use `effigy test --plan` before
  choosing focused raw test commands
- **Required validation:** task inventory and test plan; API/client focused
  tests; `acme-api/check`; `acme-client/check`; `acme-docs/qa:docs`;
  `acme-docs/qa:northstar`; root validation once after the batch; `git diff
  --check`
- **PR base/head:** current pushed `main` / selected worker branch
- **PR URL:** pending
- **Review state:** awaiting orchestrator review after worker completion
- **Merge authorisation:** absent; do not merge

## Boundaries

- **In scope:** change the route to
  `/v1/admin/projects/{project_id}/tasks:batch-delete`; update the Acme Client
  command and focused tests; keep OpenAPI comments/current route inventory
  accurate; prove the old slash path is absent; add one target log.
- **Out of scope:** other task actions, category/project/media routes, payloads,
  handlers, envelopes, auth/CSRF/role policy, database behavior, UI redesign,
  dependencies, task-runner repair, Underlay source, release, deploy, or target
  roadmap sequencing.
- Do not retain a same-handler slash alias. The authorised compatibility window
  is none.
- Preserve `POST`, request/response types, audit behavior, and handler semantics.
- Never clean, reset, stash over, or edit another checkout. Do not merge the PR.

## Important Context

- Current server route:
  `apps/acme-api/crates/api/src/routes/admin/router.rs` mounts
  `/v1/admin/projects/{project_id}/tasks/batch-delete`.
- `apps/acme-api/crates/api/src/routes/admin/tasks.rs` already documents the
  colon form, so align assembly to that stated contract.
- Current client caller:
  `packages/acme-client/src/commands/admin/task-commands.ts`.
- Current client proof:
  `packages/acme-client/tests/commands/admin/task-commands.test.ts`.
- Historical logs and completed roadmaps may retain the old path as evidence.
  Do not rewrite archival records.
- The papercuts worker may change Effigy/Vitest task wiring. Rebase if it merges;
  do not absorb that repair into this PR.

## Suggested Next Move

Read this file from the top, complete worktree preflight, revalidate the current
caller inventory, then make the server/client/test change atomically.

## Completion Protocol

### Before you start

1. Run `git rev-parse --show-toplevel`, `git branch --show-current`, `git
   status --porcelain`, and `git worktree list --porcelain` before broad reads.
2. Use the clean registered launcher-supplied non-`main` worktree. If it is
   dirty or on `main`, stop. Only an operator-configured fallback is allowed;
   never use `/tmp` or guess a path.
3. Fetch and require `HEAD == origin/main`; require
   `2e67d297a74c469d5495da587d9f2913367e5ef2` is an ancestor; require this
   handoff exists in `HEAD`.
4. Read the root, `apps/acme-api`, `packages/acme-client`, and docs
   `AGENTS.md`; this handoff; external Underlay `g09.059`; contracts `027` and
   `029`; and current relevant source/tests.
5. Run `effigy tasks` and `effigy test --plan`. Recheck open PRs and worktrees
   for route/client overlap.

### While you work

- Keep the cutover atomic and colon-only.
- Add positive proof for the new path and negative proof for the old path.
- Stop if current source exposes a caller outside the supported repository or
  if the two paths would not be semantically identical.
- Do not modify the independent papercuts handoff or target planning queue.

### When the assigned runway is complete

1. Run focused API and Acme Client tests chosen from the test plan.
2. Run `effigy acme-api/check`, `effigy acme-client/check`,
   `effigy acme-docs/qa:docs`, and `effigy acme-docs/qa:northstar`.
3. Run root `effigy validate` once. If the known parallel `bun x vitest` race
   is still present, record the exact failure and retain green isolated proof;
   do not edit task wiring.
4. Run `git diff --check`. Add one target log under `docs/logs/2026-08/` with
   scope, compatibility decision, validation, residual risk, and next task.
5. Push the worker branch and open one PR against current `main`. Report the PR
   URL and exact head. Do not merge or mark external `g09.059` complete.

### Review and merge path

Await exact-head orchestrator review. Apply bounded corrections in the same
worker thread. Merge only after explicit operator authorisation.

- **Closeout refs:** target execution log; external Underlay `g09.059`; this
  handoff; the PR

### Handoff closeout

Leave the target log and checkout honest. If blocked, record the exact reason
and stop rather than retaining an alias or widening the roadmap.
