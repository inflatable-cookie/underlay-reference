# g01.010 TypeScript Type-Safety Hygiene

Status: ready
Owner: repo maintainers
Updated: 2026-07-18
Governing refs: underlay `docs/contracts/090` (TS runtime), underlay `docs/roadmaps/g08/024-strict-type-and-dependency-hygiene.md`, underlay `docs/logs/2026-07/18-100000-consumer-audit-underlay-reference.md`
Planning state: ready

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

- [ ] Replace `as never` casts with correctly-typed props. Where the underlay
  template's prop type genuinely does not fit, capture it as an upstream
  finding (a real underlay template typing gap) rather than casting past it.
- [ ] Replace the 5 webauthn `as any` casts with the proper WebAuthn/underlay
  types (`toPublicKeyRequestOptions` / assertion helpers already exist in
  underlay `utils/webauthn`).
- [ ] Add `check` scripts to each package's `package.json`
  (`svelte-check --tsconfig ./tsconfig.json`, or `tsc --noEmit` for
  `acme-client`) so type checking has an npm-level entry point alongside effigy.
- [ ] Keep `noImplicitAny` on (do not reintroduce the override).

## Deliverables

- [ ] `acme-admin`/`acme-front`/`acme-client`/`acme-ui` free of `as never` and
  `as any` (or each remaining cast justified with a comment + upstream link)
- [ ] `package.json` `check` scripts wired for all four packages

## Validation

- [ ] `svelte-check` reports 0 errors for each package
- [ ] `grep -rn "as never\|as any" src` returns only justified, commented
  occurrences
- [ ] each package's `npm run check` (or `bun run check`) runs the type check

## Next

`g01.011` gate hardening and lint cleanup.
