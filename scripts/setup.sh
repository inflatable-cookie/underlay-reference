#!/usr/bin/env bash
#
# Acme Reference Implementation - Development Setup Script
#
# This script sets up everything needed for local development:
#   1. Starts Docker services (PostgreSQL, MinIO, MailHog)
#   2. Creates environment files from templates
#   3. Runs database migrations
#   4. Generates JWT keys
#   5. Creates a seed admin user
#   6. Installs frontend dependencies
#
# Usage:
#   ./scripts/setup.sh           # Full setup
#   ./scripts/setup.sh --reset   # Reset database and start fresh
#
# Requirements:
#   - Docker and Docker Compose
#   - Rust toolchain (cargo)
#   - Bun (for TypeScript projects)
#   - Underlay symlink in place

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# =============================================================================
# Check Prerequisites
# =============================================================================

check_prerequisites() {
    info "Checking prerequisites..."

    # Check Docker
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed. Please install Docker Desktop."
    fi

    # Check Docker Compose
    if ! docker compose version &> /dev/null; then
        error "Docker Compose is not available. Please update Docker Desktop."
    fi

    # Check Rust
    if ! command -v cargo &> /dev/null; then
        error "Rust is not installed. Please install via rustup: https://rustup.rs"
    fi

    # Check Bun
    if ! command -v bun &> /dev/null; then
        error "Bun is not installed. Please install: https://bun.sh"
    fi

    # Check Underlay symlink
    if [ ! -L "$PROJECT_ROOT/underlay" ]; then
        error "Underlay symlink not found. Create it with: ln -s /path/to/underlay ./underlay"
    fi

    success "All prerequisites met"
}

# =============================================================================
# Docker Services
# =============================================================================

start_docker_services() {
    info "Starting Docker services..."
    cd "$PROJECT_ROOT"

    if [ "$1" == "--reset" ]; then
        warn "Resetting Docker volumes..."
        docker compose down -v 2>/dev/null || true
    fi

    docker compose up -d postgres minio minio-init mailhog

    # Wait for PostgreSQL to be ready
    info "Waiting for PostgreSQL..."
    for i in {1..30}; do
        if docker compose exec -T postgres pg_isready -U root -d acme &>/dev/null; then
            success "PostgreSQL is ready"
            return
        fi
        sleep 1
    done
    error "PostgreSQL failed to start"
}

# =============================================================================
# Environment Files
# =============================================================================

setup_env_files() {
    info "Setting up environment files..."

    # API
    if [ ! -f "$PROJECT_ROOT/acme-api/.env" ]; then
        cp "$PROJECT_ROOT/acme-api/.env.example" "$PROJECT_ROOT/acme-api/.env"
        success "Created acme-api/.env"
    else
        warn "acme-api/.env already exists, skipping"
    fi

    # Admin
    if [ ! -f "$PROJECT_ROOT/acme-admin/.env" ]; then
        cp "$PROJECT_ROOT/acme-admin/.env.example" "$PROJECT_ROOT/acme-admin/.env"
        success "Created acme-admin/.env"
    else
        warn "acme-admin/.env already exists, skipping"
    fi

    # Front
    if [ ! -f "$PROJECT_ROOT/acme-front/.env" ]; then
        cp "$PROJECT_ROOT/acme-front/.env.example" "$PROJECT_ROOT/acme-front/.env"
        success "Created acme-front/.env"
    else
        warn "acme-front/.env already exists, skipping"
    fi
}

# =============================================================================
# Database Setup
# =============================================================================

run_migrations() {
    info "Running database migrations..."
    cd "$PROJECT_ROOT/acme-api"

    # Check if migrate binary exists, build if not
    if ! cargo run -p acme-db --bin migrate_dev_db 2>&1; then
        error "Migration failed. Check the error above."
    fi

    success "Database migrations complete"
}

# =============================================================================
# Auth Keys
# =============================================================================

generate_jwt_keys() {
    info "Checking JWT keys..."
    cd "$PROJECT_ROOT/acme-api"

    # Check if keys already exist in .env
    if grep -q "AUTH_JWT_PRIVATE_KEY=" .env && grep -q "AUTH_JWT_PUBLIC_KEY=" .env; then
        # Check if they have values (not just empty)
        if grep -q "AUTH_JWT_PRIVATE_KEY=.\+" .env; then
            warn "JWT keys already configured, skipping"
            return
        fi
    fi

    info "Generating JWT keys..."
    if cargo run -p acme-auth --bin generate-jwt-env >> .env 2>&1; then
        success "JWT keys generated and added to .env"
    else
        error "Failed to generate JWT keys"
    fi
}

# =============================================================================
# Seed Data
# =============================================================================

create_seed_data() {
    info "Creating seed data..."
    cd "$PROJECT_ROOT/acme-api"

    # Check if seed user already exists by trying to run the API briefly
    # For now, we'll just inform the user to create an account manually
    warn "Seed data creation requires manual steps:"
    echo "  1. Start the API: cd acme-api && cargo run"
    echo "  2. Start the admin: cd acme-admin && bun dev"
    echo "  3. Open http://localhost:40012 and register an admin account"
    echo "  4. Manually update the user's role to 'admin' in the database:"
    echo "     UPDATE auth.users SET role = 'admin' WHERE email = 'your@email.com';"
}

# =============================================================================
# Frontend Dependencies
# =============================================================================

install_frontend_deps() {
    info "Installing frontend dependencies..."

    # Client
    info "Installing acme-client dependencies..."
    cd "$PROJECT_ROOT/acme-client"
    bun install

    # Admin
    info "Installing acme-admin dependencies..."
    cd "$PROJECT_ROOT/acme-admin"
    bun install

    # Front
    info "Installing acme-front dependencies..."
    cd "$PROJECT_ROOT/acme-front"
    bun install

    success "Frontend dependencies installed"
}

# =============================================================================
# Main
# =============================================================================

main() {
    echo ""
    echo "=========================================="
    echo " Acme Reference Implementation Setup"
    echo "=========================================="
    echo ""

    cd "$PROJECT_ROOT"

    check_prerequisites
    start_docker_services "$1"
    setup_env_files
    run_migrations
    generate_jwt_keys
    install_frontend_deps
    create_seed_data

    echo ""
    echo "=========================================="
    success "Setup complete!"
    echo "=========================================="
    echo ""
    echo "To start development:"
    echo ""
    echo "  # Terminal 1: API server"
    echo "  cd acme-api && cargo run"
    echo ""
    echo "  # Terminal 2: Admin frontend"
    echo "  cd acme-admin && bun dev"
    echo ""
    echo "  # Terminal 3: Public frontend"
    echo "  cd acme-front && bun dev"
    echo ""
    echo "Services:"
    echo "  - API:      http://localhost:40011"
    echo "  - Admin:    http://localhost:40012"
    echo "  - Front:    http://localhost:40013"
    echo "  - MailHog:  http://localhost:8025"
    echo "  - MinIO:    http://localhost:9001"
    echo ""
}

main "$@"
