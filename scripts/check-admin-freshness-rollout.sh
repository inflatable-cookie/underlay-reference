#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

failures=0

check() {
  local description="$1"
  local cmd="$2"
  if eval "$cmd" >/dev/null 2>&1; then
    printf "ok  - %s\n" "$description"
  else
    printf "fail - %s\n" "$description"
    failures=$((failures + 1))
  fi
}

echo "Running admin freshness rollout checks..."

check "admin freshness helper exists" "test -f acme-api/crates/api/src/routes/admin/freshness.rs"
check "projects route has conditional GET" "rg -n \"maybe_not_modified\" acme-api/crates/api/src/routes/admin/projects.rs"
check "projects route has If-Match guard" "rg -n \"if_match_mismatch\" acme-api/crates/api/src/routes/admin/projects.rs"
check "categories route has conditional GET" "rg -n \"maybe_not_modified\" acme-api/crates/api/src/routes/admin/categories.rs"
check "categories route has If-Match guard" "rg -n \"if_match_mismatch\" acme-api/crates/api/src/routes/admin/categories.rs"
check "tasks route has conditional GET" "rg -n \"maybe_not_modified\" acme-api/crates/api/src/routes/admin/tasks.rs"
check "tasks route has If-Match guard" "rg -n \"if_match_mismatch\" acme-api/crates/api/src/routes/admin/tasks.rs"
check "users route has conditional GET" "rg -n \"maybe_not_modified\" acme-api/crates/api/src/routes/admin/users.rs"
check "users route has If-Match guard" "rg -n \"if_match_mismatch\" acme-api/crates/api/src/routes/admin/users.rs"
check "media route has conditional GET" "rg -n \"maybe_not_modified\" acme-api/crates/api/src/routes/admin/media/crud.rs"
check "media route has If-Match guard" "rg -n \"if_match_mismatch\" acme-api/crates/api/src/routes/admin/media/crud.rs"

check "project command exposes with-etag helpers" "rg -n \"getProjectWithEtag|updateProjectWithEtag\" acme-client/src/commands/admin/project-commands.ts"
check "category command exposes with-etag helpers" "rg -n \"getCategoryWithEtag|updateCategoryWithEtag\" acme-client/src/commands/admin/category-commands.ts"
check "task command exposes with-etag helpers" "rg -n \"getTaskWithEtag|updateTaskWithEtag\" acme-client/src/commands/admin/task-commands.ts"
check "user command exposes with-etag helpers" "rg -n \"getUserWithEtag|updateUserWithEtag\" acme-client/src/commands/admin/user-commands.ts"
check "media command exposes with-etag helpers" "rg -n \"getMediaWithEtag|updateMediaWithEtag\" acme-client/src/commands/media-commands.ts"

check "project edit uses with-etag update path" "rg -n \"updateProjectWithEtag|resource.precondition_failed|changed in another session\" 'acme-admin/src/routes/(app)/projects/[projectId]/edit/+page.svelte'"
check "category edit uses with-etag update path" "rg -n \"updateCategoryWithEtag|resource.precondition_failed|changed in another session\" 'acme-admin/src/routes/(app)/categories/[categoryId]/edit/+page.svelte'"
check "task edit uses with-etag update path" "rg -n \"updateTaskWithEtag|resource.precondition_failed|changed in another session\" 'acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/edit/+page.svelte'"
check "user edit uses with-etag update path" "rg -n \"updateUserWithEtag|resource.precondition_failed|changed in another session\" 'acme-admin/src/routes/(app)/users/[userId]/edit/+page.svelte'"
check "media edit uses with-etag update path" "rg -n \"updateMediaWithEtag|resource.precondition_failed|changed in another session\" 'acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte'"

if [ "$failures" -gt 0 ]; then
  echo "Freshness rollout checks failed: $failures"
  exit 1
fi

echo "Freshness rollout checks passed."
