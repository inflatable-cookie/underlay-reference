-- Acme baseline: account schema.
--
-- Contains user profile table with all columns:
-- - Identity (full_name, display_name, avatar_url)
-- - Locale (country_code, region_code, time_zone, language, currency_preference)
-- - Preferences (email marketing/transactional, frequency)
-- - Consent tracking (cookie_consent, data_processing_consent_version)

CREATE TABLE IF NOT EXISTS account.user_profile (
    user_id UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,

    -- Identity
    full_name TEXT,
    display_name TEXT,
    avatar_url TEXT,

    -- Locale
    country_code TEXT,
    region_code TEXT,
    time_zone TEXT,
    language TEXT,
    currency_preference TEXT,

    -- Email preferences
    email_marketing_opt_in BOOLEAN NOT NULL DEFAULT FALSE,
    email_transactional_opt_in BOOLEAN NOT NULL DEFAULT TRUE,
    email_frequency TEXT NOT NULL DEFAULT 'normal' CHECK (email_frequency IN ('low', 'normal', 'high')),

    -- Consent tracking
    cookie_consent JSONB,
    data_processing_consent_version TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_account_user_profile_time_zone
    ON account.user_profile (time_zone)
    WHERE time_zone IS NOT NULL;
