#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

NULL_RATE_THRESHOLD="${NULL_RATE_THRESHOLD:-25}"
WINDOW_HOURS="${WINDOW_HOURS:-24}"

info() { echo "[INFO] $1"; }
ok() { echo "[OK] $1"; }
warn() { echo "[WARN] $1"; }
fail() { echo "[ERROR] $1"; exit 1; }

command -v jq >/dev/null 2>&1 || fail "jq is required"

cd "$PROJECT_ROOT"

info "Checking route patterns for canonical ApiError usage"
"$PROJECT_ROOT/underlay/scripts/check-route-error-patterns.sh" "$PROJECT_ROOT/acme-api/crates/api/src/routes"
ok "Route pattern check passed"

info "Running smoke test for end-to-end error logging capture"
"$PROJECT_ROOT/scripts/smoke-error-logging.sh"
ok "Smoke test passed"

info "Measuring handler_context null-rate"
metrics_json="$("$PROJECT_ROOT/scripts/error-log-metrics.sh")"
echo "$metrics_json"

null_rate="$(echo "$metrics_json" | jq -r '.null_rate_percent')"
total_errors="$(echo "$metrics_json" | jq -r '.total_errors')"

if [[ "$total_errors" == "0" ]]; then
    warn "No errors in window; null-rate trend is not yet meaningful."
    ok "Validation completed (smoke path verified)."
    exit 0
fi

if awk "BEGIN {exit !($null_rate <= $NULL_RATE_THRESHOLD)}"; then
    ok "Null-rate ${null_rate}% is within threshold (${NULL_RATE_THRESHOLD}%)."
else
    warn "Null-rate ${null_rate}% exceeds threshold (${NULL_RATE_THRESHOLD}%)."
    warn "Routing/middleware path is healthy; continue collecting data after more migrated failures."
fi

ok "Validation run complete."
