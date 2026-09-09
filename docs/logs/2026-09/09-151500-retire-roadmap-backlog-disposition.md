# Retire Roadmap Backlog: Disposition Manifest

Date: 2026-09-09
Handoff: `docs/handoffs/20260909-151020-retire-roadmap-backlog.md`
Lane: one-time Northstar roadmap-backlog retirement in underlay-reference.
Migration is not approval: nothing below authorizes execution.

## Backlog items

| Removed file | Current meaning | Disposition |
| --- | --- | --- |
| `docs/roadmaps/backlog/README.md` | Pure scaffolding ("candidate milestones not scheduled yet"); no candidate content | Removed without a triage note; meaning absorbed by `docs/triage/README.md` |
| `docs/roadmaps/backlog/advanced-features.md` § OAuth/SSO | Deferred candidate, unapproved | Triage: `docs/triage/20260909-151500-retired-roadmap-backlog-advanced-features.md` §1 |
| `docs/roadmaps/backlog/advanced-features.md` § Real-time Features | Deferred candidate, unapproved | Same triage note §2 |
| `docs/roadmaps/backlog/advanced-features.md` § Multi-tenancy Example | Deferred candidate, unapproved | Same triage note §3 |
| `docs/roadmaps/backlog/advanced-features.md` § Advanced Media (incl. explicitly blocked video upload) | Deferred candidate, unapproved | Same triage note §4 |
| Same file §§ Priority Notes, Implementation Notes | Candidate context, not accepted design | Preserved as constraints/references inside the triage note; nothing promoted to contracts |

No already-approved executable work was found: no merge into any `gNN.NNN`
task, no new task created. No durable rules or accepted design were found:
nothing promoted to architecture, policy, or other owning surfaces.

## Inbound links and surface references

| Location | Before | After |
| --- | --- | --- |
| `docs/roadmaps/README.md` § Rules | "Backlog items that are not active milestones belong in `backlog/`" | Roadmaps hold only promoted executable tasks; unresolved/deferred candidates live in `docs/triage/` until promotion |
| `docs/roadmaps/README.md` § Index | Link to `backlog/README.md` | Link to `../triage/README.md` |
| `docs/architecture/000-overview.md` project tree | `backlog/` unscheduled candidate milestones | `triage/` deferred candidates until promotion |
| `docs/roadmaps/g01/001-reference-completion.md` § Phase 9 + § Priority Order | Absolute link to `acme-docs/roadmaps/backlog/advanced-features.md` (already stale: `acme-docs/` moved to root `docs/`); summary listed "Phase 9 → Backlog" | Relative link to the triage note and "→ Deferred (see triage note)"; record otherwise untouched (imported historical milestone) |
| `docs/AGENTS.md` | New triage rule beside the existing roadmap rule | Unresolved/deferred candidates go in `triage/`; roadmaps stay executable-only |
| `docs/README.md` | No triage pointer | Triage bullet in structure and how-to-use |

## Retained historical exceptions (unchanged)

- `docs/roadmaps/g01/002-security-audit.md` progress snapshot (2026-02-09):
  "Remaining backlog" is ordinary prose in a dated snapshot inside an
  imported historical milestone, not a reference to the retired surface.
- `docs/logs/2026-03/06-143812-underlay-reference-effigy-adoption-closeout.md`:
  "doctor scan backlog" is generic wording in a closed log.
- Closed handoffs under `docs/handoffs/` and execution logs under
  `docs/logs/`: immutable records; all cited `roadmaps/g01/` paths still
  resolve.

## Scaffolding, templates, fixtures, checkers

- No backlog-item templates, fixtures, or checker rules referencing the
  backlog exist: no matches outside `docs/` for `backlog` (checked apps,
  packages, config, `effigy.toml`, root `AGENTS.md`/`README.md`).
- `effigy.toml` `qa:docs` heading gates still satisfied: required headings
  kept on both roadmap front doors.

## Approved frontier

Unchanged: `g01` active, next roadmap ID `g01.014`, no implementation card
ready. No generation rolled, no task approved, no product code touched.

## Verification

- `find docs -type d -name backlog -print` returns nothing.
- `effigy acme-docs/qa:docs`, `effigy acme-docs/qa:northstar`,
  `effigy acme-docs/validate`, `git diff --check`.
