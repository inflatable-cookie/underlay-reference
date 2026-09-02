# g01.012 Northstar Instruction And Language Quality Audit

Status: paused-for-g01.013
Owner: repo maintainers
Created: 2026-09-01
Governing refs: root and nested `AGENTS.md`,
`docs/policy/001-working-rules.md`,
`docs/specs/002-northstar-instruction-and-language-quality-audit.md`, installed
Northstar
Planning state: card 002 preserved but paused during overlapping `g01.013` rollout

## Problem

The reference implementation's seven instruction scopes, eight-crate Acme API
workspace, and four TypeScript/Svelte packages have not had one current,
repository-scope Northstar explicit audit. The existing retained-surface audit
owns overlapping `acme-admin` code, so it is paused intact while this bounded
maintenance lane runs.

## Goals

- review root, docs, app, and package `AGENTS.md` files as one nested reader
  journey;
- audit every Acme API Rust crate/target/feature and every hand-written
  TypeScript/Svelte source surface;
- record findings before mutation and apply only recorder-authorized repairs;
- preserve released Underlay/Poodle dependency boundaries, generated/vendor
  output, runtime topology, migrations, workflows, and product behavior;
- finish with one reviewable PR and honest retained findings, then resume
  `g01.007` without pretending its retained-surface classification is complete.

## Non-Goals

- no retained-surface contract decision from `g01.007`;
- no Underlay or Poodle mutation, dependency upgrade/pin change, migration,
  deployment, workflow, release, or product feature;
- no invented Rust MSRV: the workspace declares no fixed minimum;
- no threshold-led splitting, blanket lint fixing, or compatibility shim.

## Acceptance Criteria

- every root/nested instruction section has a human disposition and scope
  precedence remains clear;
- Rust scope covers all eight crates, targets, features, public APIs,
  unsafe/FFI, async/concurrency, panic/error paths, and exact forwarders;
- Rust evidence records the actual current toolchain without claiming a fixed
  MSRV; missing minimum-version policy remains a limitation unless existing
  authority resolves it;
- TypeScript/Svelte scope covers Acme Admin, Acme Front, Acme Client, and Acme
  UI with package overlays and explicit generated/vendor exclusions;
- every changed source file maps to a prior finding and authorized plan;
- finalized recorders, changed-file attribution, limitations, roadmap, spec,
  card, log, and front doors agree;
- repository validation records actual results and `g01.007` returns to its
  paused pre-audit state after closeout.

## Review Oracle

| Invariant | Adversarial counterexample | Expected response | Required proof |
| --- | --- | --- | --- |
| Nested scope is complete. | One AGENTS overlay, crate target, or JS/Svelte package lacks a disposition. | Review blocks. | Scope inventories and finalized recorders. |
| Repairs are finding-first. | A source edit lacks a prior authorized finding and plan. | Reject or revert it. | Changed-file attribution. |
| Reference contracts survive. | Wire JSON, auth/session/error behavior, workspace shape, or runtime ownership changes under cleanup authority. | Stop for planning. | API/config diff and focused tests. |
| Version evidence is honest. | The current compiler is reported as an MSRV. | Result remains limited; no version claim. | Toolchain record and retained finding. |
| Siblings remain context-only. | The worker edits Underlay or Poodle. | Stop. | Changed-repository inventory. |
| Existing lane remains intact. | Audit closeout claims the retained-surface classification is complete or silently supersedes card 001. | Review blocks. | Planning-state diff. |

## Stop Conditions

- a repair needs a public API, security contract, persistence, migration,
  dependency/version policy, runtime, CI, deployment, workflow, release, or
  retained-surface decision;
- unit ownership overlaps or generated/foreign code cannot be isolated;
- the Northstar source changes during the audit;
- validation changes the plan or requires sibling mutation.

## Next Task

Resume card 002 after `g01.013` merges. Stop at its PR for orchestrator
exact-head review; do not execute card 001 concurrently.
