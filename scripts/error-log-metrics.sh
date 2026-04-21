#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
WINDOW_HOURS="${WINDOW_HOURS:-24}"

info() { echo "[INFO] $1"; }
fail() { echo "[ERROR] $1"; exit 1; }

command -v effigy >/dev/null 2>&1 || fail "effigy is required"

cd "$PROJECT_ROOT"

info "Computing error-log metrics for the last ${WINDOW_HOURS} hour(s)"

sql="
WITH windowed AS (
  SELECT context
  FROM platform.error_log
  WHERE occurred_at > now() - make_interval(hours => ${WINDOW_HOURS})
),
stats AS (
  SELECT
    COUNT(*)::int AS total_errors,
    COUNT(*) FILTER (
      WHERE context->'handler_context' IS NULL
        OR context->'handler_context' = 'null'::jsonb
        OR context->'handler_context' = '{}'::jsonb
    )::int AS null_handler_context
  FROM windowed
)
SELECT json_build_object(
  'window_hours', ${WINDOW_HOURS},
  'total_errors', total_errors,
  'null_handler_context', null_handler_context,
  'null_rate_percent',
    CASE
      WHEN total_errors = 0 THEN 0
      ELSE ROUND((null_handler_context::numeric / total_errors::numeric) * 100, 2)
    END
)
FROM stats;
"

result="$(effigy exec --service postgres psql -U postgres -d acme -t -A -c "$sql" | tail -n 1 | tr -d '\r')"

if [[ -z "$result" ]]; then
  fail "Failed to query error log metrics"
fi

echo "$result"
