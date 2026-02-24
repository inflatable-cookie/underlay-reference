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

echo "Running auth security alerting rollout checks..."

check "security alert migration exists" "test -f acme-api/migrations/202602241700__add_auth_security_alert_events.sql"
check "auth workspace has underlay-security-alerts dependency" "rg -n \"underlay-security-alerts\" acme-api/Cargo.toml acme-api/crates/auth/Cargo.toml"

check "auth config includes security alert thresholds" "rg -n \"security_alert_window|security_alert_cooldown|security_alert_failed_attempts_threshold|security_alert_distinct_users_threshold|security_alert_lockouts_threshold\" acme-api/crates/auth/src/config.rs"
check "infra config includes security alert defaults" "rg -n \"security_alert_window_secs|security_alert_cooldown_secs|security_alert_failed_attempts_threshold|security_alert_distinct_users_threshold|security_alert_lockouts_threshold\" acme-api/crates/infra/src/config.rs acme-api/config/default.toml"

check "lockout flow records lockout-denied login attempts" "rg -n \"record_locked_login_attempt\" acme-api/crates/auth/src/local/lockout.rs acme-api/crates/auth/src/local/login.rs"
check "lockout flow evaluates alerts via shared crate" "rg -n \"evaluate_alerts|load_ip_signal_counts|has_recent_alert|insert_alert_event\" acme-api/crates/auth/src/local/lockout.rs"
check "alert emission writes operator warn log" "rg -n \"security alert emitted for suspicious login activity\" acme-api/crates/auth/src/local/lockout.rs"
check "alert emission appends audit log entry" "rg -n \"auth.security_alert_emitted|resource_type: \\\"security_alert_event\\\"\" acme-api/crates/auth/src/local/lockout.rs"

if [ "$failures" -gt 0 ]; then
  echo "Auth security alerting checks failed: $failures"
  exit 1
fi

echo "Auth security alerting checks passed."
