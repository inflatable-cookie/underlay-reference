---
title: Papercuts wave 2 vitest race worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-08-27
updated: 2026-08-27
handoff_path: /Users/tom/Dev/projects/underlay-reference/docs/handoffs/20260827-181300-papercuts-wave2-vitest-race.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts]
---

## What This Thread Was Doing

Wave 1 pinned acme-client to package-local TypeScript. Root `effigy
validate`/`qa` still races two `bun x vitest` processes on bin linking.

You are the Underlay Reference implementation worker for this one-item
lane. Doctor built-in `docs` belongs to Effigy wave 1 (already merged);
do not re-fix it here unless this checkout still shows it after current
Effigy.

## Why It Matters

The aggregate board fails even when each suite is green in isolation, so
a CSRF-only API change cannot close validate/qa on the first pass.

## Current State

- **Repository:** `/Users/tom/Dev/projects/underlay-reference`
- **Planning branch:** `main`
- **Planning base commit:** `10e8636908b9a11f9bdd70e24bf6f2194671b500`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved
  to that SHA before this handoff was created.
- **Planning checkout:** clean before this handoff file was created.
- **Worker mode:** implementation worker dispatched by the orchestrator.
- **Planning artifacts included at the base:** `PAPERCUTS.md`; this handoff.
- **Worker branch:** `worker/papercuts-wave2-vitest-race`
- **Worker worktree:** use the launcher worktree. This handoff does not
  select a manual fallback path.
- **Manual fallback command:** only after the operator supplies
  `AGENTS_WORKTREE_CONTAINER_DIR`. `.agents.local.env` was absent.
- **Active spec lane:** none.
- **Roadmap milestone:** none.
- **Ready work items, in order:**
  1. Parallel `bun x vitest` in `effigy test` races on bun bin linking
- **Allowed runway:** that one item only, one PR.
- **Remaining card budget:** one papercut.
- **Dispatch topology:** serial inside this repo; parallel with other
  wave-2 repos.
- **Parallel safety check:** no shared files with other wave-2 workers.
  Do not edit Underlay or Effigy unless the only honest fix is an Effigy
  test-board serialize — in that case stop and report.
- **Canonical refs:** `AGENTS.md`; `PAPERCUTS.md`; root `effigy.toml` /
  `[test.suites]`; acme-admin and acme-client test tasks.
- **Model capability profile:** capable coding model, medium reasoning.
- **Tool/runtime restrictions:** use Effigy selectors.
- **Required validation:** root `effigy test` / `validate` can run
  acme-admin and acme-client vitest without `EEXIST` on bun bin linking.
  Isolated suite runs stay green.
- **PR base/head:** current pushed `main` / selected worker branch
- **PR URL:** pending
- **Review state:** awaiting orchestrator review after worker completion
- **Merge authorisation:** absent; do not merge

## Boundaries

- **In scope:** serialize `bun x` on this test board, or run the
  workspace-local vitest binary instead of `bun x`. Close the papercut.
- **Out of scope:** T3 launching the Underlay worktree; Doctor `docs`
  built-in (Effigy); CSRF / runtime-access roadmaps.
- Preferred fix: workspace-local `vitest` binary. Serializing the two
  suites is acceptable if the binary is not already installed.
- Do not merge the PR.

## Important Context

- **Planning lineage:** papercuts wave 2 after Underlay Reference PR 7.
- **Report after:** the race proof and a green aggregate test; then PR.
- **Report to:** the operator, who will relay progress to the orchestrator.

## Suggested Next Move

Read this file from the top. Run the worktree-safety preflight. Use the
launcher worktree if it is clean, dedicated, and not `main`.

Reproduce the parallel `bun x vitest` EEXIST, then switch those tasks off
`bun x`.

## Completion Protocol

### Before you start

1. Read this handoff. Then run `git rev-parse --show-toplevel`,
   `git branch --show-current`, `git status --porcelain`, and
   `git worktree list --porcelain`.
2. If the current root is a registered worktree, status is empty, and the
   branch is not `main`, accept it.
3. If the launcher supplied a dirty or `main` worktree, stop and report
   it. `.agents.local.env` was absent, so ask before creating a fallback.
   Never use `/tmp`.
4. Confirm `HEAD == origin/main`, confirm
   `git merge-base --is-ancestor 10e8636908b9a11f9bdd70e24bf6f2194671b500 HEAD`,
   and confirm this handoff exists in `HEAD`.
5. Read `AGENTS.md` and `PAPERCUTS.md`.

### While you work

- Keep the diff in test-task wiring.

### When the assigned runway is complete

1. Run the aggregate test/validate path that used to race.
2. Close the papercut in `PAPERCUTS.md`.
3. Push the worker branch and open a PR against current pushed `main`.
4. Report the PR URL. Do not merge.

### Review and merge path

Awaiting orchestrator review. Merge is operator-authorised only.

- **Closeout refs:** `PAPERCUTS.md`; this handoff; the PR.

### Handoff closeout

If the only honest fix is an Effigy test-board change, stop and report.
Do not start an Effigy worker from this repo.
