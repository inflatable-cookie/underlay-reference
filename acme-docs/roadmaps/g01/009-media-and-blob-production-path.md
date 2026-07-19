# g01.009 Media And Blob Production Path

Status: ready
Owner: repo maintainers
Updated: 2026-07-18
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

- [ ] Wire a real production `S3Adapter` from environment config (bucket,
  region, endpoint, credentials) in `main.rs`; keep `ACME_ALLOW_NOOP_BLOB=1` as
  the explicit storage-less escape hatch, and remove the panic once a real
  adapter path exists.
- [ ] Replace the hand-rolled upload validation with the foundation's
  `initiate_upload_validated` and `finalise_upload_verified`; delete the
  duplicated sniffing logic in `upload.rs` once the validated helpers cover it.
- [ ] Confirm the SVG/HTML/JS content-type exclusions and size limit come from
  the foundation defaults (not a local re-list that can drift).

## Deliverables

- [ ] production-capable blob adapter construction with documented env vars
  (add to `.env.example`)
- [ ] media upload path routed through the validated foundation helpers
- [ ] tests: an upload whose bytes do not match the declared content type is
  rejected; an oversized upload is rejected; an allowed image succeeds

## Validation

- [ ] `cargo build` and `cargo test` green
- [ ] no local magic-byte / content-type re-implementation remains in
  `upload.rs` once the validated helpers are adopted

## Next

`g01.010` type-safety and TypeScript hygiene.
