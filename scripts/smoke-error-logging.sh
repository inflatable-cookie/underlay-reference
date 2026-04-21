#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

API_BASE_URL="${API_BASE_URL:-https://api.acme.test}"
SMOKE_ENDPOINT="${SMOKE_ENDPOINT:-/v1/dev/error-smoke}"

info() { echo "[INFO] $1"; }
ok() { echo "[OK] $1"; }
fail() { echo "[ERROR] $1"; exit 1; }

command -v curl >/dev/null 2>&1 || fail "curl is required"
command -v effigy >/dev/null 2>&1 || fail "effigy is required"

cd "$PROJECT_ROOT"

info "Triggering forced error: ${API_BASE_URL}${SMOKE_ENDPOINT}"
http_code="$(curl -sS -o /tmp/acme-error-smoke-response.txt -w '%{http_code}' -X POST "${API_BASE_URL}${SMOKE_ENDPOINT}" || true)"

if [[ "$http_code" != "500" ]]; then
  cat /tmp/acme-error-smoke-response.txt || true
  fail "Expected HTTP 500 from smoke endpoint, got ${http_code}. Ensure acme-api is running in debug mode."
fi
ok "Smoke endpoint returned 500"

info "Waiting briefly for async error-log write..."
sleep 1

sql="
SELECT json_build_object(
  'error_code', error_code,
  'message', message,
  'handler_context', context->'handler_context'
)
FROM platform.error_log
WHERE error_code = 'smoke.forced_db_failure'
ORDER BY occurred_at DESC
LIMIT 1;
"

row_json="$(effigy exec --service postgres psql -U postgres -d acme -t -A -c "$sql" | tail -n 1 | tr -d '\r')"

if [[ -z "$row_json" || "$row_json" == "" ]]; then
  fail "No matching error_log row found for code smoke.forced_db_failure"
fi

echo "$row_json"

echo "$row_json" | rg -q '"error_code"\s*:\s*"smoke.forced_db_failure"' || fail "error_code missing in row"
echo "$row_json" | rg -q '"message"\s*:\s*"Forced failure for error-log smoke testing"' || fail "message missing in row"
echo "$row_json" | rg -q '"handler_context"\s*:\s*\{' || fail "handler_context missing or null"
echo "$row_json" | rg -q '"operation"\s*:\s*"smoke.error_logging_capture"' || fail "handler_context.operation missing"

ok "Smoke test passed: error_code, message, and handler_context are present"
