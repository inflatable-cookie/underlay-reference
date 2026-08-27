---
title: Papercuts wave 1 pin acme-client TypeScript worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-08-27
updated: 2026-08-27
handoff_path: /Users/tom/Dev/projects/underlay-reference/docs/handoffs/20260827-160200-papercuts-wave1-pin-tsc.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts]
---

## What This Thread Was Doing

A papercuts sweep found `packages/acme-client/effigy.toml` running
`bun x tsc`, which fetches TypeScript 7.0.2 instead of the package-pinned
`typescript@^5.9.3`. TS 7 removes `baseUrl` and fails acme-client health.

The operator approved wave 1. You are the Underlay Reference implementation
worker for this one-item lane.

This is not g09.053 CSRF work, not a T3 launch-path fix, and not the
Effigy Doctor `docs` built-in fix (that lives in the Effigy wave-1 worker).

## Why It Matters

Root `effigy health` fails on an unrelated client typecheck. Workers
changing API migration/test surfaces cannot use health as a cheap gate.

## Current State

- **Repository:** `/Users/tom/Dev/projects/underlay-reference`
- **Planning branch:** `main`
- **Planning base commit:** `f89e3616a0906c044f14f3ddbeb20332a4dd480d`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved
  to that SHA before this handoff was created.
- **Planning checkout:** clean before this handoff file was created.
- **Worker mode:** implementation worker dispatched by the orchestrator; this
  handoff activates the worker-only worktree preflight.
- **Planning artifacts included at the base:** `PAPERCUTS.md`; this handoff.
- **Worker branch:** `worker/papercuts-wave1-pin-tsc`
- **Worker worktree:** use the clean dedicated non-`main` worktree supplied
  by the launcher. This handoff does not select a manual fallback path.
- **Manual fallback command:** only after the operator supplies an absolute
  `AGENTS_WORKTREE_CONTAINER_DIR`. `.agents.local.env` was absent in the
  planning checkout, so do not create a manual fallback without asking.
- **Active spec lane:** none for this papercuts lane.
- **Roadmap milestone:** none.
- **Ready work items, in order:**
  1. `bun x tsc` in acme-client health resolves TypeScript 7 and rejects
     `baseUrl`
- **Allowed runway:** that one item only, one PR.
- **Remaining card budget:** one papercut.
- **Dispatch topology:** serial inside this repo; parallel with other
  wave-1 repos.
- **Parallel safety check:** no shared files with other wave-1 workers.
  Do not edit Underlay itself.
- **Canonical refs:** `AGENTS.md`; `PAPERCUTS.md`;
  `packages/acme-client/effigy.toml`;
  `packages/acme-client/package.json`.
- **Model capability profile:** capable coding model, medium reasoning.
- **Tool/runtime restrictions:** use Effigy selectors; do not change
  TypeScript language level as a workaround.
- **Required validation:** acme-client `check`/`health` uses the pinned
  TypeScript 5.9.x compiler and accepts the current `baseUrl`. Prefer
  `effigy acme-client/health` or `bun run check` from that package.
- **PR base/head:** current pushed `main` / selected worker branch
- **PR URL:** pending
- **Review state:** awaiting orchestrator review after worker completion
- **Merge authorisation:** absent; do not merge

## Boundaries

- **In scope:** pin or route acme-client `build`/`check` to the
  package-local TypeScript. Close the papercut.
- **Out of scope:** T3 launching the Underlay worktree; Doctor built-in
  `docs` steps (Effigy worker); parallel `bun x vitest` EEXIST races;
  CSRF / runtime-access roadmaps.
- Preferred fix: `bun run check` / the package-pinned `tsc`. Pinning
  `bun x typescript@5.9.3 tsc` is acceptable if the package script is
  worse. Do not keep an unpinned `bun x tsc`.
- Apply the same routing to `build` and `check` in
  `packages/acme-client/effigy.toml` if both use `bun x tsc`.
- Do not merge the PR.

## Important Context

- **Planning lineage:** operator-authorized papercuts wave 1, 2026-08-27.
- **Why this item is ready:** one file, one compiler pin, no product
  decision.
- **Report after:** health/check proof; then PR.
- **Report to:** the operator, who will relay progress to the orchestrator.

## Suggested Next Move

Read this file from the top. Run the worktree-safety preflight. Use the
launcher worktree if it is clean, dedicated, and not `main`.

Then read `packages/acme-client/effigy.toml` and `package.json` and replace
`bun x tsc` with the pinned compiler.

## Completion Protocol

### Before you start

1. Read this handoff. Then run `git rev-parse --show-toplevel`,
   `git branch --show-current`, `git status --porcelain`, and
   `git worktree list --porcelain`.
2. If the current root is a registered worktree, status is empty, and the
   branch is not `main`, accept it and record the actual path/branch.
3. If the launcher supplied a dirty or `main` worktree, stop and report
   it. `.agents.local.env` was absent, so ask before creating a fallback.
   Never use `/tmp`.
4. Confirm `HEAD == origin/main`, confirm
   `git merge-base --is-ancestor f89e3616a0906c044f14f3ddbeb20332a4dd480d HEAD`,
   and confirm this handoff exists in `HEAD`.
5. Read `AGENTS.md`, `PAPERCUTS.md`, and the acme-client manifest.

### While you work

- Keep the diff small.
- Report through the operator after the pin and the health proof.

### When the assigned runway is complete

1. Run acme-client health/check.
2. Close the papercut in `PAPERCUTS.md`.
3. Push the worker branch and open a PR against current pushed `main`.
4. Report the PR URL. Do not merge.

### Review and merge path

Awaiting orchestrator review after the PR exists. Merge is
operator-authorised only.

- **Closeout refs:** `PAPERCUTS.md`; this handoff; the PR.

### Handoff closeout

If `package.json` already has a `check` script that uses the pinned tsc,
prefer wiring Effigy to that script instead of adding another pin.
