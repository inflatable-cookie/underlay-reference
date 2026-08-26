# g01.008 Auth Service Adoption And Hardening

Status: done (2026-07-19)
Owner: repo maintainers
Updated: 2026-07-19
Governing refs: `acme-docs/roadmaps/g01/002-security-audit.md`, `acme-docs/architecture/product-guardrails.md`, underlay `docs/contracts/030` (auth session and token boundary), underlay `docs/logs/2026-07/18-100000-consumer-audit-underlay-reference.md`
Planning state: ready

## Goal

Move the reference app's hand-rolled local auth (`acme-api/crates/auth/src/local/`)
onto the foundation's hardened auth services so it *demonstrates* the g08 posture
instead of diverging from it. Today the app reimplements login, TOTP verification,
and refresh, and in doing so loses three protections the foundation already
ships.

## Why this matters now

The g08 consumer audit (2026-07-18) found the reference app hand-rolls auth and
misses foundation hardening. The login user-enumeration timing hole was patched
in place (a `dummy_verify` on the miss paths), but the deeper gaps below are
architectural and were deferred to this card. As the *reference* implementation,
acme-api is what downstream apps copy — divergence here propagates.

## Findings this card closes

1. **Refresh-replay: no session-family revocation.** `local/session.rs` returns
   `TokenInvalid` on a replayed/superseded refresh token but does **not** revoke
   the token family. The foundation's `SessionManager` refresh path
   auto-revokes the whole family on reuse detection (RFC 6819 / OAuth BCP).
   A fingerprint mismatch on refresh only logs a warning; it does not reject.
2. **2FA verification is not throttled.** `local/totp.rs` calls raw
   `verify_totp_with_replay_protection` (underlay `TotpService`), not
   `verify_second_factor_throttled`. Only the outer login rate-limit applies, so
   per-user 2FA/backup-code guessing is under-throttled.
3. **Duplicate spoofable X-Forwarded-For helpers.** `routes/shared/auth/mod.rs`
   has the correct trusted-proxy path (`acme_infra::extract_client_ip`) *and*
   two helpers (`extract_client_ip` at ~109-133, `login_client_fingerprint` at
   ~275-293) that split `X-Forwarded-For` with no trust validation, used by the
   register/login paths. These are spoofable and contradict the trusted-proxy
   path in the same file.

## Scope

- [x] **Refresh-replay revocation.** Replicated the foundation `SessionManager`
  refresh behavior in `local/session.rs`: reuse of a superseded refresh token
  (stale fingerprint or id/version mismatch) revokes the entire session family;
  rotation commits through an atomic `rotate_session_if_current` CAS, and a
  lost CAS (legitimate concurrent refresh) rejects without revoking.
- [x] **Reject on refresh fingerprint mismatch** — strict by default via the
  layered-config knob `auth.refresh_fingerprint_strict` (set `false` to
  restore warn-and-continue).
- [x] **Throttle 2FA.** TOTP + backup-code verification routed through
  `verify_second_factor_throttled` (per-user `2fa:<user_id>` key,
  `max_totp_attempts`/hour against the shared rate-limit backend).
- [x] **Consolidate client-IP extraction.** Both unvalidated `X-Forwarded-For`
  helpers removed; all callers use `acme_infra::extract_client_ip` +
  `TrustedProxyConfig`.
- [x] **`PasswordAuthService` adoption evaluated, hand-rolled path retained**
  with rationale documented in code: the local path records per-IP login
  attempts and feeds the security-alert pipeline, which the foundation's
  `PasswordAuthRepository` seam cannot express. `dummy_verify` kept.

## Deliverables

- [x] refactored `acme-api/crates/auth/src/local/` with the three findings
  closed (session/2FA via foundation services/behavior; password path retained
  by documented decision)
- [x] tests: refresh replay revokes the family; 2FA verify is capped per user;
  a spoofed `X-Forwarded-For` cannot change the resolved client IP
- [x] execution log:
  `acme-docs/logs/2026-07/19-120000-g01-008-auth-service-adoption-and-hardening.md`

## Validation

- [x] `cargo build` and `cargo test` green across the acme-api workspace
  (validated against a local Postgres 16)
- [x] new tests above pass
- [x] no remaining direct `X-Forwarded-For` reads outside the trusted-proxy path
  (grep clean)

## Next

`g01.009` media and blob production path.
