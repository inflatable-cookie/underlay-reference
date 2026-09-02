# 002 - Northstar AGENTS, Rust, And TypeScript Audit

Status: paused-for-g01.013
Owner: repo maintainers
Created: 2026-09-01
Roadmap: `g01.012`
Spec: `docs/specs/002-northstar-instruction-and-language-quality-audit.md`
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

- [ ] every nested instruction scope, crate/target/feature, and hand-written
      TypeScript/Svelte unit is owned or explicitly excluded;
- [ ] every normative rule and required assessment pass has a verdict per unit;
- [ ] source edits map only to pre-recorded authorized findings;
- [ ] generated/vendor/sibling files and retained-surface authority survive;
- [ ] missing fixed MSRV, warnings, unavailable services, and retained findings
      remain honest limitations;
- [ ] repository QA, docs/Northstar QA, package validation, focused tests, and
      `git diff --check` record actual results;
- [ ] one PR targets `main`; the worker does not merge or execute card 001.

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

## Next Task

Resume this card after `g01.013` merges and open a PR. Do not merge or start
card 001.
