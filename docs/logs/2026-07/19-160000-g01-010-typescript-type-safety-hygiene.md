# 2026-07-19 16:00:00 - g01.010 TypeScript Type-Safety Hygiene

## Summary

Executed `g01.010`: removed every `as never` (63) and `as any` (5) cast from
the acme TypeScript packages and wired npm-level `check` scripts. The
`as never` casts all shared one root cause — an underlay template typing gap
— which was fixed upstream instead of being cast past or annotated.

## Completed work

- **Upstream fix (underlay `c5f3cb7c`)**: `TemplateSurface` was
  `Snippet | fn`; bare `Snippet` means `Snippet<[]>`, so every
  argument-taking snippet failed to assign — the reason all 63 casts
  existed. It is now the structural rest-args function type, which accepts
  snippets of any arity and avoids cross-package `Snippet` unique-symbol
  mismatches. `DetailItemConfig.value` and `MediaActionsMenu.trigger` moved
  off bare `Snippet` too. (The roadmap called for capturing this as an
  upstream finding; the foundation lives in this workspace, so it was fixed
  at source.)
- **acme-admin**: all 63 `as never` casts deleted (detail pages: media,
  tasks, projects, categories, users; list pages; dashboard; system). Local
  `MediaActionsMenu` wrapper `trigger` prop now uses `TemplateSurface`.
- **webauthn**: the 5 `as any` casts removed — the underlay helpers
  (`toPublicKeyRequestOptions` / `toPublicKeyCreationOptions`) take
  `unknown`, `publicKey.challenge` is a typed field, and
  `PasskeyLoginFinishRequest.credential` is `unknown`; none of the casts
  were needed.
- **`check` scripts** added to `acme-admin`, `acme-front`, `acme-ui`
  (`svelte-check --tsconfig ./tsconfig.json`) and `acme-client`
  (`tsc --noEmit -p tsconfig.json`) so type checking has an npm-level entry
  point alongside effigy.
- `noImplicitAny` remains on everywhere (no tsconfig overrides exist).

## Validation

- `grep -rn "as never\|as any" acme-*/src` → 0 occurrences
- `bun run check` green in all four packages (0 errors, 0 warnings)
- underlay itself stays green (`svelte-check`, `tsc`, export check) with the
  new `TemplateSurface`

## Next Task

Execute `g01.011` (gate hardening and lint cleanup).
