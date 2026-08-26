# g01.011 Gate Hardening And Lint Cleanup

Status: done (2026-07-19)
Owner: repo maintainers
Updated: 2026-07-19
Governing refs: `acme-docs/policy/001-working-rules.md`, underlay `docs/logs/2026-07/18-100000-consumer-audit-underlay-reference.md`
Planning state: complete

## Goal

Make the reference app's CI gate actually catch what it should, and clear the
lint debt that gate was hiding.

## Why this matters now

The g08 audit found `acme-api`'s `effigy validate` runs `build` **only**. As a
direct result, `crates/infra`'s tests did not even compile (a missing
`use std::path::Path`) and shipped undetected, and 13 clippy issues accumulated
in the api crate. A reference implementation's gate should be exemplary.

## Findings this card closes

1. **Gate runs `build` only.** `acme-api/effigy.toml` has
   `validate = [{ task = "build" }]`; neither `test` nor `clippy` runs, so broken
   tests and lint regressions pass CI. (The audit fixed the broken test compile;
   the gate that let it through is still open.)
2. **13 pre-existing clippy lints** in `acme-api` (large `Err` variants —
   suggesting boxing; several `too_many_arguments`; a redundant closure; a
   no-effect struct update), surfaced once the crate compiled clean under
   `clippy -D warnings`.
3. **Fire-and-forget audit logging.** Admin mutations call `activity::log_activity`
   with `let _ =`, so audit-write failures are silently dropped — a
   security-relevant action can succeed with no audit trail and no signal.

## Scope

- [x] `validate` now runs build → test → clippy (`-D warnings`) → fmt.
- [x] 13 clippy lints cleared: `result_large_err` sites annotated with
  rationale (matching underlay-http house style — boxing `ApiError` would
  force `map_err` at every `?` site), `too_many_arguments` allowed with
  rationale, redundant closure / no-effect struct update / items-after-test-
  module fixed outright.
- [x] Audit-log failures visible: `activity::log_activity_reported` logs at
  `error` level with action/resource context; all 30 `let _ =` sites route
  through it. Decision: mutations do not fail on audit-write failure
  (availability wins), recorded in the helper's doc.
- [x] TS packages' `effigy validate` already runs their `check` task — no
  change needed.

## Deliverables

- [x] `acme-api` `effigy validate` runs build + test + clippy (+ fmt), green
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] audit-log failures are logged, not silently dropped

## Validation

- [x] `effigy validate` (from `acme-api`) passes build + test + clippy + fmt
- [x] `cargo clippy --workspace --all-targets -- -D warnings` returns no errors
- [x] `cargo fmt --all --check` clean

## Next

`g01` continues; open `g01.012` when the next real milestone is scoped.
