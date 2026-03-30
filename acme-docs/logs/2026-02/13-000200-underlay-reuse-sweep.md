# Sweep 002 - Underlay Reuse Sweep (acme-admin + acme-front)

Date: 2026-02-13

Scope reviewed:

- `acme-admin/src`
- `acme-front/src`
- Underlay canonical exports from:
  - `underlay/ts/src/components/index.ts`
  - `underlay/ts/src/patterns/index.ts`

## Adoption snapshot

- Underlay import density is high in both apps, especially for forms, list controls, cards, dialogs, and auth UI.
- No direct `bits-ui` imports were found in consuming app source.
- Most list-heavy routes already use shared `FilterBar`, `OrderBy`, `DataTable`, `ListCard`, and `ListGrid`.

## Findings

### [FORMS] [MEDIUM] Raw color input duplicates shared color widget

- **Location:** `acme-admin/src/lib/forms/CategoryForm.svelte:107`
- **Existing shared alternative:** `@decodelabs/underlay/components` `ColorPicker`
- **Why this is duplicate:** A native `<input type="color">` is used directly inside a shared `Field`, while Underlay already exposes a color picker primitive.
- **Classification:** Must migrate
- **Remediation plan:** Replace native color input with `ColorPicker`, keep field name/value contract unchanged.
- **Owner:** Frontend/platform
- **Target date:** Next UI consistency pass
- **Status:** Resolved (migrated to shared color picker pattern)

### [FORM DIALOGS] [HIGH] Create-project flow uses `AlertDialog` for form input

- **Location:** `acme-front/src/routes/(app)/dashboard/+page.svelte:113`
- **Existing shared alternative:** `@decodelabs/underlay/patterns` `FormDialog` (or `Dialog` + `Form` + `FormActions`)
- **Why this is duplicate:** Input fields are embedded in `AlertDialog`, which is intended for confirm/destructive prompts; this diverges from shared form dialog pattern.
- **Classification:** Must migrate
- **Remediation plan:** Switch to `FormDialog` and shared submit/cancel action layout.
- **Owner:** Frontend/platform
- **Target date:** Next frontend sweep batch
- **Status:** Resolved (migrated to form-dialog pattern)

### [FORM DIALOGS] [HIGH] Create-task flow uses `AlertDialog` for form input

- **Location:** `acme-front/src/routes/(app)/projects/[projectId]/+page.svelte:221`
- **Existing shared alternative:** `FormDialog` (preferred)
- **Why this is duplicate:** Multi-field form is implemented in an alert-confirm container instead of shared form dialog primitives.
- **Classification:** Must migrate
- **Remediation plan:** Replace with `FormDialog` and shared form action slots.
- **Owner:** Frontend/platform
- **Target date:** Next frontend sweep batch
- **Status:** Resolved (migrated to form-dialog pattern)

### [FORM DIALOGS] [MEDIUM] Custom dialog footer bypasses shared form actions

- **Location:** `acme-admin/src/routes/(app)/users/[userId]/+page.svelte:445`
- **Existing shared alternative:** `FormDialog` or `Dialog` + `FormActions`
- **Why this is duplicate:** Dialog uses ad-hoc footer markup and button layout (`user-view__dialog-footer`) rather than shared footer conventions.
- **Classification:** Must migrate
- **Remediation plan:** Move role-change modal to `FormDialog` or use `FormActions` in footer snippet.
- **Owner:** Frontend/platform
- **Target date:** Next admin UX consistency pass
- **Status:** Resolved (uses `FormActions` pattern)

### [LIST SELECTION] [LOW] Inline checkbox selection in list cards

- **Location:**
  - `acme-admin/src/lib/cards/ProjectListCard.svelte:77`
  - `acme-admin/src/routes/(app)/projects/[projectId]/+page.svelte:529`
- **Existing shared alternative:** No direct shared checkbox-selection list-card primitive found.
- **Why this is duplicate:** Selection-mode checkbox behavior is implemented locally in card media slots.
- **Classification:** Allowed exception (for now)
- **Remediation plan:** Keep local implementation; optionally evaluate extending Underlay with a reusable selectable-list-card pattern.
- **Owner:** Frontend/platform
- **Target date:** Backlog (optional enhancement)
- **Status:** Accepted exception (unchanged)

## Resolution update (2026-02-13)

- Must-migrate findings from this sweep were remediated in the current roadmap execution.
- The remaining low-severity selectable-card behavior remains an accepted exception pending potential Underlay extraction.

## Reuse sweep summary

- Candidates found: 8 (raw-input candidates + dialog pattern candidates)
- Must migrate: 4
- Extend then migrate: 0
- Allowed exceptions: 1

## Follow-up

- Shared component enhancements needed:
  - Optional: consider a reusable selectable-list-card checkbox pattern.
- App-level migrations queued:
  1. Migrate front create dialogs from `AlertDialog` to `FormDialog`.
  2. Replace admin category color native input with `ColorPicker`.
  3. Normalize admin role dialog footer to `FormActions`/`FormDialog`.

## Guardrail status (Step 6)

- Existing status:
  - Good: no `bits-ui` direct usage found in consuming apps.
  - Gap: app `AGENTS.md` files do not explicitly include a "reuse Underlay first" rule.
- Suggested guardrails to add:
  - PR checklist item: "Did this recreate an existing Underlay component/pattern?"
  - CI grep checks:
    - `rg -n "from \"bits-ui\"|from 'bits-ui'" src`
    - `rg -n "<(input|select|textarea)\\b" src/routes src/lib/forms`
