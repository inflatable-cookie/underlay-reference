-- Acme baseline: auth schema.
--
-- Contains all auth tables from Underlay auth:
-- - users (with lockout support)
-- - credentials
-- - sessions
-- - auth_state
-- - totp_credential
-- - login_attempts
-- - email_totp_codes
-- - email_totp_rate_limits
-- - verification_sessions

-- =========================================
-- Users
-- =========================================

CREATE TABLE IF NOT EXISTS auth.users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,

    -- Coarse primary role (mirrors common Underlay Principal roles).
    role TEXT NOT NULL DEFAULT 'student'
        CHECK (role IN ('student', 'tester', 'tutor', 'editor', 'admin', 'support', 'superadmin')),

    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'deleted')),

    -- Display name for UI
    display_name TEXT,

    -- Account lockout support
    failed_login_count INTEGER NOT NULL DEFAULT 0,
    lockout_until TIMESTAMPTZ NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_auth_users_email ON auth.users (email);

CREATE INDEX IF NOT EXISTS idx_auth_users_lockout
    ON auth.users (lockout_until)
    WHERE lockout_until IS NOT NULL;

-- =========================================
-- Credentials
-- =========================================

CREATE TABLE IF NOT EXISTS auth.credentials (
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,

    type TEXT NOT NULL
        CHECK (type IN ('password', 'totp', 'passkey', 'oauth_google')),

    -- For password this stores a hash; for others it is an encrypted blob.
    secret_encrypted TEXT NOT NULL,

    -- JSON metadata matching Underlay CredentialMetadata shapes.
    metadata JSONB NOT NULL,

    verified BOOLEAN NOT NULL DEFAULT TRUE,

    -- Optional user-facing label (e.g. passkey device name).
    display_name TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_auth_credentials_user_id ON auth.credentials (user_id);
CREATE INDEX IF NOT EXISTS idx_auth_credentials_type ON auth.credentials (type);

-- Password/TOTP/OAuthGoogle should be unique per user.
-- Passkeys should allow multiple entries per user.
CREATE UNIQUE INDEX IF NOT EXISTS idx_auth_password_unique
    ON auth.credentials (user_id)
    WHERE type = 'password';

CREATE UNIQUE INDEX IF NOT EXISTS idx_auth_totp_unique
    ON auth.credentials (user_id)
    WHERE type = 'totp';

CREATE UNIQUE INDEX IF NOT EXISTS idx_auth_oauth_google_unique
    ON auth.credentials (user_id)
    WHERE type = 'oauth_google';

-- Passkey credential IDs should be globally unique.
CREATE UNIQUE INDEX IF NOT EXISTS idx_auth_passkey_credential_id_unique
    ON auth.credentials ((metadata->>'credentialId'))
    WHERE type = 'passkey';

-- =========================================
-- Sessions
-- =========================================

CREATE TABLE IF NOT EXISTS auth.sessions (
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,

    -- Application roles snapshot (used when building principals).
    roles JSONB NOT NULL,

    is_active BOOLEAN NOT NULL,

    access_token_fingerprint TEXT NOT NULL,
    refresh_token_fingerprint TEXT NOT NULL,

    -- Refresh rotation state.
    refresh_token_id UUID NOT NULL,
    refresh_token_version INTEGER NOT NULL,

    access_token_expires_at TIMESTAMPTZ NOT NULL,
    refresh_token_expires_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    ip_address TEXT,
    user_agent TEXT,

    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked', 'expired')),
    revocation_reason TEXT,
    revoked_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_auth_sessions_user_id ON auth.sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_auth_sessions_active ON auth.sessions (user_id, is_active);
CREATE INDEX IF NOT EXISTS idx_auth_sessions_refresh_id ON auth.sessions (refresh_token_id);

-- =========================================
-- Auth state (multi-step flows)
-- =========================================

CREATE TABLE IF NOT EXISTS auth.auth_state (
    id UUID PRIMARY KEY,
    user_id UUID NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    state_type TEXT NOT NULL,
    state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_auth_auth_state_user ON auth.auth_state (user_id);
CREATE INDEX IF NOT EXISTS idx_auth_auth_state_expires ON auth.auth_state (expires_at);
CREATE INDEX IF NOT EXISTS idx_auth_auth_state_type ON auth.auth_state (state_type);

-- =========================================
-- TOTP replay protection + backup code hashes
-- =========================================

CREATE TABLE IF NOT EXISTS auth.totp_credential (
    credential_id UUID PRIMARY KEY REFERENCES auth.credentials(id) ON DELETE CASCADE,
    last_counter BIGINT NOT NULL DEFAULT 0,
    backup_code_hashes JSONB NOT NULL
);

-- =========================================
-- Login attempts tracking
-- =========================================

CREATE TABLE IF NOT EXISTS auth.login_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    ip_address INET NULL,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL DEFAULT FALSE,
    failure_reason TEXT NULL CHECK (failure_reason IS NULL OR char_length(failure_reason) <= 128)
);

CREATE INDEX IF NOT EXISTS idx_auth_login_attempts_user_id
    ON auth.login_attempts (user_id, attempted_at DESC);

CREATE INDEX IF NOT EXISTS idx_auth_login_attempts_ip
    ON auth.login_attempts (ip_address, attempted_at DESC)
    WHERE ip_address IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_auth_login_attempts_user_failures
    ON auth.login_attempts (user_id, attempted_at DESC)
    WHERE success = FALSE;

-- =========================================
-- Email TOTP codes
-- =========================================

CREATE TABLE IF NOT EXISTS auth.email_totp_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    code_hash TEXT NOT NULL,
    purpose TEXT NOT NULL CHECK (purpose IN ('login', 'password_change', 'sensitive_action', 'password_reset')),
    expires_at TIMESTAMPTZ NOT NULL,
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 5,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_totp_codes_user_purpose
    ON auth.email_totp_codes(user_id, purpose)
    WHERE used_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_email_totp_codes_expires_at
    ON auth.email_totp_codes(expires_at)
    WHERE used_at IS NULL;

-- =========================================
-- Email TOTP rate limiting
-- =========================================

CREATE TABLE IF NOT EXISTS auth.email_totp_rate_limits (
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    hour_bucket TIMESTAMPTZ NOT NULL,
    send_count INT NOT NULL DEFAULT 1,
    attempt_count INT NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, hour_bucket)
);

-- =========================================
-- Verification sessions
-- =========================================

CREATE TABLE IF NOT EXISTS auth.verification_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    purpose TEXT NOT NULL CHECK (purpose IN ('login', 'password_change', 'sensitive_action', 'password_reset')),
    method TEXT NOT NULL CHECK (method IN ('totp', 'passkey', 'email_totp')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_verification_sessions_user_purpose
    ON auth.verification_sessions(user_id, purpose, expires_at)
    WHERE used_at IS NULL;
