# Wasteful Endpoint Calls Remediation

This roadmap captures findings from running Underlay sweep **021 (Wasteful Endpoint Calls)** against this reference repo.

Run date: 2026-02-15

Source sweep: `underlay/docs/sweeps/021-wasteful-endpoint-calls-sweep.md`

## Sweep Summary

Pages/components audited: 18
Critical issues: 0
High issues: 1
Medium issues: 5
Low issues: 6
Notes: 2

This codebase is relatively clean. The reference app has no N+1 fan-out, no exhaustive pagination, and no duplicate sibling requests. The main issues are:

1. Eager category data fetches for filter dropdowns (should lazy-load)
2. Emails detail page mounts all tab content simultaneously (low impact — single API call)
3. Heavy DTO used where lightweight suggestion endpoints already exist
4. Several dead/vestigial command exports

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

## Patterns Not Found (Clean)

The following patterns from the sweep were audited and found clean:

| Pattern | Status | Notes |
|---------|--------|-------|
| **C** Duplicate identical requests | Clean | No sibling components call the same endpoint |
| **E** Exhaustive pagination on load | Clean | No `listAll`/`fetchAll`/paginated-exhaust patterns found |
| **F** N+1 client-side fan-out | Clean | No `Promise.all(items.map(async ...))` in data loaders |
| **H** Missing supplementary data on list DTOs | Clean | List endpoints include counts and labels directly |

---

## Notes

### Note 1: DetailPageShell lazy-mount is well-implemented

The `DetailPageShell` component uses a `mountedTabsSet` pattern that properly lazy-mounts tab content on first activation and keeps it mounted thereafter. All five pages now using DetailPageShell (users, categories, media, emails, scheduled tasks) benefit from this automatically.

### Note 2: Data deferral patterns are well-structured

Pages with tabbed data (users, media, scheduled tasks) properly defer secondary data fetches using conditional `$effect` blocks that check `activeTab`. This is the correct pattern and prevents unnecessary API calls on page load.
