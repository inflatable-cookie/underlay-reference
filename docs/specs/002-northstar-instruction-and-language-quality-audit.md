# 002 - Northstar Instruction And Language Quality Audit

Status: complete (audit PR open, awaiting orchestrator review)
Owner: repo maintainers
Created: 2026-09-01
Roadmap refs: `g01.012`
Governing refs: root and nested `AGENTS.md`,
`docs/policy/001-working-rules.md`, installed Northstar

## Objective

Run one finding-first repository audit across the reference implementation's
instruction, Rust, and TypeScript/Svelte surfaces without absorbing the paused
retained-surface decision lane.

## Ready Chain

- `002-northstar-agents-rust-typescript-audit.md` — complete; PR open at
  `main` <- `worker/northstar-agents-rust-typescript-audit`
- `001-audit-retained-acme-admin-underlay-surface.md` — paused, unchanged;
  resumes after the audit PR merges

## Authority Boundary

Audit recorders authorize only finding-first repairs. They do not authorize
Underlay/Poodle changes, dependency or MSRV policy, product/runtime contract
changes, or the retained-surface classification owned by `g01.007`.

## Next Task

Orchestrator: review the card 002 PR at its exact head and merge. Then resume
`g01.007` and card 001. The retained-surface classification is still open —
this audit neither advanced nor absorbed it.
