# Agents Guide: Underlay Reference Implementation

## Purpose

This repository is a **reference template** for bootstrapping Underlay-based apps. Prefer canonical, reusable patterns over one-off customization.

## Keep AGENTS Lean

`AGENTS.md` files should contain only:

1. Scope and intent
2. Hard operational rules
3. Minimal validation commands
4. Links to detailed docs

Detailed implementation notes are documented in:
- `acme-docs/processes/210-reference-implementation-notes.md`
- `README.md`

## Hard Rules

- For bootstrap work, copy and rename from the reference packages; avoid inventing alternate structure without a clear reason.
- Use `bun` for TypeScript/Svelte tasks.
- Keep wire JSON naming and API conventions aligned with Underlay guides.
- Keep changes scoped; avoid unrelated refactors.

## Effigy-First Execution

Default flow from the workspace root:
1. Run `effigy tasks`
2. Run `effigy health`
3. Run `effigy validate`
4. Prefer `effigy <task>` for supported workspace and child-repo work
5. Fall back to raw package-local commands only when Effigy does not yet cover the path

For first-time local bring-up from outside this repo:
- use `effigy bootstrap git@github.com:inflatable-cookie/underlay-reference.git`
- add `--start` when you want bootstrap to launch `dev` after dependency setup

Workspace notes:
- use root Effigy tasks for cross-repo orchestration (`health`, `validate`, `qa`, `qa:docs`, `qa:northstar`, `dev`)
- use child-owned tasks from the workspace root when they resolve uniquely (`db:*`)
- when modifying a specific repo, follow that repo's local `AGENTS.md`
- do not treat `cargo build`, `bun check`, or ad hoc shell commands as the default entrypoint when an Effigy task exists

## Validation

- `effigy health`
- `effigy validate`
- `effigy qa:docs`

## Source of Truth

For reference-app planning and architecture, prefer `acme-docs/`. In the
active strict lane, a bare `continue` should resolve through the previous
`Next Task` into the current ready card under `acme-docs/specs/` or back into
planning if no ready card exists. For shared framework conventions, prefer
Underlay docs in `underlay/docs/guides/`. Do not create parallel roadmap or
report docs elsewhere in this repo.
