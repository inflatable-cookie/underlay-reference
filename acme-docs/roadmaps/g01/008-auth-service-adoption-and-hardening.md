# g01.008 Auth Service Adoption And Hardening

Status: ready
Owner: repo maintainers
Updated: 2026-07-18
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

- [ ] **Refresh-replay revocation.** Adopt the foundation `SessionManager`
  refresh path (or replicate its behavior): on detecting reuse of a superseded
  refresh token, revoke the entire session family, not just reject the token.
  Preserve the legitimate concurrent-refresh race (the atomic
  `rotate_session_if_current` path must not revoke on a lost CAS).
- [ ] **Reject on refresh fingerprint mismatch** (or make the strictness
  explicit and configurable) rather than warn-and-continue.
- [ ] **Throttle 2FA.** Route TOTP + backup-code verification through
  `verify_second_factor_throttled` (per-user attempt caps against a
  `RateLimitBackend`, incrementing on failure, resetting on success).
- [ ] **Consolidate client-IP extraction.** Remove the two unvalidated
  `X-Forwarded-For` helpers; route every caller through the single trusted-proxy
  path (`acme_infra::extract_client_ip` + the `TrustedProxyConfig` extension).
- [ ] **Prefer adopting `PasswordAuthService`** for login rather than the
  hand-rolled `verify_user_credentials`, so the dummy-verify timing, lockout,
  and failure accounting come from the foundation (the in-place `dummy_verify`
  added in the audit becomes redundant once the service is adopted). If full
  adoption is judged too invasive, keep the hand-rolled path but document why and
  keep the `dummy_verify`.

## Deliverables

- [ ] refactored `acme-api/crates/auth/src/local/` that uses the foundation
  session/2FA (and ideally password) services, with the three findings closed
- [ ] tests: refresh replay revokes the family; 2FA verify is capped per user;
  a spoofed `X-Forwarded-For` cannot change the resolved client IP
- [ ] one roadmap-aligned execution log recording the batch

## Validation

- [ ] `cargo build` and `cargo test` green across the acme-api workspace
- [ ] new tests above pass
- [ ] no remaining direct `X-Forwarded-For` reads outside the trusted-proxy path
  (grep clean)

## Next

`g01.009` media and blob production path.
