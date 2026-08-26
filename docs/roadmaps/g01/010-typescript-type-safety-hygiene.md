# g01.010 TypeScript Type-Safety Hygiene

Status: done (2026-07-19)
Owner: repo maintainers
Updated: 2026-07-19
Governing refs: underlay `docs/contracts/090` (TS runtime), underlay `docs/roadmaps/g08/024-strict-type-and-dependency-hygiene.md`, underlay `docs/logs/2026-07/18-100000-consumer-audit-underlay-reference.md`
Planning state: complete

## Goal

Restore real type safety across the acme TypeScript packages so the g08
TS-surface hardening actually holds in the reference app.

## Why this matters now

The g08 audit removed `noImplicitAny: false` from `acme-admin` and `acme-front`
tsconfig (it silently re-permitted implicit `any` under `strict`). svelte-check
stayed green — but only because the type holes are papered over by casts. The
casts are the real erosion and remain.

## Findings this card closes

1. **63 `as never` casts in `acme-admin`** (concentrated in detail pages: media,
   tasks, projects, categories, users), mostly forcing Svelte snippet/config
   props into underlay template props. Combined with the (now-removed)
   `noImplicitAny: false`, they substantially undermined the hardened types.
2. **5 `as any` casts** in webauthn code (`account/passkeys/+page.svelte`,
   `(auth)/login/+page.svelte`).
3. **Empty `package.json` script blocks** in `acme-admin`, `acme-front`,
   `acme-client`, `acme-ui` — no `check`/`svelte-check`/`lint` scripts, so all
   type/guard validation is only reachable through effigy, with no npm-level
   fallback.

## Scope

- [x] All 63 `as never` casts removed. Root cause was a real underlay
  template typing gap (`TemplateSurface` rejected parameterized snippets) —
  fixed **upstream** (underlay `c5f3cb7c`) rather than cast past or merely
  recorded, since the foundation lives in this workspace.
- [x] The 5 webauthn `as any` casts removed — the underlay
  `utils/webauthn` helpers already take `unknown` and return typed options;
  no cast was needed.
- [x] `check` scripts added to all four packages
  (`svelte-check --tsconfig ./tsconfig.json`; `tsc --noEmit -p
  tsconfig.json` for `acme-client`).
- [x] `noImplicitAny` stays on (no overrides exist).

## Deliverables

- [x] `acme-admin`/`acme-front`/`acme-client`/`acme-ui` free of `as never`
  and `as any` (zero occurrences, none needed justification)
- [x] `package.json` `check` scripts wired for all four packages

## Validation

- [x] `svelte-check` reports 0 errors for each package
- [x] `grep -rn "as never\|as any" src` returns nothing
- [x] each package's `bun run check` runs the type check (verified green)

## Next

`g01.011` gate hardening and lint cleanup.
