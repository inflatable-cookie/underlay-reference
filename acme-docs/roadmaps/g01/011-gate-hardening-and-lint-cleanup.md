# g01.011 Gate Hardening And Lint Cleanup

Status: ready
Owner: repo maintainers
Updated: 2026-07-18
Governing refs: `acme-docs/policy/001-working-rules.md`, underlay `docs/logs/2026-07/18-100000-consumer-audit-underlay-reference.md`
Planning state: ready

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

- [ ] Add `test` and `clippy` (`cargo clippy --workspace --all-targets -D
  warnings`) to `acme-api`'s `effigy.toml` `validate` sequence.
- [ ] Clear the 13 clippy lints: box the large error variants where flagged,
  add params structs or `#[allow(clippy::too_many_arguments)]` (matching the
  existing house style in `db/src/media/`), remove the redundant closure and the
  no-effect struct update.
- [ ] Make audit-log write failures visible: at minimum log at `error` level (not
  swallow), and decide whether any mutation should fail if its audit write fails.
- [ ] Consider wiring the TS packages' `check` scripts (from `g01.010`) into
  their `effigy validate` too.

## Deliverables

- [ ] `acme-api` `effigy validate` runs build + test + clippy and is green
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] audit-log failures are logged (or enforced), not silently dropped

## Validation

- [ ] `effigy validate` (from `acme-api`) runs and passes build + test + clippy
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` returns no errors
- [ ] `cargo fmt --all --check` clean

## Next

`g01` continues; open `g01.012` when the next real milestone is scoped.
