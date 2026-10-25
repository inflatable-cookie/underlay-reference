# Agents Guide: Underlay Reference Implementation

This repository is a **reference template** for bootstrapping Underlay-based
apps. Prefer canonical, reusable patterns over one-off customization.

## Where things live

- Current state: `docs/README.md`
- Knowledge (one owner per fact): `docs/knowledge/README.md`
- Retired concepts, which must not come back: `docs/knowledge/retired.toml`
- Open questions: `docs/knowledge/questions.md`
- Papercuts: Queue via `papercut.add` (see the `northstar` skill); no
  `PAPERCUTS.md`.
- Package implementation notes: `docs/knowledge/operations/reference-implementation-notes.md`

The plan (lanes, their documents and their order), leads, papercuts, brief
drafts, tasks and status live in Queue, never in this repository. Read what's
next with `plan.get` (see the `northstar` skill).

Nested `AGENTS.md` files stay to four things: scope, local hard rules, validation
commands, and links. Point here rather than restating.

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
- When a change alters what is true, update the owning knowledge file in the
  same PR.
- An operator ruling given in conversation goes into its owning file before
  the thread ends.

## Effigy-First Execution

Effigy is the default command surface. Use the installed Effigy Agent Skill for
shared command routing. These rules are specific to this workspace:

- JS/Svelte work needs one frozen root install first: `effigy workspace:js:prepare`.
  That is the only install; never run a per-package `bun install`.
- Prefer `effigy <task>` over raw package-manager or shell commands wherever a
  task exists, and fall back to raw commands only where Effigy does not yet
  cover the path.

Completion gates belong at the end of a change, not the start of a turn. See
Validation.

## Runtime Stance

- Treat the live stack as Effigy-owned. `effigy dev` runs the hybrid profile (services in containers, API/jobs/front/admin on the host); `effigy dev:container` is the whole-stack container session. Use `effigy dev`, `effigy dev:down`, `effigy hybrid:exec -- <cmd>` or a package-owned task (`acme-admin/prepare`, `acme-front/prepare`, `acme-api/migration:*`) when you need to affect the running environment.
- Do not run `bun install`, `npm install`, `pnpm install`, `oxide build`, or similar hydration/build commands on the host expecting them to change the live runtime unless the task is explicitly host-owned.
- Do not treat host-side `node_modules`, `vendor`, `target`, `.pnpm-store`, `.svelte-kit`, or similar artifact dirs as the source of truth for the running stack. Those may be isolated inside the container runtime.
- When `bun` or `oxide` needs the live services, run it through `effigy hybrid:exec -- <cmd>` (hybrid) or `effigy container shell` (container mode). `hybrid:exec` names the hybrid profile, not a guest: the bundle's task executes its command on the host with the profile environment, so Rust there is a host Rust call.

For first-time local bring-up from outside this repo:
- use `effigy bootstrap git@github.com:inflatable-cookie/underlay-reference.git`
- add `--start` when you want bootstrap to launch `dev` after dependency setup

Workspace notes:
- root-owned orchestration tasks are `dev`, `health`, `validate`, `qa`, and the
  `qa:*` conformance set
- child-owned tasks resolve from the workspace root when the name is unique:
  `qa:docs` and `qa:northstar` (acme-docs), `migration:*` (acme-api)
- when modifying a specific repo, follow that repo's local `AGENTS.md`
- do not treat `oxide build`, `bun check`, or ad hoc shell commands as the default entrypoint when an Effigy task exists
- This workspace consumes released Underlay and Poodle versions (pinned in `effigy.toml`, `Cargo.toml` and the `package.json` files)
  dependencies. Its Effigy bundle disables sibling catalogs and bootstrap
  children with `bundle.sources.siblings = false`.
- treat this repo as the canonical underlay consumer shape; prefer fixing shared patterns here or in the bundle before inventing app-specific exceptions elsewhere

## Host Rust

Host Rust routes use Oxide, not Cargo. See the managed Oxide block at the end
of this file, and the host selectors in `apps/acme-api/effigy.toml`:

- Write `oxide` wherever you would write `cargo` (`oxide build`, `oxide test`,
  `oxide run -p acme-api`); `oxide exec -- <tool>` is only for a tool that runs
  Cargo itself or owns its target directory.
- The API, jobs, build, check, clippy, fmt, `rust` test suite, `test:db` and
  `migration:*` selectors all route host Rust through Oxide. Keep them that way;
  do not add a per-selector `:cargo` escape or a plain-`cargo` twin.
- The operator escape is `EFFIGY_OXIDE_ESCAPE=cargo` or running Cargo by hand.
  It is deliberate and local, never a selector fallback.
- Keep `cargo:` directives in `build.rs`, true CI/guest recipes and Cargo-subject
  fixtures as Cargo.

## Validation

Run `effigy workspace:js:prepare` first if JavaScript dependencies are missing.
Choose the narrowest selector for the change, and run each required check once:

- Docs-only changes: `effigy acme-docs/qa:docs`.
- Package changes: the touched catalog's `validate` selector — `acme-api`,
  `acme-admin`, `acme-front`, `acme-client`, `acme-ui`, or `acme-docs` — when
  the change needs that package gate.
- Workspace changes: `effigy health` for the cheap workspace check.

Run full `effigy qa` and workspace conformance checks on `main` at Queue
milestones, not as a per-task default. Root `effigy validate` and `effigy qa`
cover this repository's catalogs; the bundle opts out of sibling catalogs.
Open papercuts are tracked in Queue.

## Scratch

Agent and proof scratch belongs on the mounted Builds volume:
`/Volumes/Builds/scratch/underlay/<unique-run>`. Never use the home directory
or a drive root for scratch, and never recreate the retired `~/ur.*` fixtures.
A missing mounted volume refuses the placement; do not silently fall back to a
home or system temp path or invent a mount. Colima host mounts are allowed only
when an explicitly authorized future run needs them. Delete only scratch your
own finite checks created, and leave unknown state alone. Product-managed temp
files are not agent scratch and keep their existing behavior.

## Env And Secret Authority

- `config/env-manifest.txt` is the complete env surface; `config/required-secrets.txt`
  is the startup-critical subset. Adding a runtime env read without adding it to
  the manifest is drift.
- `.env`, `.env.local`, and `.env.example` are not part of the runtime contract.
  Non-secret values go in the root `config/` stack; secrets go through Effigy
  runtime injection or the local vault.
- Never commit a secret value. The authority files carry keys only.

## Shared framework docs

For shared framework conventions, prefer Underlay docs at
<https://github.com/inflatable-cookie/underlay/tree/v0.10.2/docs/guides/>. Do
not create parallel planning or report docs elsewhere in this repo.

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `docs/knowledge/contracts/writing-style.md`

## Effigy Agent Skill

Use the maintained Effigy Agent Skill from the installed user-level skill
roots, normally `~/.agents/skills/effigy`. Other supported roots should resolve
to that same canonical directory. If the skill is missing or roots resolve to
different trees, refresh the maintained skill at the canonical root, fix the
aliases, then start a fresh agent context. Do not add a repository-local copy;
plain `effigy init` does not install or refresh the skill.

The installed skill and `effigy` executable are separate. Check the executable
with `command -v effigy`; when host-wide admission is needed, require
`effigy admission status --json` to report `effigy.admission.status.v1`.
Repository selectors and runtime guardrails remain in this file.

<!-- northstar:typescript-quality:start -->
## Northstar TypeScript/Svelte explicit audit

Use Northstar's TypeScript/Svelte quality pack only when the operator explicitly
requests a TypeScript or Svelte quality audit, no-slop pass, whole-codebase
review, or audit-and-fix action. Ordinary TypeScript/Svelte coding does not
activate it.

For explicit audit intent, run the installed-package route from this directory:
`effigy skill run northstar/language:route -- --consumer . --marker
northstar:typescript-quality --workflow explicit_audit_repair`, then follow the
`entrypoint_path` it returns. Resolve package ownership and
strict profile state before assessment. Record findings before mutation, keep
repairs inside recorder-authorized files, preserve pre-existing dirty work, and
use repository-owned compiler, framework, lint, and test evidence without
installing dependencies or inventing commands.
<!-- northstar:typescript-quality:end -->

<!-- oxide:begin -->
<!-- oxide:workspaces: apps/acme-api -->
## Rust commands: use `oxide`

Covers Cargo workspaces `apps/acme-api`, relative to this file's folder.

- Write `oxide` wherever you would write `cargo`: `oxide check`, `oxide test`, `oxide run -- <args>`, `oxide add <crate>`.
- Profiles: `quick` (default), `--profile debug` for a debugger, `--release`. Oxide places build output, so it refuses `--target-dir`.
- A tool that runs Cargo itself (Tauri CLI, npm or bun scripts): `oxide exec -- <command>`.
- A `waiting for …` line means another build holds a slot; let it wait. `oxide status` shows the pool, slots and waits.
- Use `oxide clean`, not `cargo clean`.

Managed by `oxide init`; edits between these markers are replaced.
<!-- oxide:end -->
