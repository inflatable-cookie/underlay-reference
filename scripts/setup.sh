#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

info() { echo "[INFO] $1"; }
fail() { echo "[ERROR] $1" >&2; exit 1; }

command -v effigy >/dev/null 2>&1 || fail "effigy is required"

if [[ "${1:-}" == "--reset" ]]; then
  info "Resetting the managed dev container before dependency setup"
  effigy container stack reset --repo "$PROJECT_ROOT"
fi

cd "$PROJECT_ROOT"

info "scripts/setup.sh is now a compatibility shim"
info "Running the repo-owned bootstrap dependency task through Effigy"

effigy bootstrap:deps

cat <<'EOF'

Next:
  effigy health
  effigy validate
  effigy dev

Notes:
  - use `effigy bootstrap <repo-url>` from outside the repo for first clone + setup
  - this repo expects sibling `../underlay` and `../poodle` checkouts
  - the canonical local URLs are domain-based via the Effigy gateway, not localhost ports
EOF
