//! Account database functions.
//!
//! User profiles, preferences, and account-related data.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DbPool;

// ============================================================================
// User Profile
// ============================================================================

/// Row type for account.user_profile table.
///
/// Name fields follow the Underlay pattern for culturally-inclusive identity:
/// - `full_name`: User's full name as they wish to be known
/// - `display_name`: Short name for UI display (defaults to first word of full_name)
#[derive(Debug, Clone, FromRow)]
pub struct UserProfileRow {
    pub user_id: Uuid,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub country_code: Option<String>,
    pub time_zone: Option<String>,
    pub language: Option<String>,
    pub region_code: Option<String>,
    pub currency_preference: Option<String>,
    pub email_marketing_opt_in: bool,
    pub email_transactional_opt_in: bool,
    pub email_frequency: String,
    pub cookie_consent: Option<serde_json::Value>,
    pub data_processing_consent_version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Update payload for user profile.
///
/// Fields use `Option<Option<T>>` pattern:
/// - `None` = don't update this field
/// - `Some(None)` = set field to NULL
/// - `Some(Some(value))` = set field to value
#[derive(Debug, Default)]
pub struct UserProfileUpdate {
    pub full_name: Option<Option<String>>,
    pub display_name: Option<Option<String>>,
    pub avatar_url: Option<Option<String>>,
    pub country_code: Option<Option<String>>,
    pub time_zone: Option<Option<String>>,
    pub language: Option<Option<String>>,
    pub region_code: Option<Option<String>>,
    pub currency_preference: Option<Option<String>>,
    pub email_marketing_opt_in: Option<bool>,
    pub email_transactional_opt_in: Option<bool>,
    pub email_frequency: Option<String>,
    pub cookie_consent: Option<Option<serde_json::Value>>,
    pub data_processing_consent_version: Option<Option<String>>,
}

/// Get a user's profile by user ID.
pub async fn get_user_profile(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Option<UserProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, UserProfileRow>(
        r#"
        SELECT
            user_id,
            full_name,
            display_name,
            avatar_url,
            country_code,
            time_zone,
            language,
            region_code,
            currency_preference,
            email_marketing_opt_in,
            email_transactional_opt_in,
            email_frequency,
            cookie_consent,
            data_processing_consent_version,
            created_at,
            updated_at
        FROM account.user_profile
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Create a new user profile with defaults.
pub async fn create_user_profile(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<UserProfileRow, sqlx::Error> {
    sqlx::query_as::<_, UserProfileRow>(
        r#"
        INSERT INTO account.user_profile (user_id)
        VALUES ($1)
        RETURNING
            user_id,
            full_name,
            display_name,
            avatar_url,
            country_code,
            time_zone,
            language,
            region_code,
            currency_preference,
            email_marketing_opt_in,
            email_transactional_opt_in,
            email_frequency,
            cookie_consent,
            data_processing_consent_version,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Get or create a user's profile.
///
/// Returns the existing profile if one exists, otherwise creates a new one with defaults.
pub async fn get_or_create_user_profile(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<UserProfileRow, sqlx::Error> {
    // Try to get existing profile first
    if let Some(profile) = get_user_profile(pool, user_id).await? {
        return Ok(profile);
    }

    // Create new profile - use INSERT ... ON CONFLICT to handle race conditions
    sqlx::query_as::<_, UserProfileRow>(
        r#"
        INSERT INTO account.user_profile (user_id)
        VALUES ($1)
        ON CONFLICT (user_id) DO UPDATE SET user_id = EXCLUDED.user_id
        RETURNING
            user_id,
            full_name,
            display_name,
            avatar_url,
            country_code,
            time_zone,
            language,
            region_code,
            currency_preference,
            email_marketing_opt_in,
            email_transactional_opt_in,
            email_frequency,
            cookie_consent,
            data_processing_consent_version,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Update a user's profile.
///
/// Only updates fields that are `Some` in the update struct.
/// Returns the updated profile.
pub async fn update_user_profile(
    pool: &DbPool,
    user_id: Uuid,
    updates: UserProfileUpdate,
) -> Result<UserProfileRow, sqlx::Error> {
    sqlx::query_as::<_, UserProfileRow>(
        r#"
        UPDATE account.user_profile
        SET
            full_name = CASE WHEN $2 THEN $3 ELSE full_name END,
            display_name = CASE WHEN $4 THEN $5 ELSE display_name END,
            avatar_url = CASE WHEN $6 THEN $7 ELSE avatar_url END,
            country_code = CASE WHEN $8 THEN $9 ELSE country_code END,
            time_zone = CASE WHEN $10 THEN $11 ELSE time_zone END,
            language = CASE WHEN $12 THEN $13 ELSE language END,
            region_code = CASE WHEN $14 THEN $15 ELSE region_code END,
            currency_preference = CASE WHEN $16 THEN $17 ELSE currency_preference END,
            email_marketing_opt_in = CASE WHEN $18 THEN $19 ELSE email_marketing_opt_in END,
            email_transactional_opt_in = CASE WHEN $20 THEN $21 ELSE email_transactional_opt_in END,
            email_frequency = CASE WHEN $22 THEN $23 ELSE email_frequency END,
            cookie_consent = CASE WHEN $24 THEN $25 ELSE cookie_consent END,
            data_processing_consent_version = CASE WHEN $26 THEN $27 ELSE data_processing_consent_version END,
            updated_at = NOW()
        WHERE user_id = $1
        RETURNING
            user_id,
            full_name,
            display_name,
            avatar_url,
            country_code,
            time_zone,
            language,
            region_code,
            currency_preference,
            email_marketing_opt_in,
            email_transactional_opt_in,
            email_frequency,
            cookie_consent,
            data_processing_consent_version,
            created_at,
            updated_at
        "#,
    )
    .bind(user_id)
    // full_name
    .bind(updates.full_name.is_some())
    .bind(updates.full_name.flatten())
    // display_name
    .bind(updates.display_name.is_some())
    .bind(updates.display_name.flatten())
    // avatar_url
    .bind(updates.avatar_url.is_some())
    .bind(updates.avatar_url.flatten())
    // country_code
    .bind(updates.country_code.is_some())
    .bind(updates.country_code.flatten())
    // time_zone
    .bind(updates.time_zone.is_some())
    .bind(updates.time_zone.flatten())
    // language
    .bind(updates.language.is_some())
    .bind(updates.language.flatten())
    // region_code
    .bind(updates.region_code.is_some())
    .bind(updates.region_code.flatten())
    // currency_preference
    .bind(updates.currency_preference.is_some())
    .bind(updates.currency_preference.flatten())
    // email_marketing_opt_in
    .bind(updates.email_marketing_opt_in.is_some())
    .bind(updates.email_marketing_opt_in.unwrap_or(false))
    // email_transactional_opt_in
    .bind(updates.email_transactional_opt_in.is_some())
    .bind(updates.email_transactional_opt_in.unwrap_or(true))
    // email_frequency
    .bind(updates.email_frequency.is_some())
    .bind(updates.email_frequency.unwrap_or_else(|| "normal".to_string()))
    // cookie_consent
    .bind(updates.cookie_consent.is_some())
    .bind(updates.cookie_consent.flatten())
    // data_processing_consent_version
    .bind(updates.data_processing_consent_version.is_some())
    .bind(updates.data_processing_consent_version.flatten())
    .fetch_one(pool)
    .await
}

/// Upsert a user's profile (create if not exists, update if exists).
///
/// Useful for profile updates when you don't know if the profile exists yet.
pub async fn upsert_user_profile(
    pool: &DbPool,
    user_id: Uuid,
    updates: UserProfileUpdate,
) -> Result<UserProfileRow, sqlx::Error> {
    // First ensure the profile exists
    get_or_create_user_profile(pool, user_id).await?;

    // Then update it
    update_user_profile(pool, user_id, updates).await
}
