# g01.012 Northstar AGENTS, Rust, And TypeScript/Svelte Audit

Status: complete
Owner: repo maintainers
Date: 2026-09-01
Roadmap: `docs/roadmaps/g01/012-northstar-instruction-and-language-quality-audit.md`
Spec: `docs/specs/002-northstar-instruction-and-language-quality-audit.md`
Card: `docs/specs/batch-cards/002-northstar-agents-rust-typescript-audit.md`
Handoff: `docs/handoffs/20260901-083034-northstar-agents-rust-typescript-audit.md`

## Run Identity

- Worker branch: `worker/northstar-agents-rust-typescript-audit`
- Worker worktree: `/Users/tom/.paseo/worktrees/119ajruu/northstar-language-audit`
- Planning base `46a54a4e` is an ancestor of the launch `HEAD` `135fab45`,
  which equalled `origin/main` at preflight.
- Sibling links verified in the worktree container: `underlay ->
  /Users/tom/Dev/projects/underlay`, `poodle -> /Users/tom/Dev/projects/poodle`.
- Northstar source: `/Users/tom/Dev/projects/northstar/skills/northstar` at
  `dbce3856be6ec6093d2e5c071568a6dbe953df49`.
- The globally installed copy at `~/.agents/skills/northstar` is **stale**
  against that hash (it predates the Rust evidence-collection repair in
  `dbce3856`). Every mode file, projection, schema, and tool in this run was
  loaded from the pinned source tree, not from the installed copy.
- Toolchains: `rustc 1.97.1 (8bab26f4f 2026-07-14)`,
  `cargo 1.97.1 (c980f4866 2026-06-30)`, `effigy v0.12.1+local.47458a1`,
  `stopslop 0.5.1`.

## Managed Northstar Setup

Neither language pack was previously activated. Setup was applied from the
pinned source:

- `northstar/rust-quality:setup apply <root> apps/acme-api` — scope
  `apps/acme-api`, 9 manifests, 0 toolchain policy paths. It reported
  `activation installed, but repository MSRV is unresolved`.
- `northstar/typescript-quality:setup apply <root> .` — scope `.`, 5 packages
  discovered, 0 unregistered candidates.

Setup wrote `docs/contracts/rust-quality-profile.json`,
`docs/contracts/rust-quality-deviations.json`,
`docs/contracts/typescript-quality-profile.json`,
`docs/contracts/typescript-quality-deviations.json`, the
`northstar:typescript-quality` block in root `AGENTS.md`, and the
`northstar:rust-quality` block in `apps/acme-api/AGENTS.md`.

## Instruction Surface Audit (Northstar AGENTS review)

Scope: root `AGENTS.md`, `docs/AGENTS.md`, `apps/acme-api/AGENTS.md`,
`apps/acme-admin/AGENTS.md`, `apps/acme-front/AGENTS.md`,
`packages/acme-client/AGENTS.md`, `packages/acme-ui/AGENTS.md`, and the Claude
bridge. No other instruction surface exists in this repository.

### Mechanical measurements

`effigy --repo <northstar> northstar/check:agent-instructions <root>` measures
the root file only; nested scopes were measured with the same non-blank/byte
counts.

| Scope | Non-blank before | after | Bytes before | after | Note |
| --- | --- | --- | --- | --- | --- |
| `AGENTS.md` | 98 | 113 | 6247 | 7243 | +13 non-blank is the generated `northstar:typescript-quality` activation block; the review's own edits are +2 |
| `docs/AGENTS.md` | 36 | 37 | 2459 | 2236 | −223 bytes |
| `apps/acme-api/AGENTS.md` | 44 | 51 | 2674 | 2999 | +10 non-blank is the generated `northstar:rust-quality` block; the review's own edits are −3 |
| `apps/acme-admin/AGENTS.md` | 28 | 25 | 1541 | 1258 | −283 bytes |
| `apps/acme-front/AGENTS.md` | 27 | 24 | 1432 | 1143 | −289 bytes |
| `packages/acme-client/AGENTS.md` | 28 | 26 | 1454 | 1169 | −285 bytes |
| `packages/acme-ui/AGENTS.md` | 26 | 24 | 1382 | 1091 | −291 bytes |
| `CLAUDE.md` | absent | 1 | 0 | 11 | new bridge |

Setting aside the two generated activation blocks, the instruction surface lost
1371 bytes across the six nested scopes and gained 190 at root. The root file
remains above Northstar's 100-non-blank-line advisory target at 113; that is
retained deliberately rather than met by deleting sections that carry authority
(env/secret authority, worker-mode activation, workspace shape). The checker's
own note applies: the goal is a guide an unfamiliar agent can work from, not the
smallest file.

"Before" is the launch `HEAD` `135fab45`, i.e. prior to Northstar activation
blocks. Checker leads on the root file before: `placement_leads=9`, `procedure_leads=10`,
`freshness_leads=2`, plus `missing Claude bridge`. After: `placement_leads=11`,
`procedure_leads=10`, `freshness_leads=2`, `Claude bridge OK: CLAUDE.md ->
@AGENTS.md`. The two extra placement leads are the newly correct `../underlay`
and `docs/contracts/` references the checker counts as scoped paths. These are
context-cost measurements, not a prose verdict.

### Section-intent map and dispositions

Force key: **boundary** = non-negotiable, **default** = normal practice,
**taste** = maintainer preference, **pointer** = navigation.

#### Root `AGENTS.md`

| Section | Intent | Force | Disposition |
| --- | --- | --- | --- |
| Purpose | Establishes that this repo is a template to be copied, which is why canonical patterns beat local cleverness. | boundary | retain |
| Keep AGENTS Lean | Protects the instruction surface itself from growth. | taste | rewrite for intent — the rule as written ("`AGENTS.md` files should contain only …") is falsified by the root file it sits in, so an agent applying it literally would prune root sections that carry real authority. Scope it to the app/package scopes it actually governs. |
| Hard Rules | Names the properties a change must not break: workspace shape, single lockfile, wire conventions, worker-mode activation. | boundary | retain |
| Effigy-First Execution | Teaches the default command surface. | default | retain |
| Runtime Stance | Explains *why* host artifact directories are not the running stack — the cause/consequence that makes the prohibition memorable. | boundary | retain; it is the canonical statement the nested scopes should point at rather than copy. |
| Validation | The command list. | default | retain |
| Env And Secret Authority | Defines what "complete" means for env surface changes and why `.env` is not a contract. | boundary | retain |
| Source of Truth | Routes planning questions to `docs/` and framework questions to Underlay. | pointer | rewrite for intent — `underlay/docs/guides/` contradicts the same file's `../underlay` mount convention twelve lines earlier. |
| Internal Writing Style | Pointer to the style policy. | pointer | retain |
| Effigy Agent Contract (managed block) | Effigy's synced agent contract. | default | report only — the block is sync-managed (`36cdd7e3 chore: sync Effigy agent skill`) and its three "Reference docs" links (`docs/guides/047-…`, `076-…`, `017-…`) resolve to nothing in this repository because they address Effigy's own docs tree. Repairing generated content here would be overwritten on the next sync and is out of this card's authority. |
| Northstar TypeScript/Svelte explicit audit (managed block) | Activation for this pack. | boundary | retain as generated. |

#### `docs/AGENTS.md`

| Section | Intent | Force | Disposition |
| --- | --- | --- | --- |
| Scope | Names root `docs/` as the authority and its `acme-docs` catalog alias. | boundary | retain |
| Hard Rules | Roadmap ID shape, log filename shape, no shim docs, tier meanings. | boundary | retain |
| Effigy-First Execution | Two runtime-stance bullets copied verbatim from root, then docs-specific selectors. | default | merge — drop the duplicated root bullets, keep the docs-specific selector guidance and repo notes. |
| Validation | Docs selectors. | default | retain |
| Reference Docs | Pointers. | pointer | retain |
| Internal Writing Style | Pointer. | pointer | retain |

#### `apps/acme-api/AGENTS.md`, `apps/acme-admin/AGENTS.md`, `apps/acme-front/AGENTS.md`, `packages/acme-client/AGENTS.md`, `packages/acme-ui/AGENTS.md`

All five share one shape, so they share dispositions.

| Section | Intent | Force | Disposition |
| --- | --- | --- | --- |
| Scope | One sentence on what the unit is. | boundary | retain |
| Hard Rules | The package-local properties that must survive. `acme-api`'s are the sharpest in the repo (route registration home, env-in-bootstrap-only, peer-aware client IP). | boundary | retain |
| Effigy-First Execution | Two runtime-stance bullets copied verbatim from root plus a four-step "default flow" that restates `effigy health` and `effigy validate`, which the file's own Validation section then repeats. | default | merge — remove the duplicated root bullets and the restated commands; keep the package-local `effigy` default and the repo notes that are genuinely local (`acme-api`'s `health`/`validate`/nextest/`migration:*` notes). |
| Validation | The commands. | default | retain |
| Reference Docs | Pointers into root `docs/` and the sibling Underlay guides. | pointer | rewrite for intent — the sibling entries use `../underlay/…`, which is root-relative, while the entries beside them use file-relative `../../docs/…`. From the file's own directory `../underlay` resolves to `apps/underlay` or `packages/underlay`, neither of which exists. |
| Internal Writing Style | Pointer. | pointer | retain |
| Northstar Rust Quality (`acme-api`, managed block) | Activation for the Rust pack. | boundary | retain as generated. |

#### Claude bridge

| Surface | Disposition |
| --- | --- |
| `CLAUDE.md` | investigate → repair. The file is absent, so a Claude-family agent entering this repository gets no instruction surface at all. Northstar's bridge contract is a single `@AGENTS.md` reference with Claude-only guidance added *only* when it cannot live in the shared contract; nothing here qualifies, so the bridge is exactly that one reference. |

### Instruction findings

| ID | Scope | Finding | Disposition |
| --- | --- | --- | --- |
| `AGENTS-001` | 5 app/package files | Sibling Underlay guide links use a root-relative base inside an otherwise file-relative list and do not resolve from the file's own directory. | repair |
| `AGENTS-002` | root `AGENTS.md` | "Source of Truth" writes `underlay/docs/guides/` where the same file establishes the `../underlay` mount. | repair |
| `AGENTS-003` | 6 nested files | The two Runtime Stance bullets are duplicated verbatim from root, and each file restates its own validation commands twice. | repair |
| `AGENTS-004` | repository root | No `CLAUDE.md` bridge exists. | repair |
| `AGENTS-005` | root `AGENTS.md` | "Keep AGENTS Lean" states a rule the file it lives in does not satisfy. | repair |
| `AGENTS-006` | root `AGENTS.md` | The managed Effigy Agent Contract block's three reference-doc links do not resolve in this repository. | report only — sync-managed block, out of card authority |

### Instruction repairs applied

| ID | Change |
| --- | --- |
| `AGENTS-001` | The 12 sibling Underlay guide links in the five app/package files became file-relative (`../../../underlay/docs/guides/…`). All 22 link targets in the nested files now resolve from their own directory; verified by resolving each path against its containing directory. |
| `AGENTS-002` | Root "Source of Truth" now says `../underlay/docs/guides/`, matching the `../underlay` mount the same file declares. |
| `AGENTS-003` | The two Runtime Stance bullets duplicated verbatim into all six nested files were replaced by a one-line pointer at root, and each nested file's restated `effigy health` / `effigy validate` sequence was folded into the Validation section it duplicated. Package-local content — `acme-api`'s health/validate/nextest/`migration:*` notes and `docs`'s `acme-docs/...` selector guidance — was kept and, in the docs case, rewritten to say why the selectors exist. |
| `AGENTS-004` | `CLAUDE.md` created with the single required `@AGENTS.md` reference and nothing else. |
| `AGENTS-005` | "Keep AGENTS Lean" now scopes its four-item rule to the app and package files it actually governs and adds "A nested file should point at this one rather than restate it", so the rule is no longer contradicted by the file stating it. |
| `AGENTS-006` | Not repaired. Reported. |

## Rust Audit

Mode: Northstar Rust explicit audit and repair, `repository` scope.

### Tooling

- Payload: `northstar-rust-quality` built from the pinned source at
  `2b75b0866e3bedf99c133e53cb742c284715fb1f10f589358ce2a91331571157`
  (`verify-install` receipt `current: true`, embedded and source payload hashes
  equal). The pre-existing probe binary in the Northstar cache was stale against
  the pinned source and was not used for any operation.
- Scanner: `stopslop 0.5.1`, verified by `--version`.

### Scope

| | |
| --- | --- |
| Workspaces | 1 (`apps/acme-api`) |
| Packages | 8 — acme-api, acme-auth, acme-core, acme-db, acme-domain, acme-infra, acme-jobs, acme-test-utils |
| Targets | 19 target entries over 17 distinct source paths (9 lib, 6 bin, 2 test, 1 custom-build, 1 shared) |
| Cargo features | 0 declared anywhere in the workspace |
| Assessed units | 9 — one per crate plus `workspace-manifest` |
| Owned anchors | 145 files (136 `.rs` + 8 crate manifests + the workspace manifest) |
| Excluded dirty files | 13, all non-Rust: 7 `AGENTS.md`, `CLAUDE.md`, 4 `docs/contracts/*.json`, this log |

Every discovered package manifest and target source path is owned by an
assessed unit; the recorder's `plan` operation rejects a coverage claim that
does not match Cargo discovery exactly, so this is checked rather than asserted.

### Record

- Discovery `3aaa6e11dbb3d8dcf385968964cb7f05b5fcd4abaeb22f17e56d8d469ff715f4`
- Plan `45c737d18c6c0cedf3f3376637ce79cb640091a22974c0cbf6b310487bfd535e`
- Policy (strict projection) `c2ccfbe0d28c7a938abe33f6455b3788d9f65442a3e8fb9a58cfd545f4539a2c`
- `result.json` `fd1d731ee91d735be61121d6e7ed4ec247ee0b952f699b716d40b14970fead27`
- `report.md` `5f486abbd5f677738e111885d27f201145a72e050a87ef4ac388e05894de85bd`
- Final status: `operator_action_required`
- Records live in Git metadata at
  `<git-dir>/northstar/rust-quality/audits/g01012-rust/`; they are not tracked
  files and are not part of the PR diff.

Six normative rules carry a verdict for all nine units (54 verdicts), each with
three dimension attestations. `RUST-SLOP-001` is `prototype` /
`evaluation_only` in the strict projection, and the recorder rejects it as both
a verdict rule and a finding rule, so its candidate ledger is carried in each
unit's architecture attestation and repeated under "Retained findings" below.

### Repairs applied (4 files)

| Unit | Plan | File | Change |
| --- | --- | --- | --- |
| acme-infra | `INFRA-P1` | `crates/infra/src/encryption.rs` | `EncryptionService::is_encrypted` decoded the same base64 input twice and had an unreachable `Err(_)` arm. Now decodes once. |
| acme-auth | `AUTH-P1` | `crates/auth/src/local/rate_limit.rs` | Six near-identical rate-limit checks now build their key and call one `enforce_hourly_rate_limit` helper. Every key string, every configured limit, and the denial shape are unchanged. Same wave fixes a reachable panic: `format!("{:x}", …)[..8]` became `format!("{:016x}", …)[..8]`, because `{:x}` does not pad and a hash below `0x10000000` renders shorter than the slice. 150 lines became 51. |
| acme-auth | `AUTH-P2` | `crates/auth/src/local/mod.rs` | Removed a dead `use crate::email_totp::{…}` and its `#[allow(unused_imports)]`, whose comment claimed email TOTP was not yet integrated. |
| acme-jobs | `JOBS-P1` | `crates/jobs/src/handlers/projects.rs` | The two project-report handlers duplicated ~60 lines of batch logic verbatim. Extracted `resolve_project_ids` and `run_project_report_batch`; every log event, field name and error string is preserved. 204 lines changed, net −28. |

The other five units are byte-for-byte unchanged; the recorder verifies that
against their initial fingerprints before finalization.

### Retained findings (no mutation)

| ID | Rule | Disposition | Summary |
| --- | --- | --- | --- |
| `AUTH-FINGERPRINT-FAIL-OPEN` | RUST-ERR-001 | operator_decision | `SessionFingerprint::matches` returns true when either side's IP or User-Agent is absent, so a refresh without a User-Agent skips that check. The doc states the intent, so this is a security-contract decision, not a defect to fix under audit authority. |
| `AUTH-SILENT-FALLBACKS` | RUST-ERR-001 | reported | `map_credential_row` swaps a failed `CredentialMetadata` parse for hard-coded argon2 parameters with no log; `map_user_row` maps an unknown `status` to `Active` and an unknown `credential_type` to `Password`. Both defaults fail open. |
| `AUTH-DEAD-LOGIN-ENTRYPOINTS` | RUST-READ-001 | reported | `login_with_password`, `login_start_with_password` and `login_start_with_password_and_ip` have no caller anywhere; the last duplicates `login_start_with_email_fallback(…, false)` exactly. |
| `AUTH-DEBUG-GAP` / `API-DEBUG-POLICY` / `JOBS-DEBUG-GAP` / `DB-PARAMS-NO-DEBUG` / `TU-TESTDB-NO-DEBUG` | RUST-API-001 | reported | Roughly 60 public types omit `Debug`. Some omissions are correct protection (`AppState`, `EncryptionService`, `AcmeLocalAuthService`, the credential-bearing auth DTOs); others are not (`CsrfState`, `ApiVersionState`, `LogActivityParams`, `TestDb`, the eight job handlers, the email-TOTP repositories). Deciding which must stay unformattable is a security-contract call across the whole public surface, and deriving `Debug` wholesale would be exactly the blanket lint fixing this mode forbids. |
| `API-BLOB-BOOTSTRAP-DUP` | RUST-READ-001 | reported | `crates/api/src/main.rs` and `crates/jobs/src/main.rs` duplicate ~45 lines of blob-adapter selection and have drifted: the API binary treats a failed `ensure_bucket_ready` on the production S3 path as fatal; the jobs binary never calls it there. Unifying them needs an owner outside both units. |
| `DB-ACTIVITY-PROJECTION` | RUST-READ-001 | reported | The eleven-column audit-log projection and its `has_more` tail are written out three times in `activity.rs` and must stay in step with `ActivityWithActorRow`. |
| `INFRA-FROM-ENV-SWALLOW` | RUST-ERR-001 | reported | `EncryptionService::from_env` collapses "unset" and "malformed" into `None`, so a corrupt `ENCRYPTION_KEY` silently disables the legacy TOTP read path. |
| `INFRA-REDUNDANT-ARM` | RUST-READ-001 | reported | `EmailAdapterType::parse` lists `"noop" \| "none" \| ""` directly above an identical `_` arm, so an unrecognised `EMAIL_ADAPTER` value is silently accepted. |
| `JOBS-FANOUT-DOC` | RUST-READ-001 | reported | `GenerateProjectReportsHandler`'s doc claims it fans out into individual jobs; it builds every report inline. |
| `JOBS-PROJECTIDROW-DUP` | RUST-READ-001 | reported | `ProjectIdRow` is declared identically in two handler modules, both with an unnecessary `#[allow(dead_code)]`. |
| `DOMAIN-DEAD-TYPES` | RUST-READ-001 | reported | `domain::TaskComment` and `domain::Tag` are never constructed anywhere; `acme-db` models comments with its own row type. Left in place because this is a reference template's worked example surface. |
| `TU-CLEANUPSTATS-ZERO` | RUST-ERR-001 | reported | `cleanup_all_test_data` returns `CleanupStats` with `projects` and `tasks` always zero even though it deletes both. |
| `TU-RUN-IN-TRANSACTION` | RUST-ERR-001 | reported | `run_in_transaction` has no caller and its doc example does not compile as written. |
| `TU-DOC-EXAMPLE-DRIFT` | RUST-READ-001 | reported | The `acme-test-utils` crate doc example calls the fixtures with the wrong arity and treats `TestDb` as a pool. |

`RUST-SLOP-001` candidate ledger (evaluation-only, no repair authority).
`stopslop 0.5.1 SLOP039` returned zero candidates for all eight crates; the
following were identified by inspection. **Retained with a named
responsibility:** the `acme-core` re-export facade, `AcmeLocalAuthProvider`
(AuthProvider substitution seam), `DynamicRateLimiter` (backend dispatch),
`acme_db::create_pool` / `run_migrations` (supply app-owned `DbConfig` and
`MIGRATOR`), `db_errors::internal_with_diagnostics` (`pub(crate)` single import
point for 127 call sites, documented message policy), `acme_api::build_router`
(documented test seam), `init_tracing` (AppConfig translation),
`EmailAdapterType::from_str` (std trait seam), `AcmeLocalAuthService::register`
and `refresh` (default-argument seams with live test callers), `TestDb::pool` /
`pool_clone`, `create_test_admin`. **Without one:**
`acme_infra::init_tracing_default` (documented "for backwards compatibility",
no caller), `acme_infra::create_email_context` (no caller),
`acme_jobs::create_registry` and `create_registry_with_blob` (no caller; only
`create_registry_with_media` is used), `acme_test_utils::run_in_transaction`
(no caller), and the three uncalled login entrypoints above. Also recorded:
`AcmeLocalAuthService::google_oauth` and `oauth_cipher` are constructed from
environment at startup and never read, and `RedisRateLimitBackend::reset`
rebuilds its window key with a hard-coded 3600-second bucket while `check` and
`increment` derive theirs from `config.window_seconds()` — the two agree only
because every configured limit is currently `per_hour`.

### Toolchain and MSRV

`rustc 1.97.1 (8bab26f4f 2026-07-14)`, `cargo 1.97.1 (c980f4866 2026-06-30)`.
That is the compiler this audit actually ran on. It is **not** an MSRV and is
not reported as one.

No `rust-version` appears in any of the nine Cargo manifests, and no
`rust-toolchain`/`rust-toolchain.toml` exists in the repository — setup recorded
`toolchain_paths=0` and reported `repository MSRV is unresolved`. `RUST-MSRV-001`
is therefore `degraded` for all nine units, each linked to a
`rust-msrv-unresolved-<unit>` limitation. `[workspace.package]` declares
`edition = "2021"` and the audit found no 2024-edition-only construct.
Declaring a minimum is a `change_version_policy` operator decision under
`g01.012`'s stop conditions and was not made.

### Mechanical evidence

36 immutable records, four classes for each of the nine units, all `passed`
with exit 0 and zero warnings. Selectors resolved from the repository's own
`apps/acme-api/effigy.toml`:

| Class | Selector | Origin |
| --- | --- | --- |
| compiler | `acme-api/check` — `cargo check --workspace --all-features`, narrowed per package | repository_task |
| lint | `acme-api/clippy` — `cargo clippy --workspace --all-targets -- -D warnings`, narrowed per package | repository_task |
| test | `acme-api` test suite `rust` — `cargo test --workspace`, narrowed per package | repository_task |
| scanner | `stopslop 0.5.1 --select SLOP039` per crate | agent_resolved |

`docs` and `graph` are not applicable classes here: the repository declares no
docs task, `cargo test --workspace` already runs the doctests, and `effigy
graph` is a code-navigation surface rather than validation evidence.

**Honest limit on the test evidence.** `cargo test --workspace` exits 0, but no
database was reachable in the audit environment (`DATABASE_URL` and
`TEST_DATABASE_URL` both unset). Every DB-backed test in `acme-test-utils`,
`acme-db`, `acme-jobs` and `acme-api` self-skips with an `eprintln` and reports
`ok`, so the green result proves compilation and the pure-logic tests
(`acme-domain` enums, `acme-infra` startup posture and encryption, `acme-auth`
config and Redis key building, `acme-api` middleware/CSRF/client-IP), not
persistence or handler behaviour.

## TypeScript/Svelte Audit

Mode: Northstar TypeScript/Svelte explicit audit and repair, `repository` scope.

### Scope and overlays

| Package | Overlays | Svelte | SvelteKit |
| --- | --- | --- | --- |
| `apps/acme-admin` | base, svelte, sveltekit | ^5.56.8 | ^2.70.2 |
| `apps/acme-front` | base, svelte, sveltekit | ^5.56.8 | ^2.70.2 |
| `packages/acme-ui` | base, svelte | ^5.56.8 | absent |
| `packages/acme-client` | base | absent | absent |
| root workspace package | base | absent | absent |

Setup discovered five owned packages and **zero unregistered candidates**. 201
in-scope files across 8 disjoint units: `acme-client-commands` (24),
`acme-client-core` (14), `acme-ui` (27), `acme-front` (25), `acme-admin-lib`
(41), `acme-admin-routes` (61), `acme-admin-support` (8), `workspace-root` (1).

Explicit exclusions: `apps/acme-admin/src/lib/icons.generated.ts` (header
"Generated by poodle-icons. Do not edit."); and, as untracked build output
under `.gitignore`, `node_modules/`, `.svelte-kit/`, `build/`, `dist/` and
`apps/*/src/lib/config/public-api.generated.ts`. No vendored third-party source
exists inside the four package roots. The 13 dirty files at initialization were
all non-TypeScript and each carries an excluded disposition.

### Record

- Catalogue `3a524acd30de9a7637141da70bef921306c4544eeb37a9dd46e78cbfd51094a8`
- `result.json` `e4d139428c542341f5ea7892c4a61d54f0161721a2914db544fc4bb9f5e7ebf7`
- 28 findings, 4 applied repairs, 24 remaining limitations, 0 deviations
- Records live at
  `.effigy/typescript-quality/audits/g01012-typescript/`, which `.gitignore`
  excludes, so they are not part of the PR diff.

### Repairs applied (6 files)

| Unit | Rule | File(s) | Change |
| --- | --- | --- | --- |
| acme-admin-routes | TS-READ-001 | `src/hooks.client.ts`, `src/hooks.server.ts` | Both files imported `configureAcmeClient` **twice from the same specifier** under two aliases and called both, under a comment claiming this configured "both entrypoints" of a mixed `@api-client`/`acme-client` graph. Neither import named `acme-client`, and both aliases map to the same path anyway, so the second call was a no-op and the comment described a mechanism that did not exist. Now one import, one call, and a comment that states the actual alias situation. |
| acme-client-core | TS-EVIDENCE-001 | `src/utils/http-client.ts` | Removed seven `as UnderlayHttpError` assertions applied to values `instanceof` had already narrowed (one in `convertError`, six in the 401 CSRF-clear branches). |
| acme-ui | SVELTE-A11Y-001 | `src/nightfire/notes/TaskChecklistEditor.svelte` | The item checkbox had no label or `aria-label`, and the three icon-only buttons announced as "↑", "↓" and "×" because button content wins over `title` in accessible-name computation. Added `aria-label` to all four; no handler, state or layout change. `svelte-check` reports zero warnings here, so nothing mechanical was catching it. |
| acme-front | TS-READ-001 | `src/routes/(app)/dashboard/+page.svelte`, `src/routes/(app)/projects/[projectId]/+page.svelte` | Both script headers had imports at column 0 interleaved with two- and four-space indentation, statements split mid-list, and two separate imports from `@inflatable-cookie/poodle-svelte`. Normalized to one grouped block per file with the identical set of bindings; side-effect `@acme/ui/*` imports keep their leading position so Nightfire registration still runs first. |

### Retained findings (selected)

- **`apps/acme-front` has three test files that have never run.** `vite.config.ts`
  sets `test.include: ["src/**/*.{test,spec}.ts"]`, but the tests live under
  `tests/`. Verified by running the package's own command: `bun x vitest run`
  prints `No test files found, exiting with code 1`. The package also has no
  `test` script, and `effigy.toml` marks the suite `default = false` with the
  comment "the empty suite does not fail root `effigy test`" — the suite is
  empty only because the glob points at the wrong directory. Reported rather
  than repaired: correcting the glob changes what repository validation runs,
  and this audit cannot prove the three files pass without making that change.
- **`response.body!.data` twelve times** across the admin command modules, on a
  value typed `T | null`, with no guard. An empty or 204 response throws a
  `TypeError` inside the client instead of surfacing a typed failure.
- **Asymmetric wire-case conversion.** `toSnakeCaseValue` deliberately skips
  Nightfire values and blocks so their keys survive; `toCamelCaseValue` has no
  matching guard, so the same document is rewritten on the way back.
- **Prop-mirroring `$effect`** in all three Nightfire editors
  (`TaskChecklistEditor`, `TaskGalleryEditor`, `TaskNotesEditor`): a prop is
  copied into `$state` by a reactive effect, and `emit()` closes the loop by
  writing local state and then calling `onChange`. `acme-ui` has no test suite
  at all, so a rewrite of three live editing surfaces could not be proved safe
  here.
- **Two different public `AcmeClientConfig` interfaces** in one package
  (`client-factory.ts` and `utils/http-client.ts`); `index.ts` exports the
  narrower one, so a consumer cannot construct an `HttpClient` from the type the
  package publishes.
- **Package-edge bypass.** Both apps declare `acme-client: workspace:*` but
  resolve it through a Vite alias onto `packages/acme-client/src`, while the
  package's own `main`/`types` point at a `dist/` only `effigy
  acme-client/build` produces. `acme-front` additionally deep-imports
  `@api-client/utils/client-factory.js` while `acme-admin` uses the index.
- **`apps/acme-admin/tests/components.test.ts`** imports `render`, `screen` and
  `fireEvent` and uses none of them; all 315 lines build raw DOM nodes and test
  the browser rather than the app, contributing 24 of the suite's 32 green
  assertions.
- Boundary assertions rather than validation in
  `acme-ui/nightfire/notes/registrations.ts` validators,
  `acme-front` `summariseProjectDescription`, `acme-admin`
  `media-usage-resolution.ts` (a double assertion through `unknown`), and
  `acme-admin` `hooks.server.ts` `handleError`.

### What the codebase gets right

Recorded because an audit that reports only defects misrepresents the subject.
Zero `any`, zero `@ts-ignore`/`@ts-expect-error`, zero `eslint-disable` and zero
`svelte-ignore` anywhere in 201 files. Both `{@html}` sites sanitize first and
say why. The admin app uses real `<button>` elements, keyed `{#each}` blocks and
`aria-label`s throughout. On the Rust side: no `unsafe`, no FFI, zero
`.unwrap()` in non-test source, `.expect` confined to startup and constant
parsing, no non-async lock anywhere, Argon2 correctly moved off the executor
with `spawn_blocking`, and clippy clean at `-D warnings` across all targets.

## Validation

All commands run from the worker worktree on the head this PR proposes.

| Command | Result |
| --- | --- |
| `cargo check --workspace --all-features` (via `acme-api/check` selector) | exit 0, 0 warnings |
| `cargo clippy --workspace --all-targets -- -D warnings` (via `acme-api/clippy`) | exit 0, 0 warnings |
| `cargo fmt --all --check` | exit 0 |
| `cargo test --workspace` | exit 0; DB-backed tests self-skipped, see the Rust evidence caveat above |
| `effigy acme-client/check` (`tsc --noEmit`) | exit 0, no diagnostics |
| `effigy acme-ui/check` (`svelte-check`) | 538 files, 0 errors, 0 warnings |
| `effigy acme-admin/check` (`svelte-check`) | 4867 files, 0 errors, 0 warnings |
| `effigy acme-front/check` (`svelte-check`) | 885 files, 0 errors, 0 warnings |
| `packages/acme-client` `bun run test` | 2 files, 3 tests passed |
| `apps/acme-admin` `bun run test` | 2 files, 32 tests passed |
| `effigy qa:conformance` | exit 0; workspace-shape and env-authority checks passed |
| `effigy acme-docs/qa:docs` | exit 0; link, forbidden and heading checks passed |
| `effigy acme-docs/qa:northstar` | exit 0; heading checks passed |
| `effigy acme-api/validate` (build, test, clippy, fmt) | exit 0; 52 + 22 + 11 + 3 tests ok |
| `effigy acme-client/validate` (check, build) | exit 0 |
| `effigy acme-ui/validate` (check) | exit 0 |
| `effigy acme-front/validate` (check, build) | exit 0 |
| `effigy acme-admin/validate` (check, build) | exit 0 |
| `effigy acme-docs/validate` (rollout checks) | exit 0 |
| `effigy validate` (workspace root) | **fails, and not because of this change** — see below |
| `git diff --check` | clean |
| `effigy --repo <northstar> northstar/check:agent-instructions .` | Claude bridge OK; measurements above |

**Root `effigy validate` fails in the sibling Underlay checkout.** The
bundle-provided root sequence fans out into the mounted `underlay` catalog and
runs Underlay's own vitest suite, where
`ts/tests/tools/workspace-shape.test.ts > flags disposable leftover top-level
package trees after apps/packages migration` fails (`expected [] to have a
length of 1`). That is 1 failure out of 813 tests in a different repository:
`/Users/tom/Dev/projects/underlay` at `ca654570`, working tree clean, untouched
by this worker. Every task in the same run that belongs to this repository
passed, and every one of this repository's six catalog `validate` tasks passes
individually, as recorded above. Fixing or triaging it is Underlay's, not this
card's — `g01.012` forbids sibling mutation.

Not run: `effigy qa:security` and `effigy qa:templates`, which execute
`../underlay/scripts/*.sh` from the sibling checkout. They are conformance
checks over Underlay's own contract rather than validation of this change.

## Changed-File Attribution

Every changed file maps to a finding recorded before mutation.

| File | Lane | Authority |
| --- | --- | --- |
| `AGENTS.md` | instruction | `AGENTS-002`, `AGENTS-005`; plus generated `northstar:typescript-quality` block |
| `CLAUDE.md` (new) | instruction | `AGENTS-004` |
| `docs/AGENTS.md` | instruction | `AGENTS-003` |
| `apps/acme-api/AGENTS.md` | instruction | `AGENTS-001`, `AGENTS-003`; plus generated `northstar:rust-quality` block |
| `apps/acme-admin/AGENTS.md` | instruction | `AGENTS-001`, `AGENTS-003` |
| `apps/acme-front/AGENTS.md` | instruction | `AGENTS-001`, `AGENTS-003` |
| `packages/acme-client/AGENTS.md` | instruction | `AGENTS-001`, `AGENTS-003` |
| `packages/acme-ui/AGENTS.md` | instruction | `AGENTS-001`, `AGENTS-003` |
| `docs/contracts/rust-quality-profile.json` (new) | setup | `northstar/rust-quality:setup apply` |
| `docs/contracts/rust-quality-deviations.json` (new) | setup | `northstar/rust-quality:setup apply` |
| `docs/contracts/typescript-quality-profile.json` (new) | setup | `northstar/typescript-quality:setup apply` |
| `docs/contracts/typescript-quality-deviations.json` (new) | setup | `northstar/typescript-quality:setup apply` |
| `apps/acme-api/crates/infra/src/encryption.rs` | Rust | plan `INFRA-P1` |
| `apps/acme-api/crates/auth/src/local/rate_limit.rs` | Rust | plan `AUTH-P1` |
| `apps/acme-api/crates/auth/src/local/mod.rs` | Rust | plan `AUTH-P2` |
| `apps/acme-api/crates/jobs/src/handlers/projects.rs` | Rust | plan `JOBS-P1` |
| `apps/acme-admin/src/hooks.client.ts` | TypeScript | `TS-READ-001/remove_duplicate_client_configuration` |
| `apps/acme-admin/src/hooks.server.ts` | TypeScript | `TS-READ-001/remove_duplicate_client_configuration` |
| `packages/acme-client/src/utils/http-client.ts` | TypeScript | `TS-EVIDENCE-001/remove_redundant_error_assertions` |
| `packages/acme-ui/src/nightfire/notes/TaskChecklistEditor.svelte` | TypeScript | `SVELTE-A11Y-001/name_checklist_item_controls` |
| `apps/acme-front/src/routes/(app)/dashboard/+page.svelte` | TypeScript | `TS-READ-001/normalize_mangled_import_header` |
| `apps/acme-front/src/routes/(app)/projects/[projectId]/+page.svelte` | TypeScript | `TS-READ-001/normalize_mangled_import_header` |
| `docs/logs/2026-09/…` (this file), card, roadmap, spec, front doors | closeout | card 002 |

No file outside the four `apps/*` and `packages/*` roots, root `docs/`, and the
root instruction surface was touched. Underlay and Poodle were read as context
only; `git status` in both sibling checkouts is the operator's to confirm, and
this worker issued no write to either path.

## Limitations

1. **No fixed Rust MSRV.** The workspace declares none. `RUST-MSRV-001` is
   `degraded` for all nine units. `rustc 1.97.1` is recorded as the observed
   toolchain, not a minimum. Resolving this is a `change_version_policy`
   operator decision.
2. **DB-backed Rust tests did not exercise a database.** `cargo test
   --workspace` passes, but the persistence and handler suites self-skip
   without `DATABASE_URL`.
3. **`apps/acme-front`'s three test files have never run** and this audit did
   not change the glob that excludes them.
4. **`packages/acme-ui` has no test suite**, so the prop-mirroring `$effect`
   finding in its three Nightfire editors could not be repaired with proof.
5. **`RUST-SLOP-001` has no recorder representation.** The rule is
   `prototype`/`evaluation_only` and the Rust recorder rejects it as both a
   verdict and a finding rule, so its total candidate ledger lives in the unit
   attestations and in this log rather than in `result.json`.
6. **`Debug` policy across ~60 public Rust types is unresolved** and needs an
   owner decision on which types must remain unformattable.
7. **The Effigy Agent Contract block's three reference-doc links do not resolve
   in this repository.** It is a sync-managed block (`36cdd7e3`), so repairing
   it here would be overwritten and is outside this card's authority.
8. **Root `effigy validate` is red because of a pre-existing failure in the
   sibling Underlay repository**, not because of anything in this change. All
   six of this repository's own catalog `validate` tasks pass.
9. **Neither recorder's records are in the PR diff.** The Rust records live in
   Git metadata and the TypeScript records under gitignored `.effigy/`. The
   hashes above are the durable reference.
10. **The retained Underlay surface owned by `g01.007` and card 001 was not
    classified.** No file under that lane's authority was touched and its
    paused state is unchanged.

## Next Task

Orchestrator: review this PR at its exact head. On acceptance, merge, then
resume `g01.007` and card 001 from their unchanged paused state. The
retained-surface classification remains open; nothing in this audit advanced,
absorbed or superseded it.
