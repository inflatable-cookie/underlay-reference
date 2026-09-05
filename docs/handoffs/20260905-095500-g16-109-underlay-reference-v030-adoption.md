---
title: g16.109 Underlay Reference Poodle v0.3.0 adoption worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
status: ready-for-independent-review
owner: Tom / Poodle coordinator
created: 2026-09-05
updated: 2026-09-05
handoff_path: /Users/tom/.paseo/worktrees/119ajruu/g16-109-underlay-reference-v030-adoption/docs/handoffs/20260905-095500-g16-109-underlay-reference-v030-adoption.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, g16, poodle, underlay-reference]
---

## Assignment

Execute the Underlay Reference lane from Poodle card `g16.109`,
`/Users/tom/Dev/projects/poodle/docs/roadmaps/g16/109-v030-consumer-adoption-wave.md`.
Adopt public-registry Poodle `0.3.0`, move every Underlay git declaration to
released tag `v0.9.8`, absorb the one markdown import migration without shims,
validate with this consumer's own headless selectors, and stop for independent
exact-head review.

## Exact workspace and heads

- repository: `git@github.com:inflatable-cookie/underlay-reference.git`;
- workspace: `/Users/tom/.paseo/worktrees/119ajruu/g16-109-underlay-reference-v030-adoption`;
- worker branch: `g16-109-underlay-reference-v030-adoption` (launcher-provided;
  reused as-is);
- required sibling worktree links: `poodle` → `/Users/tom/Dev/projects/poodle`,
  `underlay` → `/Users/tom/Dev/projects/underlay` (already present beside this
  worktree);
- base: `origin/main` at `2b7cfb9b4a38f097e760f0823893ecc437a4e69a`
  (`deps: adopt Poodle 0.3.0 consumer release (#16)`);
- implementation head before this handoff commit:
  `f92b16a25b34c1e9e38533913c894307d82c83b1`
  (`deps: pin remaining Underlay declarations to v0.9.8`);
- Underlay foundation: released annotated tag `v0.9.8`, verified on origin;
  tag object `09186eecda9a48a05239cebdc4e1c26ac666a934` peels to release commit
  `97a26d9fa0a58daf198926ddcd259193daa9d5c3`;
- Poodle release authority: `v0.3.0`, published 2026-09-05;
- Longhorn foundation for the wider wave: PR #21 merged at `168ecc72` (not a
  dependency of this repository);
- sibling checkouts are read-only inputs; no coordinator checkout or sibling
  repository was edited.

## Changes

PR #16 already landed public Poodle `0.3.0`, JavaScript Underlay `#v0.9.8`,
and the sole markdown-family import move. This head finishes the remaining
app manifests:

- Moved every `apps/acme-api` Underlay Cargo git tag from `v0.9.7` to
  `v0.9.8` and regenerated `Cargo.lock` against peel `97a26d9`. The lock diff
  is only those Underlay crate source/version lines.
- Confirmed JavaScript declarations and `bun.lock` already resolve public
  `@inflatable-cookie/poodle-core` / `@inflatable-cookie/poodle-svelte`
  exact `0.3.0` and Underlay `#v0.9.8` at `97a26d9`. `bun install` via
  `effigy workspace:js:prepare` did not change `bun.lock`.
- Confirmed the sole markdown-family import,
  `MarkdownEditor` in `packages/acme-ui/src/nightfire/notes/TaskNotesEditor.svelte`,
  already uses `@inflatable-cookie/poodle-svelte/markdown`. No HistoryCenter
  caller exists in this consumer. No shim, alias, compatibility re-export, or
  unrelated source change.
- Updated the live README pin examples from Underlay `v0.9.7` / Poodle `0.2.2`
  to Underlay `v0.9.8` / Poodle `0.3.0`.
- Recorded a papercut that PR #16 left Cargo on `v0.9.7`. No existing PAPERCUT
  entry was closed: the open entries are tooling debt, not the ten consumer
  defects named in the Poodle `0.3.0` release note.

## Dependency and lock evidence

- Bun: `1.3.14` (repository `packageManager` version).
- Cargo: `1.97.1`.
- Regeneration: `effigy workspace:js:prepare` (`bun install`) completed with
  no `bun.lock` diff; `cargo update -p underlay-core` saved `Cargo.lock`.
- Underlay release proof: `git ls-remote --tags` returned annotated tag
  `v0.9.8` (`09186eec…`) peeling to `97a26d9fa0a58daf198926ddcd259193daa9d5c3`.
- `effigy --json deps status bun`: no links, drift, conflicts, errors,
  warnings, or missing dependencies.
- `bun.lock` contains exactly one public Poodle core identity and one public
  Poodle Svelte identity, both `0.3.0`, with registry integrity hashes.
  Underlay resolves to `git+ssh://…underlay.git#97a26d9…`.
- Final lock grep found no `file:`, `link:`, sibling `poodle`/`underlay` path,
  or Poodle `0.2.x` marker.
- `Cargo.lock` Underlay sources are exclusively
  `git+ssh://git@github.com/inflatable-cookie/underlay.git?tag=v0.9.8#97a26d9…`.

## Headless validation

All commands ran from the workspace above through Effigy; no desktop app or
native proof was launched. Selectors ran serially after an earlier lane hit
Effigy task locks under parallel validators.

- `effigy acme-client/validate`: pass; TypeScript check and build completed.
- `effigy acme-ui/validate`: pass; 0 Svelte errors and 0 warnings.
- `effigy acme-admin/validate`: pass; 0 Svelte errors and 0 warnings;
  production build completed.
- `effigy acme-front/validate`: pass; 0 Svelte errors and 0 warnings;
  production build completed.
- `effigy acme-admin/test`: pass; 2 files, 32 tests.
- `effigy acme-client/test`: pass; 2 files, 3 tests.
- `effigy acme-api/validate`: pass; workspace build, `cargo test --workspace`
  (148 passed, 2 ignored, 0 failed), clippy, and fmt.
- `git diff --check`: pass.

Root `effigy validate` was not used; it fans into the mounted Underlay catalog
and is already recorded as an open papercut.

## Review boundary

PR https://github.com/inflatable-cookie/underlay-reference/pull/17 is open
against `underlay-reference` `main`. Push the current branch and stop for
independent exact-head review; do not merge, start another lane, edit Poodle
or Underlay, or run native/desktop proofs.
