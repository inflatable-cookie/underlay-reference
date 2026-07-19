# 2026-07-19 12:00:00 - g01.008 Auth Service Adoption And Hardening

## Summary

Executed `g01.008`: aligned the reference app's hand-rolled auth with the
foundation's hardened posture (underlay contract 030). Closed the three g08
consumer-audit findings: refresh-replay family revocation, unthrottled 2FA,
and spoofable client-IP helpers. Also repaired latent test-infra breakage
found while validating against a real Postgres.

## Completed work

- `crates/auth/src/local/session.rs`: refresh now mirrors the foundation
  `SessionManager` path — reuse of a superseded refresh token (stale token
  fingerprint or id/version mismatch) revokes the whole session family;
  rotation commits through an atomic `rotate_session_if_current` CAS so the
  loser of a legitimate concurrent-refresh race is rejected *without*
  revocation. The blind `update_session` write is gone.
- Client fingerprint (IP/User-Agent) mismatch on refresh now rejects by
  default (`auth.refresh_fingerprint_strict`, layered-config knob, default
  `true`; set `false` to restore warn-and-continue).
- `crates/auth/src/local/totp.rs`: TOTP + backup-code verification routed
  through the foundation's `verify_second_factor_throttled` — per-user
  attempt caps (`2fa:<user_id>` key, `max_totp_attempts`/hour) against the
  shared rate-limit backend, incrementing on failure, resetting on success.
- `crates/api/src/routes/shared/auth/mod.rs`: deleted the two unvalidated
  `X-Forwarded-For` helpers; every caller (register/login/login_start/
  login_finish) now resolves the client IP through
  `acme_infra::extract_client_ip` + `TrustedProxyConfig`;
  `login_client_fingerprint` takes the trusted-proxy config.
- `PasswordAuthService` adoption evaluated and deliberately not taken: the
  hand-rolled `verify_user_credentials` records per-IP login attempts and
  feeds the security-alert pipeline, which the foundation's
  `PasswordAuthRepository` seam cannot express. Decision + rationale recorded
  in a code comment; `dummy_verify` timing equalizer retained.
- Tests (`crates/auth/src/tests/local_hardening_tests.rs`, DB-gated):
  refresh replay revokes the family; sequential refresh chain stays valid;
  2FA is rate limited per user and does not leak across users. Route-level
  unit tests assert a spoofed `X-Forwarded-For` cannot change the resolved
  IP or login fingerprint.
- Test-infra repairs (latent, exposed by running with a real DATABASE_URL):
  - `test-utils/db.rs`: per-test pools replace the process-global pool that
    died with the first test's runtime (`block_in_place` panic on
    single-thread runtimes; "Tokio context is being shutdown" after).
  - `test-utils` `shared_runtime()` + jobs/scheduled-tasks handler tests now
    share one runtime, since handlers read the process-global `DB_POOL`.
  - `test-utils/fixtures.rs`: aligned with current schema — `auth.users` has
    no `password_hash` column; `acme.projects.description` is JSONB.

## Validation

- `cargo build --workspace` green
- `cargo test --workspace` green against a local Postgres 16 (all
  previously-skipped DB-gated tests now exercised; new hardening tests pass)
- `cargo fmt --all --check` clean
- `grep -rn "x-forwarded-for" crates/api/src` — only the trusted-proxy path
  and its tests remain

## Next Task

Execute `g01.009` (media and blob production path).
