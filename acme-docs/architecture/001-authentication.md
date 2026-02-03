# Authentication Architecture

This document describes the authentication patterns implemented in the Acme reference.

## Overview

Acme uses JWT-based authentication with refresh tokens for session management. The implementation supports:

- Email/password authentication
- Two-factor authentication (TOTP, email codes)
- Passkeys (WebAuthn)
- Session management with device fingerprints
- Rate limiting on auth endpoints

## Token Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        Token Flow                                 │
│                                                                   │
│  Login                                                           │
│    │                                                             │
│    ▼                                                             │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Access Token (JWT)                                      │    │
│  │  - Expires: 15 minutes                                   │    │
│  │  - Contains: user_id, email, role, session_id            │    │
│  │  - Used for: API authorization                           │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Refresh Token (Opaque)                                  │    │
│  │  - Expires: 30 days                                      │    │
│  │  - Stored in: Database (hashed)                          │    │
│  │  - Used for: Obtaining new access tokens                 │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

## Authentication Flows

### Password Login

```
Client                          API                          Database
  │                              │                              │
  │  POST /auth/login            │                              │
  │  {email, password}           │                              │
  │─────────────────────────────>│                              │
  │                              │  Find user by email          │
  │                              │─────────────────────────────>│
  │                              │  Check password hash         │
  │                              │<─────────────────────────────│
  │                              │                              │
  │                              │  Check 2FA required?         │
  │                              │─────────────────────────────>│
  │                              │<─────────────────────────────│
  │                              │                              │
  │  If 2FA enabled:             │                              │
  │  {requiresTwoFactor: true}   │                              │
  │<─────────────────────────────│                              │
  │                              │                              │
  │  If no 2FA:                  │                              │
  │  Create session              │                              │
  │                              │─────────────────────────────>│
  │  {accessToken, refreshToken} │                              │
  │<─────────────────────────────│                              │
```

### Two-Factor Authentication

```
Client                          API                          Database
  │                              │                              │
  │  POST /auth/login            │                              │
  │  (password verified)         │                              │
  │─────────────────────────────>│                              │
  │                              │                              │
  │  {requiresTwoFactor: true,   │                              │
  │   methods: ["totp","email"]} │                              │
  │<─────────────────────────────│                              │
  │                              │                              │
  │  POST /auth/login/finish     │                              │
  │  {email, totpCode}           │                              │
  │─────────────────────────────>│                              │
  │                              │  Verify TOTP code            │
  │                              │─────────────────────────────>│
  │                              │<─────────────────────────────│
  │                              │  Create session              │
  │                              │─────────────────────────────>│
  │  {accessToken, refreshToken} │                              │
  │<─────────────────────────────│                              │
```

### Token Refresh

```
Client                          API                          Database
  │                              │                              │
  │  POST /auth/refresh          │                              │
  │  {refreshToken}              │                              │
  │─────────────────────────────>│                              │
  │                              │  Find session by token       │
  │                              │─────────────────────────────>│
  │                              │  Validate not expired        │
  │                              │  Validate fingerprint        │
  │                              │<─────────────────────────────│
  │                              │                              │
  │                              │  Rotate refresh token        │
  │                              │─────────────────────────────>│
  │  {accessToken, refreshToken} │                              │
  │<─────────────────────────────│                              │
```

## Session Management

### Session Storage

Sessions are stored in the `auth.sessions` table:

```sql
CREATE TABLE auth.sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES auth.users(id),
    refresh_token_hash TEXT NOT NULL,
    fingerprint_hash TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    device_name TEXT,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    refresh_token_expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    revocation_reason TEXT
);
```

### Fingerprint Validation

Device fingerprints help detect token theft:

```rust
// On login, compute fingerprint from request
let fingerprint = compute_fingerprint(
    request.user_agent(),
    request.ip_address(),
);

// On refresh, validate fingerprint matches
if session.fingerprint_hash != hash(current_fingerprint) {
    // Potential token theft - revoke session
    revoke_session(session.id, "fingerprint_mismatch");
    return Err(AuthError::InvalidSession);
}
```

## API Endpoints

### Public Endpoints (No Auth Required)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/auth/register` | Create new account |
| POST | `/v1/auth/login` | Start login (password) |
| POST | `/v1/auth/login/start` | Start login (check 2FA) |
| POST | `/v1/auth/login/finish` | Complete login with 2FA |
| POST | `/v1/auth/refresh` | Refresh access token |
| POST | `/v1/auth/password/reset/request` | Request password reset |
| POST | `/v1/auth/password/reset/verify` | Verify reset token |
| POST | `/v1/auth/password/reset/complete` | Complete reset |
| GET | `/v1/auth/password/requirements` | Get password rules |

### Protected Endpoints (Auth Required)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/auth/me` | Get current user |
| POST | `/v1/auth/logout` | Logout (revoke session) |
| POST | `/v1/auth/password/change` | Change password |
| GET | `/v1/auth/sessions` | List active sessions |
| POST | `/v1/auth/sessions/:id/revoke` | Revoke a session |
| GET | `/v1/auth/2fa-status` | Check 2FA methods |

### TOTP Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/auth/totp/status` | Check TOTP enabled |
| POST | `/v1/auth/totp/setup` | Start TOTP setup |
| POST | `/v1/auth/totp/enable` | Enable TOTP |
| POST | `/v1/auth/totp/disable` | Disable TOTP |

### Passkey Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/auth/passkeys` | List passkeys |
| POST | `/v1/auth/passkeys/register/start` | Start registration |
| POST | `/v1/auth/passkeys/register/finish` | Complete registration |
| POST | `/v1/auth/passkeys/login/start` | Start passkey login |
| POST | `/v1/auth/passkeys/login/finish` | Complete passkey login |
| PATCH | `/v1/auth/passkeys/:id` | Rename passkey |
| DELETE | `/v1/auth/passkeys/:id` | Delete passkey |

## Security Considerations

### Password Requirements

Passwords are validated against configurable rules:

```rust
pub struct PasswordRequirements {
    pub min_length: usize,        // Default: 8
    pub max_length: usize,        // Default: 128
    pub require_uppercase: bool,  // Default: true
    pub require_lowercase: bool,  // Default: true
    pub require_number: bool,     // Default: true
    pub require_special: bool,    // Default: false
}
```

### Rate Limiting

Auth endpoints are rate-limited to prevent brute force:

| Endpoint | Limit |
|----------|-------|
| Login | 5 attempts per minute per IP |
| Password reset | 3 requests per hour per email |
| TOTP verify | 5 attempts per minute |

### Token Security

- Access tokens are signed with RS256 (RSA-SHA256)
- Refresh tokens are stored hashed (SHA-256)
- Cookies use `HttpOnly`, `Secure`, `SameSite=Strict`

## Frontend Integration

### Token Storage

The frontend stores tokens based on environment:

```typescript
// Browser: HttpOnly cookies (recommended)
// Set automatically by API responses

// SSR: In-memory token store
const tokenStore = createTokenStore();
```

### Auth Manager

The `AuthManager` handles automatic token refresh:

```typescript
import { createAuthManager, configureAcmeClient } from "acme-client";

// Configure once at app startup
configureAcmeClient({ apiUrl: "http://localhost:40011" });

const auth = createAuthManager({
  onLogout: () => goto("/login"),
  refreshBuffer: 60, // Refresh 60s before expiry
});

// Use in API calls
const token = await auth.getToken();
```

### Protected Routes

SvelteKit hooks handle route protection:

```typescript
// hooks.server.ts
export const handle: Handle = async ({ event, resolve }) => {
  const token = readAccessToken(event.cookies);

  if (event.url.pathname.startsWith("/app/")) {
    if (!token) {
      throw redirect(303, "/login");
    }
  }

  return resolve(event);
};
```

## Database Schema

### Users Table

```sql
CREATE TABLE auth.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    role TEXT NOT NULL DEFAULT 'user',
    status TEXT NOT NULL DEFAULT 'active',
    display_name TEXT,
    failed_login_count INTEGER NOT NULL DEFAULT 0,
    lockout_until TIMESTAMPTZ,
    totp_secret_encrypted BYTEA,
    totp_enabled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Passkeys Table

```sql
CREATE TABLE auth.passkeys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id),
    credential_id BYTEA NOT NULL UNIQUE,
    public_key BYTEA NOT NULL,
    counter INTEGER NOT NULL DEFAULT 0,
    name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);
```
