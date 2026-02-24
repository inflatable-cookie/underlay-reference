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

echo "Running reorder conflict rollout checks..."

check "projects reorder returns conflict context keys" "rg -n \"projects\\.reorder_conflict|added_ids|removed_ids\" acme-api/crates/api/src/routes/admin/projects.rs"
check "categories reorder returns conflict context keys" "rg -n \"categories\\.reorder_conflict|added_ids|removed_ids\" acme-api/crates/api/src/routes/admin/categories.rs"
check "tasks reorder returns conflict context keys" "rg -n \"tasks\\.reorder_conflict|added_ids|removed_ids\" acme-api/crates/api/src/routes/admin/tasks.rs"

check "admin has reorder conflict adapter" "test -f acme-admin/src/lib/lists/reorder-conflicts.ts"
check "projects list uses onsubmiterror adapter" "rg -n \"onsubmiterror=\\{handleReorderError\\}|recoverReorderConflict\" 'acme-admin/src/routes/(app)/projects/+page.svelte'"
check "categories list uses onsubmiterror adapter" "rg -n \"onsubmiterror=\\{handleReorderError\\}|recoverReorderConflict\" 'acme-admin/src/routes/(app)/categories/+page.svelte'"
check "project tasks list uses onsubmiterror adapter" "rg -n \"onsubmiterror=\\{handleTaskReorderError\\}|recoverReorderConflict\" 'acme-admin/src/routes/(app)/projects/[projectId]/+page.svelte'"

check "labels remain non-reorderable" "! rg -n \"reorderLabels|/labels/reorder\" acme-api acme-client acme-admin"
check "system views remain non-reorderable" "! rg -n \"reorderJobs|reorderActivity|reorderErrors\" acme-api acme-client acme-admin"

if [ "$failures" -gt 0 ]; then
  echo "Reorder conflict rollout checks failed: $failures"
  exit 1
fi

echo "Reorder conflict rollout checks passed."
