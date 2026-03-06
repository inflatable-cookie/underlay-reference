# Sweep 003 - Frontend Consistency Sweep (acme-admin + acme-front)

Date: 2026-02-13

Scope reviewed:
- `acme-admin/src`
- `acme-front/src`
- `acme-client/src` (API boundary context)

## Check outcomes

- Step 1 (route architecture): pass
  - Both apps use `(auth)` and `(app)` route groups with clear protected/public separation.
- Step 2 (API boundary): pass with one normalization gap
  - No raw feature-level `fetch()` calls found in app code.
  - Typed client command usage is consistent.
- Step 3 (naming/import hygiene): pass
  - Deep relative import drift (`../../../`) not detected.
  - Alias imports (`$lib`, `@api-client`, `@decodelabs/underlay`) are dominant.
- Step 4 (state handling): pass with one consistency note
  - Tokens are not passed via load data.
  - Admin uses URL-backed tab/list state more broadly than front.
- Step 5 (forms/lists/dialogs): mostly pass
  - Dialog/form consistency improved from Sweep 002 remediations.
- Step 6 (loading/error/empty): pass
  - Data-heavy pages consistently use `PageLoading` + `FormError` + empty states.
- Step 7 (theme/style tokens): partial
  - Underlay styles are imported in both apps.
  - Significant hardcoded color usage remains in feature files/utilities.
- Step 8 (a11y baseline): partial
  - One repeated anti-pattern found and fixed during sweep (`button` missing `type`).

## Findings

### [API BOUNDARY] [MEDIUM] Client configuration is duplicated across hooks and stores

- **Location:**
  - `acme-admin/src/hooks.server.ts:15`
  - `acme-admin/src/hooks.client.ts:5`
  - `acme-admin/src/lib/stores/auth.ts:42`
  - `acme-front/src/hooks.server.ts:14`
  - `acme-front/src/hooks.client.ts:5`
  - `acme-front/src/lib/stores/auth.ts:20`
- **Check step:** Step 2.2
- **Expected pattern:** API base/version setup is centralized in one runtime boundary per app shell.
- **Observed drift:** `configureAcmeClient(...)` is initialized in both hooks and auth stores.
- **Recommended normalization:** Keep client configuration in hooks only; remove store-level duplicate setup unless there is a documented fallback requirement.
- **Owner:** Frontend/platform
- **Target date:** Next frontend architecture cleanup
- **Status:** Resolved (client configuration centralized to hooks/runtime boundaries)

### [STATE/UX] [LOW] URL-backed UI state is richer in admin than front

- **Location:**
  - Admin examples: `acme-admin/src/routes/(app)/users/[userId]/+page.svelte:381`, `acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte:357`
  - Front examples: `acme-front/src/routes/(app)/dashboard/+page.svelte`, `acme-front/src/routes/(app)/projects/[projectId]/+page.svelte`
- **Check step:** Step 4.2
- **Expected pattern:** Similar list/tab patterns use URL-backed state consistently where it improves navigation/shareability.
- **Observed drift:** Admin uses `historyKey` and query-driven list controls broadly; front routes use in-memory-only state.
- **Recommended normalization:** For front pages that gain tabs/filters/pagination, align with URL-backed patterns early (`historyKey`, `page`, `sort`, `limit`).
- **Owner:** Frontend
- **Target date:** As front list pages expand
- **Status:** Accepted note (front scope remains intentionally simpler)

### [STYLING] [MEDIUM] Hardcoded color values are spread across feature modules

- **Location (examples):**
  - `acme-admin/src/lib/utils/accents.ts`
  - `acme-admin/src/routes/(app)/system/+page.svelte`
  - `acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte`
  - `acme-admin/src/lib/ui/AdminNavList.svelte`
- **Check step:** Step 7.2
- **Expected pattern:** Theme and semantic colors are centralized via tokens/utilities.
- **Observed drift:** Many route/components encode hex values directly.
- **Recommended normalization:** Move repeated semantic colors into shared token map utilities and consume token variables in components.
- **Owner:** Frontend/platform
- **Target date:** Incremental (start with `accents.ts` + system/media pages)
- **Status:** Partial resolved (semantic-token migration completed for targeted system/media scope)

### [ACCESSIBILITY] [LOW] Buttons without explicit `type` inside detail view

- **Location:** `acme-admin/src/routes/(app)/system/emails/[id]/+page.svelte`
- **Check step:** Step 8
- **Expected pattern:** Non-submit buttons explicitly declare `type="button"`.
- **Observed drift:** Copy buttons omitted explicit type.
- **Recommended normalization:** Add `type="button"`.
- **Owner:** Frontend
- **Target date:** Immediate
- **Status:** Resolved

## Frontend consistency sweep summary

- Critical: 0
- High: 0
- Medium: 2
- Low: 1
- Notes: 1

## Normalization plan

- Immediate fixes:
  - Completed: explicit `type="button"` on email detail copy buttons.
- Follow-up refactors:
  - Continue incremental hardcoded-color migration in remaining admin surfaces.
- Accepted exceptions:
  - Front app currently has simpler in-memory list state due lower feature complexity.

## Resolution update (2026-02-13)

- API boundary duplication finding is closed.
- Accessibility issue remains closed.
- Styling drift is reduced for priority areas and tracked as incremental follow-up for remaining surfaces.
