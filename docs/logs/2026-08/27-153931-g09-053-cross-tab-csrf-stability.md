# g09.053 Underlay Reference Cross-Tab CSRF Stability

Date: 2026-08-27
Roadmap: Underlay `g09.053`
Handoff: `docs/handoffs/20260827-145501-g09-053-cross-tab-csrf-stability.md`

## Outcome

`GET /v1/auth/csrf-token` now reuses a non-empty incoming CSRF cookie and
mints only when that cookie is absent or empty. The JSON body and the emitted
`Set-Cookie` carry the same token. The double-submit model stays stateless:
no session store, no expiry state, no rotation protocol.

A second same-origin tab that reads the token with the browser cookie already
set receives the first tab's token. The first tab's original cookie/header
pair still passes the real CSRF middleware on a cookie-backed mutation.

Route, envelope, and cookie attributes are unchanged.
`SingleResponse<CsrfTokenResponse>` still serialises `data.csrf_token`.

Target `g01.007` planning state was not changed.

## Worker Checkout

- worktree: `/Users/tom/.t3/worktrees/underlay-reference/t3code-c1525847`
- branch: `t3code/follow-csrf-stability-handoff`
- planning base ancestor: `6af27837`
- branch base: `2cc2578b` (`origin/main` at launch, the handoff commit)

The launcher supplied a clean, registered, non-`main` worktree, so it was
accepted as-is. No second worktree was created.

## Behaviour

Before: `csrf_token` always called `Uuid::new_v7()` and set that value on
both the cookie and the body. A second tab's GET overwrote the browser-wide
cookie; the first tab's cached `X-Csrf-Token` then failed middleware.

After:

- absent cookie → mint a non-empty token
- empty cookie (`csrf_token=`) → mint
- non-empty cookie → return that value unchanged
- body token always equals the `Set-Cookie` pair
- cookie attributes stay `SameSite=Lax; Path=/; Max-Age=<refresh>; Secure`
  under `AuthCookieConfig::new()`

`extract_csrf_token` already skipped empty values; issuance now uses that
same parse. `csrf_token_to_issue` is the shared decision. The production
handler takes request headers and `State<AuthCookieConfig>` via `FromRef`
from `AppState`, so tests mount the real handler without standing up auth,
email, or blob services.

## Changed Surfaces

- `apps/acme-api/crates/api/src/routes/shared/auth/mod.rs` — inspect request
  cookies; mint only when absent/empty
- `apps/acme-api/crates/api/src/state.rs` — `FromRef<AppState> for
  AuthCookieConfig`
- `apps/acme-api/crates/api/src/tests/routes/csrf_tests.rs` — issuance and
  two-tab middleware proof through `get(csrf_token)` and
  `csrf_protection_middleware`
- `PAPERCUTS.md` — parallel `bun x vitest` link race on an unprepared host

No client, public URL, session schema, or Underlay planning edits.

## Proof

Focused tests drive the production `csrf_token` handler over HTTP, then the
real CSRF middleware on `POST /v1/auth/refresh` with a refresh cookie:

1. tab A GET with no cookie → token A plus `Set-Cookie`
2. tab B GET with that cookie → token A again, same cookie value
3. tab A POST with cookie A + `X-Csrf-Token: A` → 200, not 403

Also: absent cookie mints; empty cookie mints rather than echoing blank;
reuse preserves cookie attributes; body and cookie stay aligned.

Effigy `test` is the full workspace board and cannot select the named
regression, so the focused run used:

`cargo test --manifest-path apps/acme-api/Cargo.toml -p acme-api --lib csrf`

That is a recorded fallback, not a second test surface.

## Validation

All run from the worker worktree.

- `effigy tasks`
- focused CSRF lib tests — 4 new tests plus the matching middleware filter,
  all green
- `effigy acme-api/health` — passed
- `effigy acme-api/clippy` — passed (`-D warnings`)
- `effigy test --plan` — `targets: 3` (`acme-admin`, `acme-api`,
  `acme-client`); `acme-front` still has no default suite
- `effigy workspace:js:prepare` — frozen root install, required before
  host-side JS validate
- first `effigy validate` without prepare — failed: parallel `bun x vitest`
  on acme-admin and acme-client raced with `Failed to link
  rolldown/vitest/why-is-node-running: EEXIST`
- `effigy validate` after prepare — passed
- `effigy qa` — passed
- `effigy acme-docs/qa:docs` — passed
- `effigy acme-docs/qa:northstar` — passed
- `git diff --check` — clean

No active CSRF wording in target docs still claimed that every GET mints.
`g01.002` still has a stale unchecked "add CSRF endpoint" item; that is
historical `g01` planning and was left untouched.

## Residual Risk

- host `effigy validate`/`qa` still need one frozen `workspace:js:prepare`
  first; an unprepared tree lets `bun x vitest` race on bin linking
- reuse does not rotate a stolen cookie. That is the settled stateless
  policy, not a gap in this card

## Next Task

PR is open for orchestrator exact-head review. Do not merge. After an
operator-authorised merge, the orchestrator promotes Underlay `g09.054`
fleet closeout.
