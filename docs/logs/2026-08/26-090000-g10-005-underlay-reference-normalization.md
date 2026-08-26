# 2026-08-26 09:00:00 - g10.005 Underlay Reference Normalization

## Summary

Executed Underlay `g10.005`. The reference fixture now physically matches the
single-repository monorepo contract (Underlay contract `024`, spec
`monorepo-consumer-workspace-rollout`): runtime apps under `apps/*`, reusable
packages under `packages/*`, docs authority at root `docs/`, one root Bun
workspace manifest and lockfile, and internal `workspace:*` edges. The sibling
Underlay workspace-shape checker goes from seven violations to zero.

Application dependency versions are unchanged: Underlay Git tag `v0.9.4` on both
language surfaces and Poodle `0.2.2`.

## Directory mapping

All five package moves and the docs move used `git mv`; `git diff --summary`
reports them as renames.

| Before | After |
| --- | --- |
| `acme-api/` | `apps/acme-api/` |
| `acme-admin/` | `apps/acme-admin/` |
| `acme-front/` | `apps/acme-front/` |
| `acme-client/` | `packages/acme-client/` |
| `acme-ui/` | `packages/acme-ui/` |
| `acme-docs/*` | `docs/*` |

`docs/handoffs/` already existed at the base commit, so each tracked `acme-docs`
entry moved into the existing root `docs/` around it. `acme-docs/` no longer
exists and no compatibility directory, symlink, or path fallback was left
behind. Package and crate identities (`acme-api`, `acme-client`, `@acme/ui`, …)
and every Effigy catalog alias are unchanged.

## Root workspace and lock

Root `package.json` now pins `packageManager` to `bun@1.3.14` and declares the
four JavaScript workspace members explicitly:

- `apps/acme-admin`
- `apps/acme-front`
- `packages/acme-client`
- `packages/acme-ui`

`apps/acme-api` is Rust-only and stays out of the JavaScript workspace; its
Cargo workspace remains app-local and was not hoisted. `apps/acme-api/Cargo.lock`
is a byte-identical rename.

Four child `bun.lock` files were removed and replaced by one root `bun.lock`
generated through the Effigy-owned root install task. The four internal `file:`
edges (`@acme/ui` and `acme-client` in Admin and Front) are now `workspace:*`.

Lock evidence:

- root lock declares `apps/acme-admin`, `apps/acme-front`,
  `packages/acme-client`, `packages/acme-ui`
- `@inflatable-cookie/underlay` resolves to
  `git+ssh://…/underlay.git#7004af5b3461b6c89a7faa646575ff69576c73b8`, which is
  the dereferenced `v0.9.4` tag commit and the same identity the removed child
  locks recorded
- `@inflatable-cookie/poodle-core` and `@inflatable-cookie/poodle-svelte` stay at
  `0.2.2`
- no sibling application source paths appear in the lock

`workspace:js:prepare` is the new repo-owned root install task
(`bun install --frozen-lockfile`). The initial lock was generated through that
same task, then the frozen form replayed clean.

## Two declared dependencies that were previously phantom

The old per-package installs used Bun's hoisted linker, so two undeclared direct
imports resolved by accident. One root workspace uses Bun's isolated linker
(matching Acowtancy, the live proof), which exposed them as hard `svelte-check`
errors:

- `packages/acme-ui` imports `marked` — now declared as `^18.0.9`, the range
  `@inflatable-cookie/poodle-svelte@0.2.2` already carried
- `apps/acme-front` imports `lucide-svelte` — now declared as `^1.0.1`, matching
  the declaration `apps/acme-admin` already carried

Neither adds a package to the resolved set; both make an existing transitive
resolution explicit. No Underlay or Poodle version moved.

## Effigy and path alignment

- root `[bundle.dirs]` now maps to `docs`, `apps/acme-api`, `apps/acme-admin`,
  `apps/acme-front`, `packages/acme-client`, `packages/acme-ui`. That input also
  drives `catalog.members`, container `isolated_dirs`, deploy `source_root`
  values, and bootstrap dependency sync, all of which now resolve correctly.
- root `qa:templates` targets `apps/acme-admin`.
- new root `qa:workspace-shape` runs the sibling Underlay checker through the
  existing `../underlay` tooling boundary. It uses a relative sibling path, not
  an absolute workstation path, and does not touch application dependencies.
- Admin and Front Svelte/Vite aliases now point at
  `../../packages/acme-client/src`.
- Admin and Front `scripts/generate-public-config.ts` derive the project root
  three levels up instead of two. Both regenerate `public-api.generated.ts` with
  the values from root `config/default.toml`.
- `docs/scripts/rollout-checks.rhai` targets `../apps/acme-api`,
  `../apps/acme-admin`, and `../packages/acme-client`. All three retained rollout
  checks pass.
- Active docs, READMEs, and `AGENTS.md` files use the new physical paths. Effigy
  catalog aliases (`acme-docs/qa:docs`, `acme-admin/check`, …) are semantic names
  and stayed as they were.

## Repo-owned root task override (bundle limitation)

The shared Underlay Effigy bundle renders the root `health`, `validate`, `qa`,
and `dev` sequences as `{{ inputs.dirs.<role> }}/<task>`, reusing `bundle.dirs`
as both the physical directory and the task-selector prefix. Under `apps/*` and
`packages/*` those are no longer the same string, so the generated selectors
resolved to `apps/acme-admin/health` and failed with `task catalog prefix
'docs' not found`. Setting `bundle.dirs` to the alias instead is not possible:
the same input feeds `catalog.members`, which must be a real path (verified —
removing `[bundle.dirs]` fails with `invalid catalog member declared at
catalog.members.admin`).

This repo therefore restates those four sequences in its own `effigy.toml`
against catalog aliases, with a comment marking the override as removable once
the bundle can express selectors separately from dirs. Two `PAPERCUTS.md`
entries record the bundle defect and a related one: bundle container
`isolated_dirs` still assume per-package `node_modules`, which a hoisting root
workspace no longer produces.

**This needs an orchestrator decision before `g10.006`–`g10.009`.** Every other
bundle-backed consumer will hit the same wall. Acowtancy sidesteps it by owning
`infra/*.toml` fragments instead of using the shared bundle.

## Validation

Run from the worker worktree after one frozen root install:

- `effigy workspace:js:prepare` — frozen install clean, no lock drift
- `effigy tasks` — nine catalogs resolve to their new physical roots
- `effigy health` — pass (docs rollout, API fmt, client, UI, admin, front; all
  `0 ERRORS 0 WARNINGS`)
- `effigy qa:docs` — pass
- `effigy test --plan` — four targets (`acme-admin`, `acme-api`, `acme-client`,
  `acme-front`), Underlay and Poodle excluded; unchanged from the base
- `effigy acme-admin/check`, `acme-front/check`, `acme-ui/check`,
  `acme-client/check` — pass
- `effigy acme-api/build` — pass against released Underlay `v0.9.4`
- `effigy acme-docs/check:rollout` for `admin-freshness`,
  `auth-security-alerting`, `reorder-conflict` — pass
- `effigy qa:templates`, `effigy qa:security` — pass
- `effigy qa:workspace-shape` — all checks passed (was seven violations)
- `git diff --check` — clean

Full root `validate`/`qa` were **not** run and are not claimed green: the
bundle composes `underlay/validate` (the sibling framework's own full gate) into
this repo's root `validate`.

## Baseline notes carried forward, not fixed here

- `effigy doctor` still reports four errors and two warnings: unsupported root
  `test.exclude_catalogs`, sibling Underlay's unsupported `isolation` key, scan
  findings (attention markers, god files, generated-in-src), and the built-in
  `docs` task-reference false positive already recorded in `PAPERCUTS.md`. Docs
  QA itself passes.
- The handoff recorded a baseline of four Poodle Svelte accessibility warnings
  from Front health. Front now reports zero: with the isolated linker
  `svelte-check` no longer walks hoisted Poodle sources. No Poodle code changed.
- The handoff described the base `effigy test --plan` as five target catalogs;
  the measured base and the post-migration result are both four.
- `effigy health` cannot pass in a fresh checkout until
  `effigy workspace:js:prepare` runs, because `bun x tsc`/`svelte-check` fall
  back to the latest published binaries without a workspace install. The base
  had no root install task at all; this is what `workspace:js:prepare` fixes.
- Package `AGENTS.md` reference lists still use `../underlay/docs/guides/...`.
  That path was already wrong before this move (it resolved inside the repo) and
  is a sibling-mount convention, not a consequence of the migration. Left alone.
- `packages/acme-client/tsconfig.json` still sets `baseUrl`, which TypeScript 7
  rejects. It is harmless with the workspace-pinned `typescript@^5.9.3` and was
  only visible when `bun x tsc` fell through to the published latest.

## Boundaries respected

- `g01.007` remains the target's active retained-surface lane. Only `acme-docs`
  path literals in its spec, batch card, roadmap milestone, and working rules
  changed; statuses, decisions, and `Next Task` entries are untouched.
- Historical logs and closed roadmap milestones (`g01.001`–`g01.005`, `g01.008`–
  `g01.011`) keep their old path strings as execution evidence.
- No `.github/workflows/` edits, release mutations, other-repository edits, or
  application dependency version changes.

## Next Task

Return the PR to the orchestrator for review. Before dispatching
`g10.006`–`g10.009`, decide how bundle-backed consumers should express root task
selectors under the monorepo shape.
