# 004 - Retained Underlay Surface Contract

Status: active
Owner: repo maintainers
Updated: 2026-09-03
Roadmap refs: g01.007
Governing refs: docs/architecture/product-guardrails.md, docs/policy/001-working-rules.md, docs/roadmaps/g01/006-poodle-underlay-coexistence-proof.md

## Purpose

Freeze the approved retained Underlay surface of `acme-admin` after the
completed `g01.006` Poodle coexistence proof. This contract is the boundary
of record for the reference implementation. Downstream app rollouts use
Poodle directly for primitives and simple composites, and retain Underlay
only where a retained group below names explicit structural, workflow-heavy,
or data-heavy ownership.

Path references are repo-root-relative to the `underlay-reference`
checkout. Classifications come from live `acme-admin` source inspection on
2026-09-03 with Underlay v0.9.7 and Poodle 0.2.2 resolved.

## Downstream rule

- Use Poodle primitives and composites directly for foundational UI: theme
  bootstrap, buttons, inputs, selects, dialogs, drawers, toast presentation,
  metric tiles, and icons.
- Retain Underlay only where a retained group below says Underlay still owns
  the surface: the structural shell and the workflow-heavy or data-heavy
  template and runtime stacks.
- Reference-app composition and diagnostic surfaces are app-local; do not
  copy them into downstream apps.
- Do not migrate group-four candidates downstream by default. They wait for
  an explicit Poodle review lane decision.
- Changing this contract requires a roadmap milestone, not a drive-by
  migration.

## Retained groups

### Group 1 — Structural shell

- Category: intentionally retained structural shell.
- Owner: Underlay framework; consumed by `acme-admin`.
- Why it remains: the coexistence model keeps Poodle owning theme bootstrap
  and presentation primitives while Underlay keeps the app spine — security
  headers, auth session runtime, shared navigation contract, feedback
  runtime state, and the stylesheets that retained template surfaces
  require. This split is the pattern downstream apps are expected to copy.

Representative surfaces:

- Server security shell: `apps/acme-admin/src/hooks.server.ts` imports
  `underlay/server` (`createCspConfig`, `generateNonce`,
  `createCspResolveOptions`, `applyCspHeaders`) and maps
  `UnderlayHttpError` in `handleServerError`.
- Auth runtime: `apps/acme-admin/src/routes/(app)/+layout.svelte` calls
  `configureAuth`; `useAuthenticatedData` drives 12 authenticated route
  surfaces; `apps/acme-admin/src/lib/utils/auth-tokens.ts` uses
  `createAuthCookieHelpers`; `apps/acme-admin/src/routes/(auth)/login/+page.svelte`
  and `apps/acme-admin/src/routes/(app)/account/passkeys/+page.svelte` use
  `underlay/utils/webauthn`.
- Navigation contract: `underlay/client/navigation` (`gotoWithContext`,
  `navigateOnCancel`) across 17 files and `underlay/runtime/navigation`
  (`NavigationContext`, `computeBackInfo`, `consumeNavigationContext`,
  `getBackButtonInfo`) across 15 files — the shared back/cancel behavior of
  every detail and edit flow.
- Feedback runtime: `apps/acme-admin/src/routes/(app)/+layout.svelte` creates
  the Underlay toast store and sets `UNDERLAY_TOASTS_CONTEXT_KEY`; Poodle
  `ToastHost` renders it. `copyToClipboard` serves the account routes.
- Route protection: `underlay/client/route-protection` `resolveRedirectTo`
  guards the six `new`/`edit` routes under users, projects, and categories.
- Stylesheets: `apps/acme-admin/src/routes/+layout.svelte` imports
  `underlay/styles/base.css`, `tokens.css`, and `forms.css` before the
  Poodle theme shell so retained template pages keep their form and base
  styling under Poodle-owned theming.

### Group 2 — Workflow-heavy and data-heavy template stacks

- Category: intentionally retained data-heavy or workflow-heavy surface.
- Owner: Underlay framework; acme data wiring is app-local.
- Why it remains: these stacks orchestrate paged loading, entity CRUD
  workflows, versioned media state machines, and system operations. Poodle
  already owns their primitives; replacing the orchestration layer is not a
  primitive migration and stays out of scope until a roadmap decision says
  otherwise.

Representative surfaces:

- Entity template stack: `underlay/templates` (`EntityListPage`,
  `EntityListCard`, `EntityDetailPage`, `EntityDetail`, `EntityDetailModule`,
  `EntityAttributeList`, `EntityFormPage`, `EntityActionsMenu`,
  `UserForm`, `UsersListPage`, `toPagedListResult`) with `underlay/patterns`
  (`createEntityListState`, `createPageListQueryState`) and
  `underlay/client/query` (`QueryParams`). Representative app wiring:
  `apps/acme-admin/src/lib/lists/UsersList.svelte`,
  `apps/acme-admin/src/lib/lists/CategoriesList.svelte`,
  `apps/acme-admin/src/lib/lists/LabelsList.svelte`,
  `apps/acme-admin/src/lib/lists/ProjectsListPage.svelte`,
  `apps/acme-admin/src/lib/lists/TasksListPage.svelte`,
  `apps/acme-admin/src/lib/cards/`, and the matching
  `apps/acme-admin/src/routes/(app)/` route files.
- Media workflow stack: `underlay/templates` (`MediaListPage`,
  `MediaDetailWorkflowPage`, `MediaEditDialog`, `MediaUploadWorkflowPage`,
  `MediaVersionsList`, `MediaVersionPreviewDialog`,
  `MediaVersionActionDialogs`, `MediaRenditionsSection`, `MediaPreviewTab`,
  `MediaReplaceFileForm`, `MediaUsageList`, `MediaActionsMenu`,
  `MediaListCard`) with `underlay/runtime/media/upload|detail|types` and
  app wiring in `apps/acme-admin/src/lib/utils/upload-pipeline.ts` and
  `apps/acme-admin/src/lib/utils/media-usage-resolution.ts`. Representative
  routes: `apps/acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte`
  and `apps/acme-admin/src/routes/(app)/media/upload/+page.svelte`.
- System operations stack: `underlay/templates` (`SystemIndexPage`,
  `SystemJobListPage`, `SystemJobDetailPage`, `SystemAuditLogListPage`,
  `ErrorLogListPage`, `ErrorLogDetailPage`, `SystemScheduledTasksListPage`,
  `SystemScheduledTaskDetailPage`, `SystemMediaTrashListPage`) with their
  loader types. Representative app wiring:
  `apps/acme-admin/src/lib/lists/JobsList.svelte`,
  `apps/acme-admin/src/lib/lists/AuditLogList.svelte`,
  `apps/acme-admin/src/lib/lists/ErrorLogList.svelte`,
  `apps/acme-admin/src/lib/lists/ScheduledTasksList.svelte`,
  `apps/acme-admin/src/lib/lists/MediaTrashList.svelte`, and the
  `apps/acme-admin/src/routes/(app)/system/` route family.
- Auth workflow patterns: `underlay/patterns` (`LoginPage`,
  `ForgotPasswordFlow`, `PasswordRequirements`, `SpaFormResult`) rendering
  `apps/acme-admin/src/routes/(auth)/login/+page.svelte` and
  `apps/acme-admin/src/routes/(auth)/forgot-password/+page.svelte`.
- Relations and selection runtime: `underlay/runtime/relations`
  (`createLocalSearchFns`, `SuggestionOptions`) and
  `underlay/runtime/selection` (`createSelectionHistory`, `SelectionHistory`)
  powering `apps/acme-admin/src/lib/forms/ProjectCategorySelector.svelte`,
  `apps/acme-admin/src/lib/forms/ProjectForm.svelte`, and
  `apps/acme-admin/src/lib/stores/selection-history.ts`.
- Nightfire rich-text stack: `underlay/nightfire/editor` (`NightfireEditor`),
  `underlay/nightfire/validation`, `underlay/nightfire/renderer`, and
  `underlay/nightfire/media-locator`, consumed by
  `apps/acme-admin/src/lib/forms/ProjectForm.svelte`,
  `apps/acme-admin/src/lib/forms/TaskForm.svelte`, the project and task
  detail routes, and the note node registrations in
  `packages/acme-ui/src/nightfire/notes/` that acme-admin loads through the
  `@acme/ui/editor`, `@acme/ui/validation`, and `@acme/ui/render` entries.

### Group 3 — Reference-app domain and diagnostic surface

- Category: reference-app/domain-specific surface.
- Owner: `acme-admin`.
- Why it remains: these surfaces exist to demonstrate or inspect the
  reference implementation itself. They are app-local by design and are not
  part of the reusable boundary downstream apps copy.

Representative surfaces:

- `apps/acme-admin/src/routes/(app)/system/poodle-gap-review/+page.svelte` —
  a Poodle-first review page (Poodle `PageHeader`, `ListContainer`, `Pill`,
  `Card`, `Select`) whose only Underlay dependency is
  `underlay/runtime/collections` `createClientPagination`. It exists to
  review the remaining ambiguous workflow surfaces against the retained
  boundary.
- The domain composition layer — `apps/acme-admin/src/lib/cards/`,
  `apps/acme-admin/src/lib/lists/`, `apps/acme-admin/src/lib/forms/`,
  `apps/acme-admin/src/lib/menus/`, and `apps/acme-admin/src/lib/ui/` — is
  acme data wiring (loaders calling `@api-client`, acme nav items feeding
  the Underlay `AdminNavList` and `AdminUserMenu` templates in
  `apps/acme-admin/src/lib/ui/AdminNavList.svelte` and
  `apps/acme-admin/src/lib/ui/AdminUserMenu.svelte`). The Underlay
  dependencies inside this layer are group 1 and group 2 surfaces; the
  wiring itself is app-local.

### Group 4 — Future Poodle review candidates (not part of g01.007)

- Category: candidate for future Poodle review, but not part of `g01.007`.
- Owner: Poodle review lane (future decision); currently consumed by
  `acme-admin`.
- Why they remain today: each is small, self-contained, and not structural,
  workflow-heavy, or data-heavy on its own. Nothing here blocks the frozen
  boundary, and no migration is scheduled in this milestone.

Candidates as of 2026-09-03:

- `underlay/runtime/feedback` toast store — a coexistence seam already
  exists (Underlay store, Poodle `ToastHost` presentation); a Poodle
  feedback contract could absorb the store.
- `underlay/utils/slug` (`slugify`, `isValidSlugFormat`, `isReservedSlug`)
  — simple string utilities with two consumers.
- `underlay/utils/html` `sanitizeSvgHtml` — single consumer on the 2FA page.
- `underlay/runtime/browser` `detectBrowserTimezone` — single consumer on
  the account page.
- `underlay/runtime/collections` `createClientPagination` — single consumer,
  the otherwise Poodle-first gap-review page.
- `underlay/patterns` `PasswordRequirements` — small form helper on the
  auth and account flows.

## Boundary summary

The retained Underlay surface of the reference admin is exactly groups 1 and
2. Poodle owns every foundational primitive and composite the coexistence
proof moved in `g01.006`; this audit found no residual Underlay primitive or
simple-composite usage to chase. Group 3 is app-local reference material.
Group 4 is parked for a future review lane and must not drift into a
migration wave without a roadmap.

## Next Task

Orchestrator reviews this contract at the Card 001 PR head; downstream
rollout planning consumes it only after that review.
