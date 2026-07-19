# g01.009 Media And Blob Production Path

Status: done (2026-07-19)
Owner: repo maintainers
Updated: 2026-07-19
Governing refs: underlay `docs/contracts/040` (media and upload enforcement), underlay `docs/logs/2026-07/18-100000-consumer-audit-underlay-reference.md`
Planning state: ready

## Goal

Close the reference app's media story: adopt the foundation's validated blob
helpers instead of reimplementing validation, and wire a real production blob
adapter so the "production is still TODO" gap is resolved.

## Why this matters now

The g08 audit patched the most dangerous symptom — production silently used a
`NoopAdapter`, so uploads were accepted then discarded (data loss). That is now
**fail-closed**: `acme-api/crates/api/src/main.rs` panics in production unless
`ACME_ALLOW_NOOP_BLOB=1` is set. That is a guardrail, not a solution — the app
still has no real production blob path, and it reimplements upload validation the
foundation now provides.

## Findings this card closes

1. **No production blob adapter.** `main.rs` only constructs a MinIO `S3Adapter`
   in development; production has no real storage (guarded by the fail-closed
   panic from the audit).
2. **Upload validation reimplemented.** `routes/admin/media/upload.rs` calls the
   raw adapter (`state.blob_adapter.initiate_upload`) and hand-rolls magic-byte
   sniffing (`infer::get` + manual declared-vs-detected MIME comparison) instead
   of the foundation's `initiate_upload_validated` / `finalise_upload_verified`
   (which enforce the content-type allowlist, size limits, and magic-byte
   verification in one place).

## Scope

- [x] Production `S3Adapter` built from `ACME_S3_*` env config in `main.rs`
  (credentials via the AWS default chain); `ACME_ALLOW_NOOP_BLOB=1` kept as
  the explicit storage-less escape hatch; the audit panic replaced with a
  clean fail-closed startup error naming the env vars.
- [x] Upload path routed through `initiate_upload_validated` and
  `finalise_upload_verified`; the duplicated `infer`-based sniffing deleted
  (and the `infer` dependency dropped).
- [x] Policy is `BlobUploadConfig` foundation defaults (asserted by test):
  SVG/HTML/JS excluded, 50 MB limit — no local re-list.

## Deliverables

- [x] production-capable blob adapter construction with documented env vars
  (`.env.example` added)
- [x] media upload path routed through the validated foundation helpers
- [x] tests: mismatched bytes rejected; oversized rejected; allowed image
  succeeds (`media_upload_validation_tests.rs`)

## Validation

- [x] `cargo build` and `cargo test` green (local Postgres 16)
- [x] no local magic-byte / content-type re-implementation remains in
  `upload.rs`

## Next

`g01.010` type-safety and TypeScript hygiene.
