# Agents Guide: acme-api

Rust backend reference implementation demonstrating Underlay patterns.

## Crate Organization

```
crates/
├── core/      # Domain primitives, error types, ID helpers
├── infra/     # Config loading, logging, email setup
├── db/        # Database pool, migrations, queries
├── auth/      # Authentication service, JWT, 2FA
├── domain/    # Business logic (your entities go here)
├── jobs/      # Background job handlers
└── api/       # HTTP handlers, routes, main.rs
```

## Key Files

### Entry Points
- `crates/api/src/main.rs` - API server entry point
- `crates/jobs/src/main.rs` - Job worker entry point

### Configuration
- `.env.example` - Environment variables template
- `crates/infra/src/config.rs` - Config struct and loading

### Database
- `migrations/` - SQL migration files
- `crates/db/src/lib.rs` - Pool creation, migration runner
- `crates/db/src/auth.rs` - Auth-related queries
- `crates/db/src/account.rs` - Account/profile queries

### Authentication
- `crates/auth/src/local.rs` - Main auth service (login, register, 2FA)
- `crates/auth/src/config.rs` - Rate limits, timeouts
- `crates/auth/src/principal.rs` - User/role types

### API Routes
- `crates/api/src/routes/shared/health.rs` - Health checks
- `crates/api/src/routes/shared/auth.rs` - Auth endpoints
- `crates/api/src/routes/shared/account.rs` - Account endpoints

## Adding New Functionality

### New Entity (e.g., "widgets")

1. **Database table** - Add migration:
   ```sql
   -- migrations/YYYYMMDDHHMI__create_widgets.sql
   CREATE TABLE acme.widgets (
       id UUID PRIMARY KEY,
       name TEXT NOT NULL,
       created_at TIMESTAMPTZ DEFAULT NOW()
   );
   ```

2. **Query functions** - Add `crates/db/src/widgets.rs`:
   ```rust
   pub async fn create_widget(pool: &DbPool, name: &str) -> Result<Widget, sqlx::Error> { ... }
   ```

3. **Domain logic** - Add `crates/domain/src/widgets.rs` if needed

4. **API route** - Add `crates/api/src/routes/widgets.rs`:
   ```rust
   pub fn router() -> Router<AppState> {
       Router::new()
           .route("/widgets", post(create_widget))
   }
   ```

5. **Wire up** - Register in `crates/api/src/routes/mod.rs`

## Patterns

### Error Handling
Use `AppError` from `acme-core` for consistent API error responses.

### Rate Limiting
Configure in `crates/auth/src/config.rs`. Uses in-memory rate limiter.

### Database Queries
Use `sqlx::query_as!` for compile-time checked queries when possible.

### Authentication Middleware
Use `Principal` extractor in route handlers:
```rust
async fn my_handler(principal: UserPrincipal) -> impl IntoResponse { ... }
```

## Commands

```bash
# Build all
cargo build

# Run API server
cargo run -p acme-api

# Run migrations
cargo run -p acme-db --bin migrate_dev_db

# Reset database
cargo run -p acme-db --bin reset_dev_db

# Generate JWT keys
cargo run -p acme-auth --bin generate-jwt-env
```
