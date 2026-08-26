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
- `docs/processes/210-reference-implementation-notes.md`
- `README.md`

## Hard Rules

- For bootstrap work, copy and rename from the reference packages; avoid inventing alternate structure without a clear reason.
- Keep the single-repository workspace shape: runtime apps under `apps/*`,
  reusable libraries under `packages/*`, docs authority at root `docs/`.
- Keep one root `package.json` workspace declaration and one root `bun.lock`.
  Never add a child lockfile, a per-package install, or a `file:` edge between
  internal packages — use `workspace:*`.
- Use `bun` for TypeScript/Svelte tasks.
- Keep wire JSON naming and API conventions aligned with Underlay guides.
- Keep changes scoped; avoid unrelated refactors.

## Effigy-First Execution

Default flow from the workspace root:
1. Run `effigy tasks`
2. Run `effigy workspace:js:prepare` (one frozen root install)
3. Run `effigy health`
4. Run `effigy validate`
5. Prefer `effigy <task>` for supported workspace and child-repo work
6. Fall back to raw package-local commands only when Effigy does not yet cover the path

## Runtime Stance

- Treat the live stack as Effigy-owned. Use `effigy dev`, `effigy prep`, package-owned Effigy tasks, or `effigy container shell` when you need to affect the running environment.
- Do not run `bun install`, `npm install`, `pnpm install`, `cargo build`, or similar hydration/build commands on the host expecting them to change the live runtime unless the task is explicitly host-owned.
- Do not treat host-side `node_modules`, `vendor`, `target`, `.pnpm-store`, `.svelte-kit`, or similar artifact dirs as the source of truth for the running stack. Those may be isolated inside the container runtime.
- When raw `bun` or `cargo` is genuinely needed, prefer running it through `effigy container shell` when the live runtime matters.

For first-time local bring-up from outside this repo:
- use `effigy bootstrap git@github.com:inflatable-cookie/underlay-reference.git`
- add `--start` when you want bootstrap to launch `dev` after dependency setup

Workspace notes:
- use root Effigy tasks for cross-repo orchestration (`health`, `validate`, `qa`, `qa:docs`, `qa:northstar`, `dev`)
- use child-owned tasks from the workspace root when they resolve uniquely (`migration:*`)
- when modifying a specific repo, follow that repo's local `AGENTS.md`
- do not treat `cargo build`, `bun check`, or ad hoc shell commands as the default entrypoint when an Effigy task exists
- sibling `underlay` and `poodle` repos are mounted from `../underlay` and `../poodle`; do not recreate the old symlink/bootstrap pattern
- treat this repo as the canonical underlay consumer shape; prefer fixing shared patterns here or in the bundle before inventing app-specific exceptions elsewhere

## Validation

- `effigy workspace:js:prepare`
- `effigy health`
- `effigy validate`
- `effigy qa:docs`
- `effigy qa:workspace-shape`

## Source of Truth

For reference-app planning and architecture, prefer `docs/`. In the
active strict lane, a bare `continue` should resolve through the previous
`Next Task` into the current ready card under `docs/specs/` or back into
planning if no ready card exists. For shared framework conventions, prefer
Underlay docs in `underlay/docs/guides/`. Do not create parallel roadmap or
report docs elsewhere in this repo.

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `docs/policy/internal-writing-style.md`

<!-- BEGIN EFFIGY AGENT CONTRACT -->
## Effigy Agent Contract

Use Effigy as the default command surface for supported project work.

Route by job, not by startup ritual:
- use `effigy graph` for code understanding
- use `effigy tasks` for selector inventory
- use `effigy doctor` for routing ambiguity or repo health
- use `effigy test --plan` when test execution shape matters

Use `effigy graph` when the job is code understanding: ownership, flow,
implementation, or changed-file impact. Do not insert graph into unrelated
deployment, state, docs, release, or direct task-execution work.

Prefer `effigy <task>`, `effigy test`, and the matching built-in surface over
raw package-manager or shell commands when Effigy covers the path. Use
`effigy --json <command>` whenever another agent or tool will consume output.

This repo's local `.agents/skills/effigy` copy is authoritative for this
project. When an agent supports both project-local and global skills, prefer
the project-local copy over any globally installed Effigy skill.

Do not add `--repo .` while already inside the target repo. Do not edit
`.github/workflows/` or run release mutations unless the user explicitly asks.

Reference docs:
- Effigy agent adoption: `docs/guides/047-agent-and-cross-repo-adoption.md`
- Graph workflows: `docs/guides/076-code-graph-and-agent-workflows.md`
- JSON contracts: `docs/guides/017-json-output-contracts.md`
<!-- END EFFIGY AGENT CONTRACT -->
