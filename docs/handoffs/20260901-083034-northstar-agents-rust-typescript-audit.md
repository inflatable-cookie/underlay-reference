---
title: Underlay Reference Northstar instruction and language audit
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: complete-merged-pr-13
owner: Tom / Northstar orchestrator
created: 2026-09-01
updated: 2026-09-01
handoff_path: /Users/tom/Dev/projects/underlay-reference/docs/handoffs/20260901-083034-northstar-agents-rust-typescript-audit.md
base_required: pushed-main
tags: [coordination, handoff, worker, audit, agents, rust, typescript]
---

## What This Thread Was Doing

The operator requested a current repository-scope Northstar audit of Underlay
Reference's nested instruction journey, eight-crate Acme API, and four
TypeScript/Svelte packages. This worker owns `g01.012` and card 002 only.
`g01.007` and card 001 are paused because they overlap Acme Admin.

## Current State

- **Repository:** `/Users/tom/Dev/projects/underlay-reference`
- **Planning base:** `46a54a4e` on pushed `main`
- **Worker branch:** `worker/northstar-agents-rust-typescript-audit`
- **Worker workspace:** Paseo-managed worktree; record its actual path
- **Required sibling links:** `underlay` from
  `/Users/tom/Dev/projects/underlay` and `poodle` from
  `/Users/tom/Dev/projects/poodle`, each beside the worker worktree
- **Authority:** `g01.012`, spec 002, card 002, root/nested `AGENTS.md`, working
  rules, installed Northstar
- **Northstar source:**
  `/Users/tom/Dev/projects/northstar/skills/northstar` at
  `dbce3856be6ec6093d2e5c071568a6dbe953df49` or later
- **PR base/head:** `main` <- `worker/northstar-agents-rust-typescript-audit`
- **Merge path:** orchestrator after exact-head review and passing checks

## Assignment

Run the complete card, not a sample:

1. Apply the Northstar AGENTS review to root, docs, every app, and every package
   instruction scope plus any Claude bridge.
2. Run the explicit Northstar Rust audit across all eight Acme API crates,
   targets, and features.
3. Run the explicit Northstar TypeScript/Svelte audit across Acme Admin, Acme
   Front, Acme Client, and Acme UI with package-aware overlays.
4. Record findings before mutation and apply only recorder-authorized repairs.
5. Reconcile recorders, card, roadmap, spec, front doors, log, limitations, and
   PR body at closeout.

The Rust workspace declares no fixed MSRV. Record the current toolchain without
inventing a minimum. Record the exact Northstar source hash and do not mix tool
versions.

## Boundaries

- Do not edit Underlay, Poodle, dependencies, migrations, runtime topology, CI,
  workflows, deployment, releases, or product behavior.
- Do not classify or close the retained Underlay surface owned by paused
  `g01.007`/card 001. Resume state is a planning closeout fact, not audit
  authority.
- Stop for public API, security-contract, persistence, dependency/version,
  retained-surface, runtime, CI, deployment, or release decisions.
- Never edit the planning checkout. Never merge the PR.

## Preflight

1. Read this tracked handoff, every applicable `AGENTS.md`, `g01.012`, spec 002,
   and card 002.
2. Confirm `HEAD == origin/main`, planning base `46a54a4e` is an ancestor, the
   worktree is clean, and branch/worktree match this lane.
3. Verify `underlay` and `poodle` sibling symlinks before broad commands. Stop
   on missing sources, occupied destinations, or mismatches; never overwrite or
   delete shared links.
4. Load the Northstar router and explicit AGENTS, Rust, and TypeScript/Svelte
   audit modes from the source above. Use the repository-local Effigy skill and
   inventory tasks before selecting validation.

## Proof And PR

Meet card 002's inventories, finalized recorder reports, changed-file
attribution, focused tests, repository QA, and adversarial review oracle. Open
one PR to `main`; report its URL and exact head. Do not merge or run card 001.

If review requests changes, remain on this branch. The orchestrator will wake
this same worker; repair only posted in-bounds findings and report a new head.
