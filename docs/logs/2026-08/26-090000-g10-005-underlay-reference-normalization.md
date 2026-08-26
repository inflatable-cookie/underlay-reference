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

`workspace:js:prepare` is the bundle-owned root install task
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

## Review round 1 (2026-08-26)

Review verdict was changes-requested on three points. Response:

1. **Repair the shared bundle rather than override it locally.** Done as
   `underlay-effigy-bundle#1`. It adds optional `catalogs.<role>` selector
   inputs that fall back to `dirs.<role>`, and a `workspace.js_root` mode that
   swaps bootstrap's per-package dependency sync for one
   `workspace:js:prepare` frozen root install and isolates the hoisted root
   `node_modules` instead of per-package trees. `scripts/dev/ui-setup.rhai`
   detects a root workspace from the filesystem and collapses hydration to a
   single frozen root install. Verified against a path-bundle checkout: this
   repo then needs zero root sequence overrides and all validation still
   passes. Flat consumers render byte-identically to the previous bundle
   revision. The bundle PR merged as `e680157e`, and this repo now consumes the
   shared inputs directly.
2. **One root install for bootstrap/dev.** Covered by the bundle change above.
   Repo side: `packages/acme-ui` no longer owns a `refresh:deps` install task,
   and its README states that dependencies come from the root install.
3. **Active setup/test docs.** `effigy bootstrap:deps` does not resolve to a
   root task (it is ambiguous across the sibling Underlay and Poodle catalogs).
   Root `README.md` and `docs/architecture/000-overview.md` now document
   `effigy workspace:js:prepare` for an existing clone, and the architecture
   page's test guidance is Effigy-first (`effigy test --plan`, `effigy test`,
   `effigy <catalog>/test`) instead of `cd acme-api && cargo test`.

## Shared bundle integration

`underlay-effigy-bundle#1` merged as `e680157e` and is now consumed directly.
The repo declares physical ownership paths in `[bundle.dirs]`, task-selector
aliases in `[bundle.catalogs]`, and root JavaScript workspace ownership through
`[bundle.workspace] js_root = true`.

The bundle now owns root `health`, `validate`, `qa`, `dev`, and
`workspace:js:prepare`. Bootstrap performs one frozen root install, and the
container stack isolates root `node_modules`. This repo carries zero local
lifecycle overrides. The two related papercuts are closed.

## Validation

Run from the worker worktree after syncing the merged shared bundle and one
frozen root install:

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
- `effigy validate` — Underlay's full gate and the Admin, API, and Client test
  targets pass; the aggregate exits 1 on the known Front Vitest routing
  baseline (`src/**/*.{test,spec}.ts` finds no tests because they live under
  `tests/`), already recorded in `PAPERCUTS.md`
- `git diff --check` — clean

Full root `qa` was not run because it repeats the measured `validate` failure
before its docs steps. The targeted health, validation, docs, security,
template, rollout, and workspace-shape surfaces above are current.

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

Return the PR to the orchestrator for final review. After merge, continue the
parallel `g10.006`–`g10.009` consumer rollout.
