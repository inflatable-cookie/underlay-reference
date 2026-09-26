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

