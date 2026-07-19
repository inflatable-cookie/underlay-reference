# 2026-07-19 17:00:00 - g01.011 Gate Hardening And Lint Cleanup

## Summary

Executed `g01.011`: the acme-api CI gate now runs build + test + clippy +
fmt, the 13 pre-existing clippy lints are cleared, and audit-log write
failures are surfaced instead of silently dropped. This closes the g08
consumer-audit tranche (`g01.008`-`g01.011`).

## Completed work

- `acme-api/effigy.toml`: `validate` = build → test (`cargo test
  --workspace`) → clippy (`cargo clippy --workspace --all-targets -D
  warnings`) → fmt (`cargo fmt --all --check`). Broken tests and lint
  regressions can no longer pass the gate.
- Clippy debt cleared (13 lints):
  - `result_large_err` (×10 incl. one closure): annotated with
    `#[allow(clippy::result_large_err)]` + rationale, matching the
    underlay-http house style — `ApiError` is the canonical error type and
    boxing it would force `map_err` noise at every `?` call site.
  - `too_many_arguments` on `sync_nightfire_block_media_usage`: allowed
    with rationale (per-call-site error-code/message pairs; matches
    `db/src/media` house style).
  - redundant closure (`map(user_role_level)`), no-effect struct update
    (`jobs.rs` list filter), and items-after-test-module fixed outright.
- Audit logging: new `acme_db::activity::log_activity_reported` logs write
  failures at `error` level with action/resource context; all 30
  fire-and-forget `let _ = log_activity(...)` sites route through it.
  Decision: mutations do not fail when the audit write fails (availability
  wins), but the failure is now visible and alertable, never silent.
- TS packages: their `effigy validate` already runs the type check
  (`check` task) — no change needed (the npm-level `check` scripts were
  added in `g01.010`).

## Validation

- `effigy validate` (from `acme-api`) runs and passes build + test +
  clippy + fmt against local Postgres 16
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- `grep "let _ = activity::log_activity"` → 0 occurrences

## Next Task

The g08 consumer-audit tranche is complete. Open `g01.012` when the next
real milestone is scoped (the `g01.007` retained-surface thread remains
live).
