# 002 - Northstar Instruction And Language Quality Audit

Status: complete (merged as PR 13)
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

- `002-northstar-agents-rust-typescript-audit.md` — complete and merged as PR 13
- `001-audit-retained-acme-admin-underlay-surface.md` — paused, unchanged;
  resumes after the operator-prioritized `g01.013` rollout

## Authority Boundary

Audit recorders authorize only finding-first repairs. They do not authorize
Underlay/Poodle changes, dependency or MSRV policy, product/runtime contract
changes, or the retained-surface classification owned by `g01.007`.

## Next Task

Audit delivery merged as PR 13. The dependent `g01.013` and `g01.007` lanes
later completed in PRs 14 and 15.
