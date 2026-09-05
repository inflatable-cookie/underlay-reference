# Papercuts

Small, actionable friction found during agent work. Agents append entries when
they hit a solvable hurdle; they do not stop the current task to fix one.

## Open

<!-- Keep entries short. Append newest entries at the top. Do not include secrets. -->

### JS Underlay pin PR left Cargo crates on the previous tag
- **Friction:** PR #16 moved JavaScript Underlay declarations and `bun.lock` to released `v0.9.8` / peel `97a26d9`, but `apps/acme-api/Cargo.toml` and `Cargo.lock` stayed on `v0.9.7` / `8a7ce84b`.
- **Impact:** After the Poodle 0.3.0 consumer PR merged, the workspace still had two Underlay identities; a follow-up pin was required before the lane could claim every app manifest.
- **Plausible fix:** Treat Cargo git tags as part of the same Underlay pin as `package.json` whenever the consumer has a Rust app.
- **Surface:** g16.109 / `apps/acme-api/Cargo.toml` / consumer adoption wave

### Bun cannot resolve an annotated git tag until its install cache is cleared
- **Friction:** `bun install` against `git+ssh://…underlay.git#v0.9.6` failed with `no commit matching "v0.9.6"` even though the annotated tag peels to `4f6d7552` and Cargo resolved the same tag immediately. `bun pm cache rm` then `bun install` succeeded.
- **Impact:** A required JS lock refresh looks like a missing release until the cache is dropped.
- **Plausible fix:** Peel annotated tags on git+ssh deps, or fetch tags when the cached repo already exists.
- **Surface:** Bun git dependencies / annotated tags / `bun.lock` refresh

### `effigy container up` cannot start Postgres without an interactive vault TTY
- **Friction:** Worker DB oracles need the stack's Postgres, but `effigy container up` fails with `container secrets require an unlocked vault passphrase and secret input requires an interactive TTY`.
- **Impact:** Media composition tests cannot use the Effigy-owned database; the worker had to stand up a throwaway `postgres:16` and apply migrations on the host.
- **Plausible fix:** Allow a non-interactive unlock from an already-initialized session vault, or a Postgres-only bring-up that does not demand secret input.
- **Surface:** `effigy container up` / secrets vault / worker DB tests

### Root `effigy validate` fails on the sibling Underlay test suite
- **Friction:** The bundle-provided root `validate` sequence fans out into the mounted `underlay` catalog and runs Underlay's own vitest suite. One pre-existing failure there (`ts/tests/tools/workspace-shape.test.ts`, 1 of 813) turns this repo's headline validation command red for reasons a consumer worker cannot fix or is forbidden to touch.
- **Impact:** `effigy validate` is unusable as this repo's own gate; each lane has to fall back to the six per-catalog `validate` tasks and explain the red root result.
- **Plausible fix:** Extend the existing `[test] exclude_catalogs = ["poodle", "underlay"]` posture to the bundle's root `validate`/`qa` fan-out, so mounted sibling catalogs stay context-only.
- **Surface:** root `effigy validate` / Underlay Effigy bundle lifecycle fan-out

### Northstar Rust recorder needs audit-unique limitation keys but records are immutable
- **Friction:** Each unit assessment carries its own limitations, so the natural key for a per-unit condition (e.g. `rust-msrv-unresolved`) repeats across units. `finalize` then rejects the whole audit with `lifecycle.limitation_key_duplicate`, and because `assess` refuses to overwrite an existing `assessment.json`, the only recovery is to delete the audit record and re-run init → assess → collect → complete from scratch.
- **Impact:** A naming choice that is only visible at the last step costs a full re-run, including re-collecting every evidence record.
- **Plausible fix:** Validate key uniqueness at `assess` time, or namespace limitation keys by unit the way `finding:<id>` keys already are.
- **Surface:** `northstar-rust-quality` lifecycle (`assess`/`finalize`)

### `RUST-SLOP-001` has no recorder representation
- **Friction:** The Rust audit mode requires a *total* exact-forwarder candidate ledger with a recorded disposition per candidate, but the rule is `prototype`/`evaluation_only`, so the recorder rejects it as a verdict rule (`ledger.verdict_rule_invalid`) and as a finding rule (`ledger.finding_rule_invalid`).
- **Impact:** The ledger the mode mandates cannot live in `result.json`; it has to be carried in free-text attestations and the closeout log, where nothing checks it for completeness.
- **Plausible fix:** Accept evaluation-only rules as report-only findings, or add an explicit ledger section to the assessment schema.
- **Surface:** `northstar-rust-quality` ledger validation / `references/modes/rust-quality-audit.md`

### Installed Northstar skill copy silently drifts from source
- **Friction:** `~/.agents/skills/northstar` was stale against `~/Dev/projects/northstar/skills/northstar` (missing the `dbce3856` Rust evidence-collection repair). Nothing in the installed copy records which source commit it came from; the drift only surfaced because `verify-install` reported a payload hash mismatch.
- **Impact:** A worker told to pin an exact Northstar hash can silently run older mode files, projections and schemas from the installed copy.
- **Plausible fix:** Write the source commit into the installed skill directory on install and have the router compare it when a handoff pins a hash.
- **Surface:** Northstar skill install / `~/.agents/skills/northstar`

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

## Closed

### [x] T3 worker launch used the Underlay worktree for an Underlay Reference handoff — 2026-08-26
- Friction: the `g09.038` worker handoff lives in Underlay Reference, but T3 started this thread in a clean Underlay worktree. `.agents.local.env` / `AGENTS_WORKTREE_CONTAINER_DIR` is also absent, so the worker had to create a second registered worktree under the existing T3 Underlay Reference container.
- Impact: the first preflight is spent proving the current root is the wrong repo instead of starting the lane.
- Fix: Northstar PR 8 (`1840c9f6d4f7127240622a09e462b06adc094971`) requires the owning repo's absolute handoff path for operator-facing dispatch. `AGENTS.md` states that rule; do not start Underlay Reference lanes from an Underlay-relative `docs/handoffs/…` lookup. `.agents.local.env` is gitignored; seed `AGENTS_WORKTREE_CONTAINER_DIR` only after asking, never commit the env file.
- Surface: T3 worker dispatch / Underlay Reference `.agents.local.env`
- Closed: 2026-08-30 (papercuts wave 19).

### [x] Doctor rejects built-in `docs` steps as unresolved task references — 2026-08-25
- Friction: `effigy doctor` reported every `docs check ...` step in
  `docs/effigy.toml` as an unresolved `docs` task even though `docs` is a
  callable Effigy built-in.
- Impact: workspace health orientation could not distinguish valid docs QA
  routing from a genuinely missing selector during the monorepo docs move.
- Fix: proved clean against Effigy `v0.12.1+local.834a4bd`. Full
  `effigy doctor` no longer emits task-reference findings for the
  `docs/effigy.toml` `docs check ...` steps; no migration-only workaround
  was present to remove. Remaining doctor errors are unrelated (vault/health,
  unsupported keys, scan markers).
- Surface: Effigy Doctor task-reference resolution / `docs/effigy.toml`

### [x] Reference runtime docs misstate database storage shape — 2026-08-26
- Friction: earlier wording said PostgreSQL persists under repo-local
  `.effigy/runtime/data/postgres`, while the live store is the named
  `underlay-reference-dev-postgres-data` volume.
- Impact: agents could misidentify the destructive boundary when preparing
  local state or reset proof.
- Fix: active runtime wording in README already names
  `underlay-reference-dev-postgres-data` and states older
  `.effigy/runtime/data/` bind-mounts are not migrated. Hunt of active
  usage/runtime guides found no remaining live claim of the bind-mount path;
  historical handoff/log wording left untouched. Original filing lived in
  Underlay; closed here on consumer proof.
- Surface: Underlay Reference README runtime notes / named volumes

### [x] Parallel `bun x vitest` in `effigy test` races on bun bin linking — 2026-08-27
- Friction: root `effigy validate`/`qa` run acme-admin and acme-client vitest together. Both invoke `bun x vitest` and collide with `Failed to link rolldown/vitest/why-is-node-running: EEXIST`.
- Impact: the aggregate board fails even when each suite is green in isolation, so a CSRF-only API change cannot close the required validate/qa gates on the first pass.
- Fix: route vitest through `bun run test` and package-local `[test.suites.vitest]` in `apps/acme-admin/effigy.toml` and `packages/acme-client/effigy.toml` so Effigy uses the workspace-installed binary instead of `bun x`.
- Surface: `effigy test` / acme-admin + acme-client

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
