# Wasteful Endpoint Calls Remediation

This roadmap captures findings from running Underlay sweep **021 (Wasteful Endpoint Calls)** against this reference repo.

Run date: 2026-02-15 (initial), 2026-02-17 (re-sweep for Patterns J+K)

Source sweep: `underlay/docs/sweeps/021-wasteful-endpoint-calls-sweep.md`

## Sweep Summary

Pages/components audited: 27
Critical issues: 0
High issues: 2
Medium issues: 5
Low issues: 6
Notes: 3

This codebase is relatively clean. The reference app has no N+1 fan-out, no exhaustive pagination, and no duplicate sibling requests. The main issues are:

1. ~~Eager category data fetches for filter dropdowns (should lazy-load)~~ — Fixed
2. ~~Emails detail page mounts all tab content simultaneously~~ — Fixed
3. ~~Heavy DTO used where lightweight suggestion endpoints already exist~~ — Fixed
4. ~~Several dead/vestigial command exports~~ — Fixed
5. Redundant per-page `getToken` and manual `tryFetch` $effect on all 27 pages (Pattern K)
6. Unguarded filter-change `$effect` calling `refetch()` on 3 pages (Pattern J variant)
7. Audit page wasteful refetch on URL filter change when filtering is client-side

---

## Phase 1 - Eager Filter Data Fetches (Pattern G + B) ✓

Priority: **High** — Completed

These are the most impactful fixes — removing unnecessary API calls from page load.

### 1.1 Projects list: lazy-load category filter dropdown ✓

- [x] Remove `adminCommands.listCategories()` from `useAuthenticatedData` callback
- [x] Switch category filter `Select` to use `loadItems` prop with async loader
- [x] Use `listCategoriesForSuggestions` for lighter payload

### 1.2 Project edit/new forms: lazy-load category dropdown ✓

- [x] Remove `adminCommands.listCategories()` from `useAuthenticatedData` callback
- [x] Switch to `fetchCategories` prop path using `listCategoriesForSuggestions`
- [x] New project page: removed `useAuthenticatedData` entirely (was only used for categories)

### 1.3 Task edit/new forms: lazy-load label selector ✓

- [x] Remove `adminCommands.listLabels()` from `useAuthenticatedData` `Promise.all`
- [x] Labels now load non-blocking after page renders via separate `$effect`
- [x] Task edit: `getTaskLabels` also deferred with separate `labelsInitialized` flag

---

## Phase 2 - DTO Right-Sizing (Pattern I) ✓

Priority: **Medium** — Completed (addressed as part of Phase 1)

### 2.1 Use suggestion endpoints for dropdowns ✓

- [x] All category filter/form contexts now use `listCategoriesForSuggestions()` instead of `listCategories()`
- [x] Projects list, project edit, project new — all three switched

---

## Phase 3 - Tab Content Lazy-Mount (Pattern A) ✓

Priority: **Low** — Completed

### 3.1 Emails detail: migrated to DetailPageShell ✓

- [x] Migrated from PageHeader + TabsRoot/TabsList/TabsTrigger/TabsContent to DetailPageShell
- [x] Replaced manual AlertDialog + DropdownMenu delete flow with EntityActionsMenu
- [x] Dynamic `emailTabs` array computed from email content (conditional html/text/source/headers)
- [x] Copy actions for ID, from address, and to addresses moved to EntityActionsMenu copies

### 3.2 Scheduled tasks detail: migrated to DetailPageShell ✓

- [x] Migrated from PageHeader + TabsRoot to DetailPageShell with tabs + tabContent
- [x] DetailMeta with ID and enabled/disabled Pill replaces PageHeaderMeta
- [x] Tab lazy-mount now automatic via DetailPageShell's mountedTabsSet pattern

---

## Phase 4 - Dead Endpoint Cleanup (Pattern D) ✓

Priority: **Low** — Completed

### 4.1 Suggestion endpoints ✓

- [x] `listCategoriesForSuggestions` — now actively used (3 callers after Phase 1). Kept.
- [x] `listProjectsForSuggestions` — removed (no callers, no planned use)

### 4.2 Unused restore commands ✓

- [x] `restoreCategory` — removed (no restore UI exists)
- [x] `restoreProject` — removed (no restore UI exists)
- `restoreTask` — does not exist (roadmap entry was incorrect)

Note: Media has a working restore flow via `MediaActionsMenu`. If restore is needed for categories/projects in the future, the commands can be re-added following that pattern.

### 4.3 Unused activity endpoint ✓

- [x] `listActivityForEntity` — removed (only `listActivity` and `listActivityForUser` are used)

---

## Phase 5 - Enable Global Auto-Fetch (Pattern K)

Priority: **High**

Every page (27 total) has both a redundant `getToken` option and a manual `tryFetch` `$effect`, because `configureAuth()` in the layout does not yet provide `getAuthLoading`/`getCurrentUser`. The `getToken` option is already redundant (global config provides it), and enabling auto-fetch would eliminate all 27 manual `$effect` blocks.

### 5.1 Extend `configureAuth()` in layout

- [ ] Add `getAuthLoading` and `getCurrentUser` to the `configureAuth()` call in `+layout.svelte`

File: `acme-admin/src/routes/(app)/+layout.svelte` (lines 20-23)

Current:
```typescript
configureAuth({
    getToken: () => auth.getToken(),
    onRefresh: auth.getRefreshHandler()
});
```

Target:
```typescript
configureAuth({
    getToken: () => auth.getToken(),
    onRefresh: auth.getRefreshHandler(),
    getAuthLoading: () => $authLoading,
    getCurrentUser: () => $currentUser
});
```

### 5.2 Remove redundant per-page `getToken` and manual `tryFetch` from all pages

- [ ] Remove `getToken: () => auth.getToken()` from every `useAuthenticatedData` options object
- [ ] Remove every `$effect(() => { pageData.tryFetch($authLoading, $currentUser); })` block
- [ ] Remove every `$effect(() => { *.tryFetch($authLoading, $currentUser); })` block (secondary data like `sessionsData`, `activityData`, `jobsData`) — replace with conditional `activeTab` gating via `queryKey` or keep as-is if tab-gated
- [ ] Remove `authLoading` and `currentUser` from imports where no longer needed
- [ ] Keep `import { auth }` where `auth.getToken()` is used for mutations

Affected files (27):

| # | File | Notes |
|---|------|-------|
| 1 | `categories/+page.svelte` | Standard — remove getToken + tryFetch |
| 2 | `categories/[categoryId]/+page.svelte` | Standard |
| 3 | `categories/[categoryId]/edit/+page.svelte` | Standard |
| 4 | `projects/+page.svelte` | Has URL-change refetch $effect — keep that, remove tryFetch |
| 5 | `projects/[projectId]/+page.svelte` | Has URL-change refetch $effect — keep that, remove tryFetch |
| 6 | `projects/[projectId]/edit/+page.svelte` | Standard |
| 7 | `projects/[projectId]/tasks/new/+page.svelte` | Standard |
| 8 | `projects/[projectId]/tasks/[taskId]/+page.svelte` | Standard |
| 9 | `projects/[projectId]/tasks/[taskId]/edit/+page.svelte` | Standard |
| 10 | `users/+page.svelte` | Also has unguarded filter $effect (Phase 6) |
| 11 | `users/[userId]/+page.svelte` | Has tab-gated tryFetch for sessions/activity — convert to separate useAuthenticatedData with tab gating |
| 12 | `users/[userId]/edit/+page.svelte` | Standard |
| 13 | `media/+page.svelte` | Has URL-change refetch $effect — keep that, remove tryFetch |
| 14 | `media/trash/+page.svelte` | Standard |
| 15 | `media/[mediaId]/+page.svelte` | Standard |
| 16 | `account/+page.svelte` | Standard |
| 17 | `account/2fa/+page.svelte` | Standard |
| 18 | `account/passkeys/+page.svelte` | Standard |
| 19 | `account/password/+page.svelte` | Standard |
| 20 | `system/audit/+page.svelte` | Has URL-change refetch + wasteful refetch (Phase 7) |
| 21 | `system/emails/+page.svelte` | Has URL-change refetch $effect — keep that, remove tryFetch |
| 22 | `system/emails/[id]/+page.svelte` | Standard |
| 23 | `system/errors/+page.svelte` | Also has unguarded filter $effect (Phase 6) |
| 24 | `system/jobs/+page.svelte` | Also has unguarded filter $effect (Phase 6) |
| 25 | `system/jobs/[id]/+page.svelte` | Standard |
| 26 | `system/scheduled-tasks/+page.svelte` | Has URL-change refetch $effect — keep that, remove tryFetch |
| 27 | `system/scheduled-tasks/[id]/+page.svelte` | Has tab-gated tryFetch for jobsData — convert |

---

## Phase 6 - Guard Filter-Change Refetch Effects (Pattern J variant)

Priority: **High**

Three pages use local state filters (not URL params) with an unguarded `$effect` that calls `refetch()` whenever `$currentUser` is truthy. This fires on mount alongside `tryFetch`, causing a double-fetch on initial page load.

### 6.1 Users list: guard filter-change refetch

- [ ] Add previous-value guard or use `queryKey` option to prevent mount-time double-fetch

File: `acme-admin/src/routes/(app)/users/+page.svelte` (lines 56-65)

Current:
```typescript
$effect(() => {
    void page;
    void roleFilter;
    void statusFilter;
    void searchQuery;
    void displayNameQuery;
    if ($currentUser) {
        pageData.refetch();
    }
});
```

The `refetch()` call fires on mount when `$currentUser` is already set, racing with `tryFetch`. Fix: add an `initialized` flag set after first successful fetch, or use `queryKey` option on `useAuthenticatedData`.

### 6.2 Error logs list: guard filter-change refetch

- [ ] Same pattern — add previous-value guard

File: `acme-admin/src/routes/(app)/system/errors/+page.svelte` (lines 47-52)

### 6.3 Jobs list: guard filter-change refetch

- [ ] Same pattern — add previous-value guard

File: `acme-admin/src/routes/(app)/system/jobs/+page.svelte` (lines 55-60)

---

## Phase 7 - Audit Page Wasteful Refetch (Note)

Priority: **Low**

File: `acme-admin/src/routes/(app)/system/audit/+page.svelte`

The audit page has a correctly guarded URL-change `$effect` that calls `refetch()`, but the `useAuthenticatedData` fetcher ignores URL params — it always fetches `{ limit: 100 }` with no filtering. Filtering for `action` and `resource_type` is done client-side. This means every URL filter change triggers a full round-trip API refetch that returns the same data.

Options:
- [ ] Remove the URL-change refetch `$effect` (since filtering is client-side, refetch is a no-op)
- [ ] OR move filtering to the API (add query params to `listActivity`) and keep the refetch

---

## Patterns Not Found (Clean)

The following patterns from the sweep were audited and found clean:

| Pattern | Status | Notes |
|---------|--------|-------|
| **C** Duplicate identical requests | Clean | No sibling components call the same endpoint |
| **E** Exhaustive pagination on load | Clean | No `listAll`/`fetchAll`/paginated-exhaust patterns found |
| **F** N+1 client-side fan-out | Clean | No `Promise.all(items.map(async ...))` in data loaders |
| **H** Missing supplementary data on list DTOs | Clean | List endpoints include counts and labels directly |
| **J** Unguarded queryKey refetch (tab-mounted) | Clean | No `queryKey` pattern used; tab-mounted components use `tryFetch` gated on `activeTab` (correct) |

Note: Pattern J's core concern (tab-mounted components refetching on sibling tab URL changes) is not present. The 3 issues in Phase 6 are a variant — unguarded local-state filter effects on non-tab pages causing double-fetch on mount.

---

## Notes

### Note 1: DetailPageShell lazy-mount is well-implemented

The `DetailPageShell` component uses a `mountedTabsSet` pattern that properly lazy-mounts tab content on first activation and keeps it mounted thereafter. All five pages now using DetailPageShell (users, categories, media, emails, scheduled tasks) benefit from this automatically.

### Note 2: Data deferral patterns are well-structured

Pages with tabbed data (users, media, scheduled tasks) properly defer secondary data fetches using conditional `$effect` blocks that check `activeTab`. This is the correct pattern and prevents unnecessary API calls on page load.

### Note 3: URL-change refetch guards are consistently applied

Six list pages use a `previousUrl` pattern to guard URL-change refetch effects: projects list, project detail, categories list, media list, emails list, and scheduled tasks list. The pattern initializes `previousUrl` to `null`, sets it in `onSuccess`, and only calls `refetch()` when `previousUrl !== null && previousUrl !== currentUrl`. This prevents double-fetch on mount and avoids spurious refetches.
