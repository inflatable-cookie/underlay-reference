# Papercuts

Small, actionable friction found during agent work. Agents append entries when
they hit a solvable hurdle; they do not stop the current task to fix one.

## Open

<!-- Keep entries short. Append newest entries at the top. Do not include secrets. -->

### [ ] Parallel `bun x vitest` in `effigy test` races on bun bin linking — 2026-08-27
- Friction: root `effigy validate`/`qa` run acme-admin and acme-client vitest together. Both invoke `bun x vitest` and collide with `Failed to link rolldown/vitest/why-is-node-running: EEXIST`.
- Impact: the aggregate board fails even when each suite is green in isolation, so a CSRF-only API change cannot close the required validate/qa gates on the first pass.
- Possible fix: serialize `bun x` on the test board, or run the workspace-local vitest binary instead of `bun x`.
- Surface: `effigy test` / acme-admin + acme-client

### [ ] T3 worker launch used the Underlay worktree for an Underlay Reference handoff — 2026-08-26
- Friction: the `g09.038` worker handoff lives in Underlay Reference, but T3 started this thread in a clean Underlay worktree. `.agents.local.env` / `AGENTS_WORKTREE_CONTAINER_DIR` is also absent, so the worker had to create a second registered worktree under the existing T3 Underlay Reference container.
- Impact: the first preflight is spent proving the current root is the wrong repo instead of starting the lane.
- Possible fix: launch consumer-repo worker threads from the owning repo, and seed `.agents.local.env` with the T3 worktree container.
- Surface: T3 worker dispatch / Underlay Reference `.agents.local.env`

### [ ] Doctor rejects built-in `docs` steps as unresolved task references — 2026-08-25
- Friction: `effigy doctor` reports every `docs check ...` step in
  `docs/effigy.toml` as an unresolved `docs` task even though `docs` is a
  callable Effigy built-in.
- Impact: workspace health orientation cannot distinguish valid docs QA routing
  from a genuinely missing selector during the monorepo docs move.
- Possible fix: teach Doctor's task-reference resolver to accept built-ins in
  sequence steps, then remove any migration-only workaround after verification.
- Surface: Effigy Doctor task-reference resolution / `docs/effigy.toml`

## Closed

### [x] `bun x tsc` in acme-client health resolves TypeScript 7 and rejects `baseUrl` — 2026-08-26
- Friction: `packages/acme-client/effigy.toml` ran `bun x tsc`, which fetched TypeScript 7.0.2 instead of the package-pinned `typescript@^5.9.3`. TS 7 removes `baseUrl` and failed `acme-client/health`.
- Impact: root `effigy health` failed on an unrelated client typecheck while other lanes only changed API migration/test surfaces.
- Fix: route `check` through `bun run check` and `build` through `bun run tsc` so Effigy uses the workspace-local TypeScript 5.9.x compiler.
- Surface: `packages/acme-client/effigy.toml` health/check / TypeScript 7

### [x] Underlay Effigy bundle reuses `bundle.dirs` as a task-selector prefix — 2026-08-26
- Friction: the bundle rendered root lifecycle selectors from physical package
  paths, which fail under the `apps/*` / `packages/*` monorepo shape.
- Impact: bundle-backed consumers needed local root lifecycle overrides.
- Fix: `underlay-effigy-bundle#1` merged as `e680157e`; this repo now supplies
  `[bundle.catalogs]` aliases and carries no local lifecycle overrides.
- Surface: `underlay-effigy-bundle` `export.toml` / root `effigy.toml`

### [x] Bundle container `isolated_dirs` assume per-package `node_modules` — 2026-08-26
- Friction: per-package `node_modules` isolation did not cover a root Bun
  workspace's hoisted dependency tree.
- Impact: the container dev stack could share root `node_modules` with the host.
- Fix: `underlay-effigy-bundle#1` merged as `e680157e`; setting
  `[bundle.workspace] js_root = true` isolates root `node_modules`.
- Surface: `underlay-effigy-bundle` container defaults

### [x] Update the bundle docs-link selector — 2026-08-11
- Friction: `effigy qa:docs` and `effigy acme-docs/qa:docs` call the removed `check-links` argument.
- Impact: the aggregate docs gate fails before running otherwise valid link checks.
- Fix: `acme-docs/effigy.toml` now uses `docs check links` / `forbidden` /
  `headings` (space form). Root `qa:docs` is wired to that catalog via the
  Underlay Effigy bundle.
- Surface: Underlay Effigy bundle docs tasks / `acme-docs/effigy.toml`

### Launcher omitted sibling underlay/poodle mounts
- **Friction:** Worktree parent lacked `../underlay` and `../poodle`, so `effigy tasks` failed until mounts were added.
- **Impact:** Blocks Effigy orientation and local `file:../../underlay` installs.
- **Plausible fix:** Launcher should create the same sibling symlinks Soundcheck gets.
- **Surface:** underlay-reference t3 worktree bring-up

### Bun keeps stale file: nested Poodle metadata
- **Friction:** After removing Poodle overrides, incremental `bun install` left `file:../../poodle/...` nests under `file:` underlay/acme-ui snapshots; only `bun update` on those file deps refreshed them.
- **Impact:** Locks can look sibling-controlled despite registry top-level installs.
- **Plausible fix:** Document `bun update <file-dep>` after override removal, or fix Bun file: snapshot refresh.
- **Surface:** acme-*/bun.lock adoption

### Effigy auto-routes vitest into empty/misaimed package suites
- **Friction:** `effigy validate`/`qa` auto-detects vitest for `acme-front` and `acme-ui`. Front includes `src/**/*` while tests live under `tests/` (exit 1). UI has no tests but inherits `node_modules/.bin/vitest` transitively (exit 1).
- **Impact:** Aggregate board fails unless an app-local `passWithNoTests` exception is added; g16.013 forbids that exception.
- **Plausible fix:** Effigy should not select vitest from a transitive binary alone, and/or honor package-owned include roots; separately authorize front include=`tests/**` and UI suite exclusion/real tests.
- **Surface:** `effigy test --plan` auto-detection for acme-front/acme-ui
