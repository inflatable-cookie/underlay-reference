# 2026-07-19 14:00:00 - g01.009 Media And Blob Production Path

## Summary

Executed `g01.009`: wired a real production S3 blob adapter from environment
config and moved the media upload path onto the foundation's validated
helpers (underlay contract 040), deleting the hand-rolled magic-byte and
size/type re-implementation.

## Completed work

- `crates/api/src/main.rs`: production now builds an `S3Adapter` from
  `ACME_S3_BUCKET` (+ optional `ACME_S3_REGION`, `ACME_S3_ENDPOINT`,
  `ACME_S3_PUBLIC_URL_BASE`, `ACME_S3_PATH_STYLE=1`); credentials come from
  the standard AWS credential chain. A broken adapter or unusable bucket is a
  boot failure — no noop fallback in production. `ACME_ALLOW_NOOP_BLOB=1`
  remains the explicit storage-less escape hatch; the audit's panic is now a
  clean startup error naming the env vars.
- `crates/api/src/routes/admin/media/upload.rs`:
  - initiate: declared size/type pre-checked against the foundation policy
    before any rows are created; upload URL issued via
    `initiate_upload_validated` (size cap + MIME allowlist before signing).
  - finalise: `finalise_upload_verified` enforces size, allowlist, and
    magic-byte verification in one place and pins the content type to the
    validated declared value; policy rejections clean up the stored object
    and fail the version, then surface as 413/422.
  - the local `infer`-based sniffing and category-match table are deleted;
    the `infer` dependency is dropped from the workspace.
- Upload policy is `AcmeConfig::default().media` = `BlobUploadConfig`
  foundation defaults — no local allowlist re-list to drift; SVG/HTML/JS
  stay excluded, limit 50 MB.
- `.env.example` added (repo previously had none) documenting the blob
  storage variables plus core/auth/proxy/rate-limit config.
- Tests (`crates/api/src/tests/routes/admin/media_upload_validation_tests.rs`):
  oversized upload rejected at initiate; disallowed content type (SVG)
  rejected; declared-PNG-with-HTML-bytes rejected at verified finalise; real
  PNG succeeds with pinned content type; app policy asserted equal to
  foundation defaults. (Exercised through the same extension-trait calls the
  routes make, against `LocalAdapter` as a dev-only seam.)
- Test-infra: the two `check_due_reminders` handler tests serialize on a
  static async mutex — the handler scans all due tasks, so parallel runs
  enqueued reminders for each other's fixtures (flaky count assertions,
  latent until DB-backed runs).

## Validation

- `cargo build --workspace` green
- `cargo test --workspace` green twice against local Postgres 16
- `cargo fmt --all --check` clean
- no `infer::get` / manual MIME-match logic remains in `upload.rs`

## Next Task

Execute `g01.010` (TypeScript type-safety hygiene).
