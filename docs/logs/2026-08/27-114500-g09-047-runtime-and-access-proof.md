# g09.047 Underlay Reference Runtime And Access Proof

Date: 2026-08-27
Roadmap: Underlay `g09.047`
Handoff: `docs/handoffs/20260827-102002-g09-047-runtime-access-proof.md`

## Outcome

Adopted released Underlay `v0.9.5`, completed the env and required-secret
authority, implemented the three settled fail-closed startup policies, split
the router into explicit runtime/shared/front/admin families, and centralised
auth client-IP resolution on the peer-aware request context.

No public URL changed. All 95 registered paths are byte-identical to the
pre-change builder, verified mechanically against `HEAD` before and after the
split.

Target `g01.007` planning state was not changed.

## Worker Checkout

- worktree: `/Users/tom/.t3/worktrees/underlay-reference/t3code-9419e889`
- branch: `t3code/runtime-access-proof`
- planning base ancestor: `854e5ad2`
- branch base: `e4235876` (`origin/main` at launch, the handoff commit)

The launcher supplied a clean, registered, non-`main` worktree, so it was
accepted as-is. No second worktree was created.

## Released Dependency Adoption

Every active Underlay declaration moved from `v0.9.4` to `v0.9.5`: 28 Cargo
workspace dependencies plus four JavaScript manifests. Both locks were
regenerated narrowly and resolve tag commit `8ffafb92`; the Cargo lock diff
touches only `underlay-*` entries.

The root manifest takes `@inflatable-cookie/underlay` as a devDependency. Bun
1.3's isolated install layout links binaries into each workspace member rather
than the root, so without a root edge the released conformance binaries are
not resolvable from the workspace root.

Root conformance no longer executes sibling Underlay source:

- `qa:workspace-shape` runs `./node_modules/.bin/underlay-workspace-shape`
- `qa:env-authority` is new and runs the released env-authority binary
- `qa:conformance` sequences both

`qa:security` and `qa:templates` still call sibling shell scripts. Those are
not published in the `v0.9.5` package (`files: ["ts"]`), so they could not be
cut over within this roadmap's scope. Recorded, not hidden.

## Env And Secret Authority

`config/env-manifest.txt` (44 keys) is now the complete environment surface
every runtime process may read, grouped by concern with each key's condition
stated inline. `config/required-secrets.txt` (4 keys) is the startup-critical
subset. App behavior is deliberately absent: it lives in typed config. See
Review Round 1 below.

Root `effigy.toml` carries repo-owned `[secrets.keys]` declarations so the
unconditional startup keys are actually required and targeted through Effigy,
not only documented.

`apps/acme-api/.env.example` is removed and the contradictory docs are
repaired. The root README previously rejected `.env` files in one section and
instructed operators to keep runtime wiring in `apps/acme-api/.env` in
another; the architecture overview pointed at the deleted example file. Both
now agree with the no-`.env` contract.

Two documented keys did not exist in code. The README listed
`BLOB_ADAPTER`, `BLOB_S3_BUCKET`, `BLOB_S3_ENDPOINT_URL`,
`BLOB_S3_PUBLIC_URL_BASE`, and `BLOB_S3_PRESIGN_URL_BASE`; the runtime reads
`ACME_S3_*`. The manifest and docs record what the code actually reads.

### Conditional secrets

The released checker's format is a flat key list with `#` comments, and its
only cross-file rule is that every required secret appears in the manifest. It
cannot express conditionality.

Rather than invent a format or claim unused providers are mandatory,
`required-secrets.txt` lists only `DATABASE_URL` and the JWT keypair
unconditionally, plus `ENCRYPTION_KEY` with its deployed-only condition stated
in a comment. Redis, SMTP, SES/AWS, Google OAuth, and object-store credentials
stay out of it, with their activating condition recorded against each key in
the manifest. This is honest under the shipped checker and needs no format
change.

## Settled Policy Implementation

The three owner decisions are encoded as one policy seam in
`acme_infra::config`, so a consumer copying this repo finds them together.

`startup_posture()` is the single place the deployed boundary is decided:
`local`, `effigy`, and `test` may warn; everything else is fatal, including
any unrecognised name, which `Environment::parse` resolves to production.

**This was the substantive defect.** Both existing checks derived the boundary
from `is_development()`, which returns true for `Dev`. A deployed `dev`
environment could therefore boot with insecure cookies and no encryption key.
There is a named regression test pinning that `Dev.is_development()` is true
while `startup_posture(Dev)` is fatal.

| Violation | Before | After |
|-----------|--------|-------|
| Malformed layered config | warn, silently use code defaults | fatal outside local/effigy/test |
| `COOKIE_SECURE=false` | warn, and only outside `is_development()` | fatal outside local/effigy/test |
| CSRF disablement | per-request env read, skips all checks | resolved at bootstrap, rejected when deployed |
| Missing `ENCRYPTION_KEY` | warn inside `is_development() \|\| is_local_dev()` | fatal outside local/effigy/test |

`AppBehaviorConfig::load()` and `AppConfig::from_env()` now return `Result`.
Both binaries refuse to start on a malformed stack when deployed. The
migration binaries' `ensure_database_url()` reports the failure and lets the
caller fail on the missing URL rather than masking it.

### CSRF

`CSRF_PROTECTION` was read per request, and any falsy value bypassed every
mutation check. It is now resolved once by
`acme_infra::resolve_csrf_protection()`, which rejects the disablement in
every deployed environment, and handed to the middleware as a narrow
`CsrfState`. A later environment change cannot weaken a running deployment.

Authenticated passkey registration is no longer exempt.
`/v1/auth/passkeys/register/start` and `/finish` require a logged-in user, so
they are cookie-backed mutations. Only passkey *login* start/finish remain
bootstrap exemptions, alongside register, login, password reset, and the CSRF
token fetch.

No client change was needed: `packages/acme-client` already injects the CSRF
header on every mutating request when using cookies.

## Router Topology

`routes/mod.rs` registered every family in one flat builder. It is now a
shallow root builder merging four explicit family builders:

| Family | Builder | Auth | CSRF | `X-Api-Version` |
|--------|---------|------|------|-----------------|
| runtime | `routes/runtime.rs` | none | none | exempt |
| shared | `routes/shared/router.rs` | bootstrap or `AuthenticatedUser` | on cookie mutations | required |
| front | `routes/front/router.rs` | `AuthenticatedUser` | on cookie mutations | required |
| admin | `routes/admin/router.rs` | `AdminUser` | on cookie mutations | required |

Health moved out of `shared` into `runtime`, where the contract puts it.
Product task routes moved into the new `front` family. `project_description`
stays where it is: it registers no routes and is shared handler support for
both front and admin, so it belongs to no family.

Cross-cutting policy moved to `routes/middleware.rs` and is layered above the
merged router, so it applies uniformly rather than per family.

`build_runtime_router` is generic over state — the runtime family needs none —
which is also what makes it directly testable.

## Version Posture

Path versioning stays the baseline. This app has *declared* the optional
header: `packages/acme-client` sends `X-Api-Version` on every request from a
value in the root config stack, and the server validates it.

The middleware applied it to everything under `/v1/`, including `/v1/health`,
so a liveness probe had to know the app's version vocabulary.
`is_versioned_business_path()` now exempts the runtime family while keeping
the header consistent across shared, front, and admin — the contract's
"declared headers apply to business families and exclude runtime". No URL or
header cutover.

## Client IP And Proxy Trust

Bootstrap already installed Underlay's peer-aware `RequestContext` extension
and `ConnectInfo`, but the auth handlers still called the app-local
header-based `acme_infra::extract_client_ip`. That left a second, weaker trust
boundary reachable by exactly the handlers that feed lockout, rate limiting,
session fingerprints, and audit.

Auth handlers now take `RequestContext` and resolve through one `client_ip()`
helper. The app-local `acme_infra::network` module and its duplicate
`TrustedProxyConfig` are removed; bootstrap declares the topology once with
`underlay_http::TrustedProxyConfig::from_env()`, which fail-closes to the
socket peer on an unrecognised mode. `AppState` drops `trusted_proxy_config`.

That moves the proxy env surface from `TRUST_PROXY_HEADERS` / `TRUSTED_PROXIES`
to `TRUSTED_PROXY` / `TRUSTED_PROXY_HOPS`. No environment in this workspace
set the old keys — the Effigy bundle injects only `ENVIRONMENT` — so no
runtime behavior changes. The manifest records the new keys.

## Focused Proof

45 new tests, all direct-router or pure-policy.

- **17 config policy** (`crates/infra`): all six environment names plus an
  unrecognised one; cookie and CSRF policy in both directions; malformed
  config fatal when deployed and warn-and-continue in local; the six migrated
  behavior keys declared legacy and proved inert; typed API-version and cookie
  overlays applied; an unserviceable default version falling back.
- **11 CSRF** (`crates/api`, direct router with the real middleware): passkey
  registration start and finish rejected without a token, on a mismatched
  token, and on a header without a cookie; accepted with a matching token;
  other authenticated cookie mutations protected; bearer-only clients not
  forced through it; safe methods untouched; bootstrap still exempt; the
  disabled path only local/effigy/test can reach.
- **9 client IP** (direct router with the real extractor and a real socket
  peer): spoofed `X-Forwarded-For`, `X-Real-IP`, and `CF-Connecting-IP` all
  ignored without a declaration and honoured only under the matching one;
  neither login nor session fingerprint shifts under a spoofed header;
  declared proxies still fall back to the peer when the header is absent.
- **8 runtime family**: runtime paths exempt from the version header, all
  three business families still carrying it, health answering under an
  unsupported version while a business route rejects it, and OpenAPI exposure
  following `include_docs`.

## Validation

All run from the worker worktree; all green.

- `effigy tasks`
- `effigy workspace:js:prepare` — one frozen root install, no changes
- `effigy qa:workspace-shape` — released binary, passed
- `effigy qa:env-authority` — released binary, passed
- `effigy secrets list` — `auth_jwt_private_key` and `auth_jwt_public_key`
  now `required: true`; `database_url` declared and targeted
- `cargo test --workspace` — 133 passed, 0 failed, 2 ignored (DB-gated)
- `effigy acme-api/health`, `acme-api/clippy -D warnings`, `acme-api/fmt`
- `effigy test --plan` — `targets: 3` (`acme-admin`, `acme-api`,
  `acme-client`); `acme-front` is listed with `available-suites: vitest` but
  no default suite and `command: <none>`, because
  `apps/acme-front/effigy.toml` sets `[test.suites.vitest] default = false`
- `effigy health` — exit 0
- `effigy validate` — exit 0
- `effigy qa` — exit 0
- `effigy acme-docs/health`, `validate`, `qa:docs`, `qa:northstar` — exit 0
- active `v0.9.4` search — no hits
- active `.env` search — three hits, all statements that `.env` is not part of
  the contract
- `git diff --check` — clean

## Review Round 1

Orchestrator requested changes on `9e582db8`. Three contract gaps, all
repaired on this branch.

### 1. Secret authority was static, not runnable

`config/required-secrets.txt` declared `DATABASE_URL` and the JWT keypair as
unconditional startup authority, but `effigy --json secrets list` omitted
`database_url` entirely and resolved both JWT keys `required: false` — the
bundle's shared declarations, never narrowed by this repo. The README removed
`.env` and pointed at the vault without giving a bootstrap path.

Root `effigy.toml` now carries repo-owned `[secrets.keys]` declarations.
Repo-local entries merge additively with the bundle's and override by name,
verified against `effigy --json secrets list`:

| Key | Before | After |
|-----|--------|-------|
| `auth_jwt_private_key` | `required: false` (bundle) | `required: true` |
| `auth_jwt_public_key` | `required: false` (bundle) | `required: true` |
| `encryption_key` | `required: false`, no condition stated | `required: false`, deployed condition in the description |
| `database_url` | absent | declared and targeted, local exception stated |

The JWT keys are safe to mark required because the bundle's
`generate-dev-secrets.rhai` hook produces both on first secrets-required task
startup, so `effigy dev` still works from a clean clone.

`database_url` stays `required: false` for the stated local exception: local
and effigy resolve it from the committed non-secret `config/effigy.toml`
overlay, and the generate hook does not produce it. Marking it required would
block the dev loop for a value the committed overlay already supplies.
Deployed injection remains required, recorded in the declaration description,
in `config/env-manifest.txt`, and in `config/required-secrets.txt`.

Effigy's `required` flag is one boolean and cannot express "required when
deployed" or "required when this adapter is selected", so those conditions
live in each declaration's `description`. The README gains a Secrets Bootstrap
section with the `init` / `list` / `doctor` / `set` path and a table of which
keys are not required and why.

### 2. Bootstrap-only env boundary was incomplete

The PR added the hard rule "read env in bootstrap only" and then left three
violations on the surfaces it changed.

`SUPPORTED_API_VERSIONS` and `DEFAULT_API_VERSION` were still read lazily by
two `OnceLock`s in `routes/middleware.rs`, so the effective version vocabulary
depended on whichever request arrived first. They are now typed config
(`[acme_api.api]`), resolved once at bootstrap into `ApiVersionState` and
passed to the middleware as typed state.

`AcmeLocalAuthService::from_env` re-resolved `ENVIRONMENT` and reloaded the
whole config stack, giving the auth service its own view of authority that
could diverge from the posture `main` had already validated. It now takes the
loaded `AppConfig` via `from_config`, and `allows_degraded_startup` takes the
resolved environment as an argument rather than re-reading it — that check
gates a secret, so two independent resolutions disagreeing is the failure mode
worth removing. `from_env` remains for test harnesses and operator tooling
that hold no `AppConfig`.

`SESSION_MAX_ABSOLUTE_DAYS` bypassed the existing typed
`absolute_session_timeout_days` field. The env read is gone.

The same treatment applies to the three cookie knobs on the policy surface
this PR already changed — `COOKIE_PREFIX`, `COOKIE_SAMESITE_STRICT`, and
`REFRESH_TOKEN_MAX_AGE` — which are now `[acme_api.cors]` fields with
committed defaults.

All six migrated keys join `LEGACY_BEHAVIOR_ENV_KEYS`, the repo's existing
warn-on-migrated-key mechanism, and are removed from
`config/env-manifest.txt` (50 keys to 44). `COOKIE_SECURE`, `COOKIE_DOMAIN`,
and `CORS_ORIGINS` stay in env: contract 031 explicitly allows env as the
allowlisted override path for cookie and CORS deployment wiring.

Scope held: no provider or adapter refactor. `RATE_LIMIT_BACKEND`, `REDIS_URL`,
`EMAIL_ADAPTER`, and the S3 keys are adapter selection, genuinely per-deploy,
and unchanged.

### 3. Test-plan claims were wrong

The README said four targets run and that `acme-front` runs Vitest. The plan
reports `targets: 3`. `acme-front` appears in the summary with
`available-suites: vitest` but `default-suites:` empty and `command: <none>`,
because `apps/acme-front/effigy.toml` sets `[test.suites.vitest]
default = false` so its empty suite does not fail root sequences.

README and this log now state the real plan and name the reason. The existing
front-test debt is left as-is, per the review.

## Residual Risk

- **Proxy env rename.** Any operator setting `TRUST_PROXY_HEADERS` or
  `TRUSTED_PROXIES` outside this workspace loses forwarding-header trust and
  falls back to the socket peer. That is the fail-closed direction, but it is
  a real deployment note for anyone who has copied this reference.
- **`qa:security` and `qa:templates`** still execute sibling Underlay shell
  scripts. Publishing them is an Underlay decision, out of scope here.
- **Conditional secrets** are expressed in descriptions and manifest comments,
  not enforced. Neither the static checker nor Effigy's single `required`
  boolean can express "required when deployed" or "required when this adapter
  is selected"; that stays a runtime and operator concern by design.
- **Migrated behavior keys.** Anyone who set `SUPPORTED_API_VERSIONS`,
  `DEFAULT_API_VERSION`, `SESSION_MAX_ABSOLUTE_DAYS`, `COOKIE_PREFIX`,
  `COOKIE_SAMESITE_STRICT`, or `REFRESH_TOKEN_MAX_AGE` in a deployed
  environment now gets a startup warning and the committed typed default. No
  environment in this workspace set any of them. The replacement field is
  named in the warning.
- **`config/default.toml` has no `[acme_api.cors] cookie_secure`**, so the
  code default (`false`) applies. A deployed environment must set it in its
  overlay or startup now fails — louder than before, and intentionally so.

## Next Task

Await orchestrator review of the PR. After an authorised merge and exact-main
verification, the orchestrator closes Underlay `g09.047` and promotes the
independent `g09.048`-`g09.052` consumer lanes whose decision gates are
satisfied.
