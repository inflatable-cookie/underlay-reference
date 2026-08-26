# Papercuts

Small, actionable friction found during agent work. Agents append entries when
they hit a solvable hurdle; they do not stop the current task to fix one.

## Open

<!-- Keep entries short. Append newest entries at the top. Do not include secrets. -->

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
