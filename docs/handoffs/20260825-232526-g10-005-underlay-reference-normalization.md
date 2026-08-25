---
title: g10.005 Underlay Reference normalization worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom
created: 2026-08-25
updated: 2026-08-25
handoff_path: /Users/tom/Dev/projects/underlay-reference/docs/handoffs/20260825-232526-g10-005-underlay-reference-normalization.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, g10, monorepo, reference]
---

## What This Thread Was Doing

Underlay's strict `g10` rollout now treats Acowtancy's single-repository
`apps/*` / `packages/*` workspace as the only supported normal consumer shape.
The authority, guide, checker, and Acowtancy evidence batches (`g10.001` through
`g10.004`) are complete.

This handoff starts `g10.005`: make Underlay Reference, the bootstrap fixture
that downstream teams copy, physically match that contract. Its application
dependencies already use released Underlay `v0.9.4` and Poodle `0.2.2`; this
worker owns the remaining directory, workspace, lockfile, internal-edge,
tooling-path, and active-doc migration.

## Why It Matters

Underlay Reference is meant to be copied, not interpreted. While it still has
five top-level package directories, a named docs directory, four child locks,
and internal `file:` edges, bootstrap teaches the topology Underlay has just
retired. This merge becomes the reference fixture for the four remaining
consumer migrations and unlocks their independent worker lanes.

## Current State

- **Repository:** `git@github.com:inflatable-cookie/underlay-reference.git`.
- **Planning branch:** `main`.
- **Planning base commit:** `85e097d3525a5867da50ec45e711b9ab760ad4eb`.
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved to
  `85e097d3525a5867da50ec45e711b9ab760ad4eb` before this handoff was created.
- **Planning checkout:** clean before this handoff was created.
- **Worker mode:** implementation worker dispatched by the Northstar
  orchestrator; this handoff activates the worker-only worktree preflight.
- **Planning artifacts included at the base:** the target's existing Northstar
  docs and a `PAPERCUTS.md` entry recording the pre-existing Effigy Doctor docs
  built-in resolution defect.
- **External planning authority:** Underlay `main` at
  `abc7b26a198b137e102a028b5f04cac90b513fbe`.
- **Worker branch label:** `worker/g10-005-underlay-reference-normalization`.
- **Worker worktree:** use the clean dedicated non-`main` worktree supplied by
  the launcher. This handoff does not select or create a manual fallback path.
- **Manual fallback:** if the launcher context is unusable, inspect
  `.agents.local.env` for an absolute `AGENTS_WORKTREE_CONTAINER_DIR`. The file
  is absent from the planning checkout. Stop and ask the operator rather than
  creating it or guessing a path.
- **Existing unrelated worktree:** a registered
  `t3code/follow-soundcheck-adoption-handoff` worktree already exists. It is not
  this lane's worktree. Do not edit, clean, reuse, or remove it.
- **Active external spec:**
  `/Users/tom/Dev/projects/underlay/docs/specs/monorepo-consumer-workspace-rollout.md`.
- **External roadmap milestone:**
  `/Users/tom/Dev/projects/underlay/docs/roadmaps/g10/README.md`.
- **Assigned card:**
  `/Users/tom/Dev/projects/underlay/docs/roadmaps/g10/batch-cards/005-underlay-reference-normalization.md`.
- **Allowed runway:** `g10.005` only.
- **Remaining card budget:** one card.
- **Dispatch topology:** serial. `g10.006` through `g10.009` remain blocked
  until this PR is reviewed and the operator authorizes its merge.
- **Parallel safety:** do not start another consumer lane. This fixture defines
  the exact migration pattern those later workers will follow.
- **Target canonical refs:** `AGENTS.md`, `README.md`, `PAPERCUTS.md`,
  `acme-docs/README.md`, `acme-docs/policy/001-working-rules.md`,
  `acme-docs/roadmaps/README.md`, `acme-docs/roadmaps/g01/README.md`, and
  `acme-docs/specs/001-retained-underlay-surface-strict-lane.md`.
- **Underlay canonical refs:**
  `/Users/tom/Dev/projects/underlay/docs/contracts/024-new-app-bootstrap-and-bring-up.md`,
  `/Users/tom/Dev/projects/underlay/docs/contracts/120-tooling-testing-and-contract-artifacts.md`,
  `/Users/tom/Dev/projects/underlay/docs/guides/020-project-structure.md`,
  `/Users/tom/Dev/projects/underlay/docs/guides/030-underlay-integration.md`, and
  `/Users/tom/Dev/projects/underlay/docs/architecture/product-guardrails.md`.
- **Required directory mapping:** `acme-api` → `apps/acme-api`, `acme-admin` →
  `apps/acme-admin`, `acme-front` → `apps/acme-front`, `acme-client` →
  `packages/acme-client`, `acme-ui` → `packages/acme-ui`, and the contents of
  `acme-docs` → root `docs`.
- **Handoff destination edge:** this file intentionally creates
  `docs/handoffs/` before the docs migration. Move each tracked top-level entry
  from `acme-docs` into the existing root `docs/`; do not produce
  `docs/acme-docs`, overwrite this handoff, or leave a compatibility directory.
- **Root manifest baseline:** root `package.json` is private but lacks
  `packageManager` and `workspaces`; no root `bun.lock` exists.
- **JavaScript baseline:** four child `bun.lock` files exist. Admin and Front
  carry four total internal `file:` edges to `acme-client` and `@acme/ui`.
- **Required root manifest shape:** keep name `underlay-reference`, keep it
  private, pin `packageManager` to `bun@1.3.14`, and explicitly list only
  `apps/acme-admin`, `apps/acme-front`, `packages/acme-client`, and
  `packages/acme-ui`. The Rust-only API is not a JavaScript workspace member.
- **Released dependency boundary:** all four JavaScript packages use Underlay
  Git tag `v0.9.4`; Poodle packages use `0.2.2`; the API Cargo workspace uses
  Underlay Git tag `v0.9.4`. Preserve those versions and lock identities.
- **Local-link state:** `effigy --json deps status` reports no active local
  links. Sibling Underlay/Poodle mounts remain allowed for QA/tooling and
  optional machine-local co-development only.
- **Path-sensitive live surfaces:** root `effigy.toml` bundle dirs and template
  conformance path; Admin/Front Svelte and Vite client aliases; their config
  generators' project-root depth; docs Effigy catalog and rollout Rhai paths;
  root/package README and AGENTS paths; config/runtime/task references; tests
  and any source code that names physical workspace paths.
- **Target planning boundary:** target `g01.007` remains its active retained-
  surface semantic lane. Update physical paths and current front-door links
  needed by this move, but do not execute, close, reopen, or redefine that
  lane. Historical logs and closed roadmaps remain historical evidence.
- **Open PR state:** none at dispatch preparation.
- **Baseline validation:** `effigy health` passes; `effigy qa:docs` passes;
  `effigy test --plan` resolves five target catalogs and excludes Underlay and
  Poodle; the workspace checker reports exactly seven expected violations.
- **Expected checker violations:** four child locks plus missing root
  `packageManager`, root lock, and root workspaces.
- **Known health output:** Front health reports four existing Poodle Svelte
  accessibility warnings and no errors. Do not fix Poodle in this card.
- **Known test-routing baseline:** full root `validate`/`qa` can auto-select
  Vitest for Front tests outside the configured include and for UI with no
  owned suite. The latest adoption PR recorded this separately. Do not add
  `passWithNoTests`; use the card's planned test surface and stop if topology
  work requires a broader test-policy decision.
- **Known Doctor debt:** unsupported root `test.exclude_catalogs`, sibling
  Underlay's unsupported `isolation`, scan findings, stale graph state, and the
  built-in `docs` task-reference false positive recorded in `PAPERCUTS.md`.
  Docs QA itself passes. Do not attribute or broadly fix this debt.
- **Tool/runtime restrictions:** use Effigy-first execution; no release
  mutations, `.github/workflows/` edits, other-repository edits, or package
  version bumps.
- **Required validation:** one frozen root install through Effigy;
  `effigy health`; `effigy qa:docs`; `effigy test --plan`; targeted Admin,
  Front, UI, Client checks and API build; the sibling Underlay workspace-shape
  check; active old-path searches; and `git diff --check`.
- **PR base/head:** `main` ← selected worker branch.
- **PR URL:** pending.
- **Review state:** awaiting implementation PR.
- **Merge authorisation:** not granted. The worker must not merge.

## Boundaries

Please keep this run inside `g10.005`:

- **In scope:** history-preserving moves into `apps/*`, `packages/*`, and root
  `docs/`; exact root workspace manifest; one root Bun lock; internal
  `workspace:*` edges; one Effigy-owned frozen root install; bundle/catalog,
  alias, config-depth, rollout-script, test, active docs, and instruction path
  updates; a repo-owned workspace-shape QA entry when it can use the existing
  sibling Underlay tooling boundary without changing application dependencies;
  and one local execution log for this migration.
- **Out of scope:** Underlay/Poodle version changes; Cargo workspace hoisting;
  app behavior, routes, auth, persistence, migrations, shared runtime, template
  extraction, or retained-surface semantics; Effigy schema design; Doctor scan
  cleanup; Poodle warning fixes; test-policy redesign; workflows; releases;
  another consumer; and Underlay planning-file edits.
- Use `git mv` for the five application/package directory moves. Root `docs/`
  already exists for this handoff, so move tracked `acme-docs` entries into it
  explicitly and remove the empty `acme-docs` directory. Do not use copy/delete
  churn that obscures history.
- Preserve package and crate names. This card changes physical ownership paths,
  not public JavaScript package identities or Rust crate identities.
- Keep the API Cargo workspace under `apps/acme-api`; do not hoist Cargo files
  to the repository root.
- Replace all four internal JavaScript `file:` declarations with
  `workspace:*`. Do not introduce a new `file:`, committed Cargo path to
  Underlay, package override, compatibility symlink, directory alias, or
  fallback for the retired layout.
- Preserve tagged Underlay `v0.9.4` and Poodle `0.2.2` dependencies. A sibling
  Underlay path may appear only in QA/tooling task commands or machine-local
  untracked link state, never in manifests or locks.
- Keep one root `bun.lock` and no child locks. Generate the root lock in the
  Effigy-owned runtime, then prove routine installation with
  `bun install --frozen-lockfile` through a root Effigy task. Do not hydrate the
  host and assume it represents the live workspace.
- Keep root `package.json` free of Effigy task mirrors. Existing package scripts
  may remain convenience wrappers.
- Update active docs, instructions, examples, and QA references to the new
  paths. Leave historical logs and closed-roadmap evidence unchanged unless a
  live docs check requires a link repair; do not rewrite frozen history merely
  because it records the old layout.
- Preserve the target's current `g01.007` statuses, decisions, and next task.
  The migration may change `acme-docs` path literals to `docs` in active
  authority surfaces, but it may not advance that semantic lane.
- If the root lock cannot be generated and installed frozen, released Underlay
  cannot compile after the move, an Effigy catalog cannot express the new
  paths, a package role becomes ambiguous, or validation demands unrelated app
  or test-policy work, stop and return evidence to the orchestrator.
- Work only in the selected clean worker worktree. Do not merge the PR.

## Important Context

- **Planning lineage:** `g10.001` made the monorepo contract normative;
  `g10.002` aligned active guidance; `g10.003` supplied the conformance checker;
  `g10.004` made Acowtancy's evidence truthful. This is the final serial proof
  before the remaining four consumers can migrate independently.
- **Why the card is ready:** package roles are unambiguous, the target has one
  Git root and no open PR, released dependency adoption has already merged, the
  exact remaining violations are known, and contract `024` fixes the target
  shape without an architecture choice.
- **Reference shape:** retain Acme names. Contract `024` allows product-specific
  names as long as the root README maps them to API/admin/front/client/UI roles.
- **Root lock:** the four existing locks were generated independently. Remove
  them only as part of the coherent root-workspace change, generate one root
  lock, and inspect that it resolves all four workspace packages, Underlay
  `v0.9.4`, and Poodle `0.2.2` without sibling application sources.
- **Root install task:** contract `024` names `workspace:js:prepare` as the
  normal shape. Reuse an existing bundle seam if it already supplies the exact
  frozen root install; otherwise add the smallest repo-owned Effigy task with
  that behavior and compose it only where the current root lifecycle requires.
  Do not add a new Effigy schema key.
- **Checker boundary:** Underlay `v0.9.4` predates the new bin, while the sibling
  checkout already supplies the checker. Using
  `bun /Users/tom/Dev/projects/underlay/ts/bin/underlay-workspace-shape.ts .`
  as a local QA/tooling check is allowed. A committed repo task may use the
  existing sibling-tooling boundary, but never an absolute workstation path.
  Do not change the application's pinned Underlay dependency or use a committed
  `file:` edge to obtain the checker.
- **Docs destination:** because this handoff must live under root
  `docs/handoffs/`, the destination is not empty. Preserve the handoff and move
  the existing docs authority around it. After the move, docs catalog alias and
  selectors may remain `acme-docs` if they are semantic names; physical path
  claims must say `docs/`.
- **Alias depth:** Admin and Front currently point to `../acme-client/src` from
  top-level app roots. After the move their source target is
  `../../packages/acme-client/src`. Update Svelte/Vite aliases and validate
  their package-owned checks rather than relying on search alone.
- **Config depth:** Admin and Front config generators currently derive the
  project root two levels above their script file. After moving under `apps/`,
  the repository root is three levels above. Prove generated config still reads
  root `config/`.
- **Docs rollout paths:** after `acme-docs/scripts/rollout-checks.rhai` becomes
  `docs/scripts/rollout-checks.rhai`, API/Admin targets move under `../apps/`
  and Client targets under `../packages/`. Run all three retained rollout
  checks after rewriting paths.
- **Effigy paths:** root `[bundle.dirs]` must resolve to `docs`,
  `apps/acme-api`, `apps/acme-admin`, `apps/acme-front`,
  `packages/acme-client`, and `packages/acme-ui`. Template conformance must
  target `apps/acme-admin`. Re-run `effigy tasks` after moving before trusting
  package selectors.
- **Historical boundary:** path strings in frozen logs and closed roadmaps are
  historical evidence, not compatibility fallbacks. Active source, config,
  tests, root/package READMEs, AGENTS files, current docs authority, and Effigy
  wiring must use the new paths.
- **Test baseline:** do not use the unsupported `test.exclude_catalogs` Doctor
  finding as a reason to remove Underlay/Poodle exclusion blindly. Preserve the
  effective five-target test plan unless a card-local supported representation
  is clear. Bring any schema decision back to the orchestrator.
- **Papercut:** the pushed base records Doctor's false unresolved-reference
  findings for built-in docs steps. Do not add selector aliases merely to make
  Doctor quiet when `effigy qa:docs` already proves the routing works.
- **Report after:** first, structural moves plus root manifest/lock and frozen
  install proof; second, Effigy/alias/config/Rhai/current-doc path alignment;
  third, final checker and package validation with the complete old-path audit.
- **Report to:** the operator, who will relay progress to the orchestrator.

## Suggested Next Move

Read this handoff from the top. Before broad repository reads, run the worktree
safety preflight below. If the launcher supplied a clean registered non-`main`
worktree, use it immediately regardless of its generated path or branch name.

Then read local `AGENTS.md`, the external Underlay spec/card/contracts, root and
child manifests, root and child Effigy catalogs, active docs front doors, and
the rollout Rhai script. Run `effigy tasks`, `effigy doctor`,
`effigy test --plan`, and the baseline workspace checker from the selected
worktree. Use targeted `rg` searches for physical path literals, `file:`, locks,
and aliases. Plan the complete path map before moving files; the first coherent
implementation chunk should leave the repo with its final directory structure,
root manifest, and root lock rather than a half-moved intermediate shape.

## Completion Protocol

### Before you start

1. This handoff's `worker_mode: implementation` and
   `dispatch_authority: orchestrator` metadata activate worker mode. Before any
   broad reads, run:

   ```sh
   git rev-parse --show-toplevel
   git branch --show-current
   git status --porcelain
   git worktree list --porcelain
   ```

2. If the current root is a registered worktree, its status is empty, and its
   branch is not `main`, accept it as launcher-provided. Record its actual root
   and branch. Do not create another worktree because its generated path or
   branch differs from this handoff's label.
3. If the launcher supplied a dirty or `main` worktree, stop and report it. For
   a manual fallback, inspect `.agents.local.env` and require an absolute
   `AGENTS_WORKTREE_CONTAINER_DIR`. Ask the operator when the file or key is
   absent. Never use `/tmp`, `TMPDIR`, or a guessed path; never clean, reset,
   stash over, or discard another checkout's state. Do not reuse the unrelated
   existing `follow-soundcheck-adoption-handoff` worktree.
4. From the selected worktree, run `git fetch origin`. Confirm `HEAD` equals
   `origin/main`, confirm
   `git merge-base --is-ancestor 85e097d3525a5867da50ec45e711b9ab760ad4eb HEAD`,
   and confirm this handoff exists in `HEAD`.
5. Read root and directory-local `AGENTS.md`, the external active spec,
   milestone, assigned card, contract `024`, tooling contract `120`, and the
   target refs named above.
6. Run `effigy tasks`, `effigy doctor`, `effigy test --plan`,
   `effigy --json deps status`, `effigy health`, `effigy qa:docs`, and the
   baseline sibling workspace-shape checker. Record the known failures and
   warnings without widening the card. Use `rg` for exact path/token proof;
   graph is optional only if ownership or changed-file impact becomes unclear.

### While you work

- Execute only `g10.005`.
- Keep the structural/root-workspace chunk coherent: history-preserving moves,
  exact root manifest, internal `workspace:*` edges, one root lock, no child
  locks, and frozen-install proof.
- In the second chunk, update every live Effigy, alias, config-depth, Rhai,
  test, active docs, and instruction path. Re-run `effigy tasks` immediately
  after catalog rewiring and stop if ownership does not resolve cleanly.
- Keep package/crate names and application behavior unchanged. A broad source
  diff beyond path mechanics needs direct evidence and orchestrator review.
- Search for old top-level path claims after editing. Classify remaining hits as
  historical evidence, semantic package names, or defects; do not blanket-
  replace package identifiers such as `acme-api` or `acme-client`.
- Keep one migration log under `docs/logs/2026-08/` with the actual mapping,
  lock/dependency evidence, validation, known baseline warnings, and stop
  boundaries. Update target docs front doors only where the physical move makes
  current links or selectors stale; preserve `g01.007` state.
- After each named report point, tell the operator which files moved or changed,
  what validation actually ran, what remains, and whether a planning decision
  is needed.
- Stop on any named stop condition. Do not add a compatibility layer or quiet
  unrelated validation by changing product/test policy.

### When the assigned runway is complete

1. Prove one frozen root install through the final Effigy task. Then run:

   ```sh
   effigy tasks
   effigy health
   effigy qa:docs
   effigy test --plan
   effigy acme-admin/check
   effigy acme-front/check
   effigy acme-ui/check
   effigy acme-client/check
   effigy acme-api/build
   bun /Users/tom/Dev/projects/underlay/ts/bin/underlay-workspace-shape.ts .
   git diff --check
   ```

   Use the moved catalog aliases above; Effigy should resolve their new physical
   roots through `[bundle.dirs]`. Also run all three docs rollout checks through
   the moved docs catalog.
2. Confirm the repository has one root `bun.lock`, no child locks, no internal
   `file:` edges, no committed Underlay/Poodle source paths, no old-path
   compatibility symlinks/fallbacks, and no application dependency version
   changes. Inspect the root lock for all four workspace members, Underlay
   `v0.9.4`, and Poodle `0.2.2` identities.
3. Inspect the final `git diff --summary` for rename detection and the full
   diff for accidental semantic churn. Ensure this handoff remains at its root
   `docs/handoffs/` path and `acme-docs/` no longer exists.
4. Push the selected worker branch.
5. Open a reviewable PR against the current pushed `main`. The planning base
   above predates the handoff commit and is intentionally not self-referential.
6. In the PR body, link the external spec/card, list the exact directory map,
   root manifest/lock evidence, Effigy/path changes, validation, known baseline
   warnings, and any remaining historical old-path hits. Do not claim full root
   `validate`/`qa` green unless those commands actually pass.
7. Report the PR URL, actual branch/worktree, changed files, rename/lock proof,
   exact validation, warnings, and any stop condition encountered. Do not merge.
8. Leave Underlay card/log/front-door promotion and the parallel
   `g10.006`–`g10.009` dispatch decision to the orchestrator after review and
   operator-authorized merge.
