---
title: g09.053 Underlay Reference cross-tab CSRF stability worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom
created: 2026-08-27
updated: 2026-08-27
handoff_path: /Users/tom/Dev/projects/underlay-reference/docs/handoffs/20260827-145501-g09-053-cross-tab-csrf-stability.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, g09, csrf, security, reference]
---

## What This Thread Was Doing

The Underlay orchestrator closed the five consumer runtime/access rollouts and
found one remaining Reference security defect before fleet closeout: every
`GET /v1/auth/csrf-token` mints a new token, so a second browser tab invalidates
the first tab's cached header.

The orchestrator repaired the numbered queue. `g09.053` now owns this bounded
Reference fix; the fleet closeout moved to `g09.054` and remains planned.

## Why It Matters

Underlay Reference is the copyable proof anchor. Its stateless double-submit
CSRF flow must work across normal same-origin tabs before the six-root fleet can
claim a stable cookie-mutation posture.

## Current State

- **Repository:** `git@github.com:inflatable-cookie/underlay-reference.git`
- **Planning branch:** `main`
- **Planning base commit:** `6af2783768e04c8def9b6bb1de5c90cbb69a7892`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved to
  `6af2783768e04c8def9b6bb1de5c90cbb69a7892` after a fresh fetch on
  2026-08-27.
- **Planning checkout:** clean and aligned with `origin/main` when this handoff
  was written.
- **Worker mode:** implementation worker dispatched by the orchestrator; this
  handoff activates the worker-only worktree preflight.
- **Planning artifacts included at the base:** merged g09.047 Reference proof.
  External Underlay roadmap authority is pushed at
  `e90493c51304a02f8a93a0bcee7347e7df768d74`.
- **Worker branch:** `worker/g09-053-reference-cross-tab-csrf` is the fallback
  name only. Accept a launcher-supplied clean non-`main` branch.
- **Worker worktree:** launcher-provided clean registered worktree; record its
  actual path during preflight. No manual fallback path has been authorised or
  guessed.
- **Worktree creation command:** launcher-owned by default. If the current and
  named contexts are unusable, read `.agents.local.env`, require
  `AGENTS_WORKTREE_CONTAINER_DIR`, ask the operator if it is absent, then use
  `git worktree add -b <unique-branch> <AGENTS_WORKTREE_CONTAINER_DIR>/<unique-name> origin/main`.
- **Worker worktree policy:** use a clean, dedicated, non-`main` registered
  worktree supplied by the launcher even when its path or branch differs from
  this fallback. Never create a second worktree for a name mismatch.
- **Active target spec lane:** target `g01.007` retained-admin work is
  independent authority and must remain untouched.
- **Roadmap milestone:** external Underlay
  `/Users/tom/Dev/projects/underlay/docs/roadmaps/g09/053-underlay-reference-cross-tab-csrf-stability.md`.
- **Ready cards, in order:** none. `g09.053` is one bounded runnable roadmap,
  not a batch-card substitute.
- **Allowed runway:** implement and prove `g09.053` in one reviewable PR.
- **Remaining card budget:** one roadmap; stop after the PR is ready.
- **Dispatch topology:** serial in Underlay Reference.
- **Parallel safety check:** the target's `g01.007` lane can touch shared docs
  and locks. Do not run it concurrently with this security repair.
- **Canonical refs:** target `AGENTS.md`, `apps/acme-api/AGENTS.md`,
  `docs/architecture/000-overview.md`, `docs/architecture/product-guardrails.md`,
  and `docs/policy/001-working-rules.md`; Underlay contracts
  `docs/contracts/025-rust-app-runtime-assembly-and-router-topology.md`,
  `docs/contracts/026-route-families-and-access-model.md`, and
  `docs/contracts/030-auth-and-session-systems.md`.
- **Model capability profile:** frontier coding model with high reasoning; this
  is authentication-adjacent policy and regression proof.
- **Tool/runtime restrictions:** use the repo-local Effigy skill and selectors
  first. Do not edit workflows, publish a release, touch another consumer,
  modify Underlay planning, or change the target `g01.007` lane.
- **Required validation:** `effigy tasks`; focused issuance/two-tab tests;
  `effigy acme-api/health`; `effigy test --plan`; `effigy validate`;
  `effigy qa`; `effigy acme-docs/qa:docs`;
  `effigy acme-docs/qa:northstar`; `git diff --check`.
- **PR base/head:** `main` / worker branch.
- **PR URL:** pending.
- **Review state:** awaiting implementation and review.
- **Merge authorisation:** none. The worker must not merge.

## Boundaries

- **In scope:** the CSRF issuance code in
  `apps/acme-api/crates/api/src/routes/shared/auth/mod.rs` or a focused sibling
  module; request-cookie inspection; a small app-owned issuance helper if
  useful; focused production-path and middleware regression tests; one target
  execution log.
- **Out of scope:** target `g01.007`; client API changes; public route or
  envelope changes; server-side CSRF persistence; session/database schema;
  general auth refactors; other consumers; Underlay source/planning; Poodle;
  workflows; releases; unrelated formatting or Doctor cleanup.
- Reuse a non-empty incoming CSRF cookie. Mint only when it is absent or empty.
  Keep the response body and emitted cookie on the same token.
- Keep the double-submit model stateless. Do not add session coupling, storage,
  expiry state, or a new rotation protocol.
- Prove tab A fetches token A, tab B reads with the same browser cookie and
  receives token A, then tab A's original cookie/header pair still passes the
  real CSRF middleware on a cookie-backed mutation.
- Preserve cookie attributes, `GET /v1/auth/csrf-token`, and the
  `SingleResponse<CsrfTokenResponse>` wire shape.
- Do not invent architecture, widen the roadmap, or reopen settled policy.
- Work only in the selected worker worktree. Never edit the orchestrator's
  planning checkout or an unrelated dirty checkout.
- Do not merge. Merge remains a separate operator-authorised action.

## Important Context

- **Planning lineage:** g09.047 shipped the Reference runtime/access proof.
  g09.048-g09.052 then merged across the five consumers. Acowtancy exact-head
  review exposed the same cross-tab failure and proved the bounded reuse policy.
  `g09.053` now repairs the Reference owner; `g09.054` closes the fleet.
- **Why this roadmap is ready:** current target behavior is exact and visible;
  the stateless policy, compatibility boundary, failure mode, acceptance, and
  validation are settled; target main is clean; no open PR exists.
- **Current implementation:** `csrf_token` always calls `Uuid::new_v7()` before
  setting the CSRF cookie and returning the body. `extract_csrf_token` already
  parses a non-empty cookie and is used by the middleware.
- **Reference evidence:** Acowtancy's merged Farmyard implementation factors
  `csrf_token_to_issue(headers, config)` and includes a two-tab regression. It
  is evidence for the policy, not authority to copy unrelated app structure.
- **Test quality:** exercise the same issuance decision used by production and
  the actual CSRF middleware. Do not close the card with a helper-only assertion
  or a synthetic test that cannot fail when the production handler regresses.
- **Open tensions:** none inside scope. If a full production handler test needs
  a new broad `AppState` seam, stop and report before widening; a focused state
  or shared production helper is preferable.
- **Report after:** the implementation plus focused regression are complete and
  bounded validation has run.
- **Report to:** the operator, who will relay progress to the orchestrator.

## Suggested Next Move

This handoff activates worker mode. Read it from the top, then run the quick
worktree preflight below before broad repository reads. Use the launcher's clean
non-`main` worktree when supplied. Read the named roadmap and refs, inspect the
exact production handler/middleware seam, implement the smallest production
path that satisfies the two-tab proof, and report one meaningful chunk.

## Completion Protocol

### Before you start

1. Read this handoff path. Its `worker_mode: implementation` and
   `dispatch_authority: orchestrator` metadata activate worker mode. Before any
   broad read, run `git rev-parse --show-toplevel`,
   `git branch --show-current`, `git status --porcelain`, and
   `git worktree list --porcelain`.
2. If the current root is a registered worktree, status is empty, and branch is
   not `main`, accept it as the launcher-provided worktree. Record its actual
   root and branch; do not compare names or create another worktree.
3. If the launcher supplied `main` or a dirty worktree, stop and report it. Only
   a normal manual launch may inspect a matching named worktree, then
   `.agents.local.env`; require `AGENTS_WORKTREE_CONTAINER_DIR`, ask the operator
   if absent, and create a unique fallback under that container. Never use
   `/tmp`, `TMPDIR`, or a guessed path. Never clean, reset, stash over, or
   discard an existing checkout.
4. From the selected worktree, confirm `HEAD == origin/main`, confirm
   `git merge-base --is-ancestor 6af2783768e04c8def9b6bb1de5c90cbb69a7892 HEAD`,
   and confirm this handoff exists in `HEAD`.
5. Read `AGENTS.md`, `apps/acme-api/AGENTS.md`, the external `g09.053` roadmap,
   and the named canonical refs.
6. Run `effigy tasks`; inspect `effigy test --plan` before selecting the focused
   test runner.

### While you work

- Execute only `g09.053`. Keep commits aligned with the implementation/proof
  chunk, not model turns.
- Prefer Effigy selectors. Use a focused raw Cargo test only when the task
  surface cannot select the named regression; record that fallback.
- Report changed files, validation run, remaining work, and blockers after the
  coherent implementation/test chunk.
- Stop if scope expands, authority is missing, a contract conflicts, or
  validation changes the plan. Do not invent a new architecture.

### When the assigned runway is complete

1. Run the required validation: focused issuance/two-tab tests;
   `effigy acme-api/health`; `effigy test --plan`; `effigy validate`;
   `effigy qa`; `effigy acme-docs/qa:docs`;
   `effigy acme-docs/qa:northstar`; `git diff --check`.
2. Add one target execution log recording exact behavior, changed files,
   validation, worktree/branch, and any retained limitation. Do not change the
   target `g01.007` card, roadmap, or front doors.
3. Push the worker branch.
4. Open a reviewable PR against current pushed `main`. The planning base above
   predates this handoff commit and is intentionally not self-referential.
5. In the PR body, link this handoff, external `g09.053`, changed surfaces,
   evidence, validation, and unresolved items.
6. Report the PR URL and evidence to the operator. Do not merge.

### Review and merge path

The orchestrator will review metadata, commits, diff, checks, and exact-head
validation against `g09.053`. Because orchestrator and worker share a GitHub
identity, the verdict will be a canonical PR comment rather than formal
self-approval. Current state: awaiting implementation. Requested changes: none.
The operator must explicitly authorise any merge.

- **Closeout refs:** external Underlay `g09.053`; target execution log; after
  reviewed merge, Underlay `g09.054` promotion gate and front doors.

### Handoff closeout

Leave the target execution evidence honest. Stop on blockers rather than
marking the runway complete. The orchestrator owns the external roadmap and
fleet-closeout updates after review and merge.
