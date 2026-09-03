# 002 - Northstar AGENTS, Rust, And TypeScript Audit

Status: complete (merged as PR 13)
Owner: repo maintainers
Created: 2026-09-01
Roadmap: `g01.012`
Spec: `docs/specs/archive/002-northstar-instruction-and-language-quality-audit.md`
Auto-start next card: no

## Objective

Run one repository-scope Northstar instruction, Rust, and TypeScript/Svelte
explicit audit and return the smallest authorized repair set as one PR.

## Scope

- root `AGENTS.md`, `docs/AGENTS.md`, all app/package `AGENTS.md` files, and any
  Claude bridge;
- all eight Acme API workspace crates and their targets/features;
- Acme Admin, Acme Front, Acme Client, and Acme UI hand-written
  TypeScript/Svelte source with package-aware overlays;
- managed Northstar audit setup, recorders, focused repairs, evidence, and
  closeout surfaces.

Generated/vendor/cache material, siblings, dependencies, retained-surface
classification, product behavior, migrations, runtime topology, CI, deployment,
workflows, and releases are out of scope.

## Ordered Work

1. Capture clean Git state, nested instruction map, Cargo/package/target/
   feature inventory, JS/Svelte unit map, current toolchains, and exclusions.
2. Record instruction dispositions. Initialize both language recorders and
   freeze disjoint units before source mutation.
3. Assess correctness, architecture, and human quality for every unit. Record
   findings and exact-forwarder candidates before repair plans.
4. Apply only recorder-authorized repairs; extend scope before touching a
   caller, test, contract, or doc outside an owned unit.
5. Record current Rust toolchain evidence without inventing an MSRV.
6. Finalize recorders, run repository-native validation, falsify against
   `g01.012`, close docs/evidence, push, and open one PR.

## Acceptance Criteria

- [x] every nested instruction scope, crate/target/feature, and hand-written
      TypeScript/Svelte unit is owned or explicitly excluded — 7 instruction
      scopes plus the Claude bridge; 8 crates, 17 target source paths, 0
      declared features across 9 Rust units; 201 files across 8 TypeScript
      units with `icons.generated.ts` and untracked build output excluded;
- [x] every normative rule and required assessment pass has a verdict per unit
      — 54 Rust verdicts (6 approved rules x 9 units) with 27 dimension
      attestations; `RUST-SLOP-001` is evaluation-only and carries a candidate
      ledger instead, which the closeout records;
- [x] source edits map only to pre-recorded authorized findings — 10 source
      files changed, each attributed in the closeout's changed-file table;
- [x] generated/vendor/sibling files and retained-surface authority survive —
      `icons.generated.ts` untouched, both sibling checkouts unwritten, no file
      under `g01.007`/card 001 authority touched;
- [x] missing fixed MSRV, warnings, unavailable services, and retained findings
      remain honest limitations — 10 limitations recorded, including the
      unresolved MSRV, the database-less test run, and the pre-existing sibling
      Underlay `effigy validate` failure;
- [x] repository QA, docs/Northstar QA, package validation, focused tests, and
      `git diff --check` record actual results — see the closeout's validation
      table;
- [x] one PR targets `main`; the worker does not merge or execute card 001.

## Review Oracle

Use `g01.012`. Reconcile both recorder changed-file unions with Git, then
sample auth/session/error Rust boundaries, typed client seams, and one
state/rendering path in each Svelte app. Try missing nested scope,
current-toolchain-as-MSRV, warning-as-pass, generated repair, sibling mutation,
and retained-lane absorption counterexamples first.

## Evidence Required

- instruction before/after measurements and section dispositions;
- finalized Rust and TypeScript/Svelte recorder reports and hashes;
- inventories, exclusions, changed-file attribution, focused proof, and QA;
- closeout under `docs/logs/2026-09/` and exact PR head.

## Stop Conditions

Use `g01.012` and spec 002 stop conditions. Stop if a profile, scope, version,
public-contract, or retained-surface decision cannot be represented honestly.

## Outcome

Closeout: `docs/logs/2026-09/01-091500-g01-012-northstar-agents-rust-typescript-audit.md`

Northstar source pinned at `dbce3856be6ec6093d2e5c071568a6dbe953df49`. Both
language recorders were initialized before any source mutation and finalized
after: Rust `result.json`
`fd1d731ee91d735be61121d6e7ed4ec247ee0b952f699b716d40b14970fead27`, TypeScript
`result.json` `e4d139428c542341f5ea7892c4a61d54f0161721a2914db544fc4bb9f5e7ebf7`.

Six instruction findings (five repaired, one reported), 22 Rust findings (four
repaired across four files, one operator decision, the rest retained), and 28
TypeScript/Svelte findings (four repaired across six files, the rest retained).

## Next Task

Merged as PR 13. The dependent `g01.013` and `g01.007` lanes later completed
in PRs 14 and 15.
