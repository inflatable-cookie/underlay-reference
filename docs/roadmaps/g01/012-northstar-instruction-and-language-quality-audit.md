# g01.012 Northstar Instruction And Language Quality Audit

Status: complete (merged as PR 13)
Owner: repo maintainers
Created: 2026-09-01
Governing refs: root and nested `AGENTS.md`,
`docs/policy/001-working-rules.md`,
`docs/specs/002-northstar-instruction-and-language-quality-audit.md`, installed
Northstar
Planning state: card 002 complete and merged; `g01.007` stays paused during `g01.013`

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

## Outcome

Card 002 ran in the isolated worker `worker/northstar-agents-rust-typescript-audit`
against Northstar `dbce3856be6ec6093d2e5c071568a6dbe953df49`.

| Acceptance criterion | Result |
| --- | --- |
| Every root/nested instruction section has a human disposition and scope precedence remains clear | Met. Seven instruction scopes plus the Claude bridge carry a section-intent map and a disposition; root now scopes its "Keep AGENTS Lean" rule to the nested files it governs, and nested files point at root rather than restating it. |
| Rust scope covers all eight crates, targets, features, public APIs, unsafe/FFI, async/concurrency, panic/error paths, and exact forwarders | Met. 9 units over 8 crates and 17 target source paths, 0 declared features; unsafe/FFI verified absent workspace-wide; async and panic paths assessed per unit; a total exact-forwarder ledger is recorded in the closeout because the recorder rejects the evaluation-only rule. |
| Rust evidence records the actual current toolchain without claiming a fixed MSRV | Met. `rustc 1.97.1` recorded as observed; `RUST-MSRV-001` is `degraded` for all nine units and the missing policy stays a limitation. |
| TypeScript/Svelte scope covers Acme Admin, Acme Front, Acme Client, and Acme UI with package overlays and explicit generated/vendor exclusions | Met. 201 files, 8 units, overlays resolved per package from real version evidence, zero unregistered candidates, exclusions named. |
| Every changed source file maps to a prior finding and authorized plan | Met. 10 source files, each attributed in the closeout. |
| Finalized recorders, changed-file attribution, limitations, roadmap, spec, card, log, and front doors agree | Met. |
| Repository validation records actual results and `g01.007` returns to its paused pre-audit state after closeout | Met. Validation is recorded including the one red result — a pre-existing failure in the sibling Underlay repository — and `g01.007` was never entered. |

Closeout: `docs/logs/2026-09/01-091500-g01-012-northstar-agents-rust-typescript-audit.md`

## Next Task

Audit delivery merged as PR 13. Execute `g01.013` card 003, then resume
`g01.007`; its retained-surface classification remains open.
