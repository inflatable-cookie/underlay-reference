# Papercuts

Small, actionable friction found during agent work. Agents append entries when
they hit a solvable hurdle; they do not stop the current task to fix one.

## Open

<!-- Keep entries short. Append newest entries at the top. Do not include secrets. -->

## Closed

### [x] Update the bundle docs-link selector — 2026-08-11
- Friction: `effigy qa:docs` and `effigy acme-docs/qa:docs` call the removed `check-links` argument.
- Impact: the aggregate docs gate fails before running otherwise valid link checks.
- Fix: `acme-docs/effigy.toml` now uses `docs check links` / `forbidden` /
  `headings` (space form). Root `qa:docs` is wired to that catalog via the
  Underlay Effigy bundle.
- Surface: Underlay Effigy bundle docs tasks / `acme-docs/effigy.toml`

### Launcher omitted sibling underlay/poodle mounts
- **Friction:** Worktree parent lacked `../underlay` and `../poodle`, so `effigy tasks` failed until mounts were added.
- **Impact:** Blocks Effigy orientation and local `file:../../underlay` installs.
- **Plausible fix:** Launcher should create the same sibling symlinks Soundcheck gets.
- **Surface:** underlay-reference t3 worktree bring-up

### Bun keeps stale file: nested Poodle metadata
- **Friction:** After removing Poodle overrides, incremental `bun install` left `file:../../poodle/...` nests under `file:` underlay/acme-ui snapshots; only `bun update` on those file deps refreshed them.
- **Impact:** Locks can look sibling-controlled despite registry top-level installs.
- **Plausible fix:** Document `bun update <file-dep>` after override removal, or fix Bun file: snapshot refresh.
- **Surface:** acme-*/bun.lock adoption
