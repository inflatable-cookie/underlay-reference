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

## Phase 1 - Eager Filter Data Fetches (Pattern G + B)

Priority: **High**

These are the most impactful fixes — removing unnecessary API calls from page load.

### 1.1 Projects list: lazy-load category filter dropdown

- [ ] Remove `adminCommands.listCategories()` from `useAuthenticatedData` callback
- [ ] Switch category filter `Select` to use `loadItems` prop with async loader
- [ ] Use `listCategoriesForSuggestions` (already exists, currently unused) for lighter payload

Files:
- `acme-admin/src/routes/(app)/projects/+page.svelte` (lines 46-49, 165-171)

Current behavior: `listCategories()` returns `CategoryWithCounts[]` (with `projectCount` subquery) on every page load. Categories are only used for the filter dropdown — the `projectCount` field is discarded at line 167.

Expected behavior: Category options load on first dropdown open via `Select.loadItems`. No category API call on page load.

### 1.2 Project edit form: lazy-load category dropdown

- [ ] Remove `adminCommands.listCategories()` from `useAuthenticatedData` callback
- [ ] Switch `ProjectForm` category selector to use `loadItems` or pass a loader function
- [ ] Use `listCategoriesForSuggestions` for lighter payload

Files:
- `acme-admin/src/routes/(app)/projects/[projectId]/edit/+page.svelte` (line 33)
- `acme-admin/src/routes/(app)/projects/new/+page.svelte` (likely same pattern)
- `acme-admin/src/lib/forms/ProjectForm.svelte`

Current behavior: Full `CategoryWithCounts[]` fetched eagerly on page load. `ProjectForm.categoryToSelectable()` only uses `id`, `name`, `description`.

Expected behavior: Category options lazy-load when user opens the selector.

### 1.3 Task edit/new forms: lazy-load label selector

- [ ] Remove `adminCommands.listLabels()` from `useAuthenticatedData` callback
- [ ] Defer label loading until label section is visible or interacted with

Files:
- `acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/edit/+page.svelte` (line 34)
- `acme-admin/src/routes/(app)/projects/[projectId]/tasks/new/+page.svelte` (line 31)

Current behavior: All project labels fetched on page load even though labels are an optional section of the form.

Expected behavior: Labels load lazily when the label section is expanded or first interacted with.

---

## Phase 2 - DTO Right-Sizing (Pattern I)

Priority: **Medium**

### 2.1 Use suggestion endpoints for dropdowns

- [ ] Replace `listCategories()` calls in filter/form contexts with `listCategoriesForSuggestions()`
- [ ] Audit `ProjectForm` to accept `Category[]` instead of `CategoryWithCounts[]`

Files:
- `acme-admin/src/routes/(app)/projects/+page.svelte`
- `acme-admin/src/routes/(app)/projects/[projectId]/edit/+page.svelte`
- `acme-admin/src/routes/(app)/projects/new/+page.svelte`
- `acme-admin/src/lib/forms/ProjectForm.svelte`

Current behavior: `listCategories()` returns `CategoryWithCounts` with a `projectCount` subquery. Filter dropdowns and form selectors discard `projectCount`.

Expected behavior: Dropdown/form contexts use `listCategoriesForSuggestions()` which returns lightweight `Category[]` without count subqueries.

---

## Phase 3 - Tab Content Lazy-Mount (Pattern A)

Priority: **Low**

### 3.1 Emails detail: migrate to DetailPageShell or add lazy-mount

- [ ] Either migrate to DetailPageShell (which has built-in lazy-mount) or add conditional rendering

File:
- `acme-admin/src/routes/(app)/system/emails/[id]/+page.svelte`

Current behavior: All four tab panels (html, text, source, headers) mount simultaneously. Uses raw `TabsRoot`/`TabsContent` without lazy-mount guards.

Mitigating factor: Single API call returns all tab data — no wasted network requests. The waste is purely DOM rendering (inactive tabs render hidden content). Impact is low because email data is typically small.

Expected behavior: Only the active tab's content renders. DetailPageShell migration would fix this automatically.

### 3.2 Scheduled tasks detail: migrate to DetailPageShell

- [ ] Migrate to DetailPageShell for consistency (data deferral is already correct)

File:
- `acme-admin/src/routes/(app)/system/scheduled-tasks/[id]/+page.svelte`

Current behavior: Uses raw `TabsRoot`/`TabsContent` but data fetching is already properly deferred (jobs tab data only loads on tab activation). However, the `TabsContent` for job-runs still mounts its DOM even when the tab is inactive.

Expected behavior: Migrate to DetailPageShell for lazy-mount and consistent patterns.

---

## Phase 4 - Dead Endpoint Cleanup (Pattern D)

Priority: **Low**

### 4.1 Remove or implement unused suggestion endpoints

- [ ] Decide: use `listCategoriesForSuggestions` in dropdown contexts (see Phase 1) or remove
- [ ] Decide: use `listProjectsForSuggestions` somewhere or remove

Files:
- `acme-client/src/commands/admin/category-commands.ts` (`listCategoriesForSuggestions`)
- `acme-client/src/commands/admin/project-commands.ts` (`listProjectsForSuggestions`)

Status: Both exported from `admin-commands.ts` but never called from acme-admin or acme-front.

### 4.2 Remove or implement unused restore commands

- [ ] Decide: implement restore UI for soft-deleted categories, projects, and tasks, or remove commands

Files:
- `acme-client/src/commands/admin/category-commands.ts` (`restoreCategory`)
- `acme-client/src/commands/admin/project-commands.ts` (`restoreProject`)
- `acme-client/src/commands/admin/task-commands.ts` (`restoreTask`)

Status: All three exported but no UI path exists to invoke them. Categories and projects can be soft-deleted via the admin UI but there is no restore action.

Note: Media has a working restore flow via `MediaActionsMenu`. The pattern exists but wasn't extended to categories/projects.

### 4.3 Remove or implement unused activity endpoint

- [ ] Decide: implement entity-specific activity view or remove

Files:
- `acme-client/src/commands/admin/activity-commands.ts` (`listActivityForEntity`)

Status: Exported but never called. Only `listActivity` (global feed) and `listActivityForUser` (user-specific) are used.

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

The `DetailPageShell` component uses a `mountedTabsSet` pattern that properly lazy-mounts tab content on first activation and keeps it mounted thereafter. All three pages migrated to DetailPageShell (users, categories, media) benefit from this automatically.

### Note 2: Data deferral patterns are well-structured

Pages with tabbed data (users, media, scheduled tasks) properly defer secondary data fetches using conditional `$effect` blocks that check `activeTab`. This is the correct pattern and prevents unnecessary API calls on page load.
