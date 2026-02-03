//! Account DTOs for user profiles and preferences.

use acme_db::account::UserProfileRow;
use serde::{Deserialize, Serialize};
use validator::Validate;

// ============================================================================
// User Profile DTOs
// ============================================================================

/// User profile response.
///
/// Name fields follow the Underlay pattern for culturally-inclusive identity:
/// - `fullName`: User's full name as they wish to be known
/// - `displayName`: Short name for UI display
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileDto {
    pub user_id: String,
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
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserProfileRow> for UserProfileDto {
    fn from(row: UserProfileRow) -> Self {
        Self {
            user_id: row.user_id.to_string(),
            full_name: row.full_name,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
            country_code: row.country_code,
            time_zone: row.time_zone,
            language: row.language,
            region_code: row.region_code,
            currency_preference: row.currency_preference,
            email_marketing_opt_in: row.email_marketing_opt_in,
            email_transactional_opt_in: row.email_transactional_opt_in,
            email_frequency: row.email_frequency,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

/// Request to update user profile.
///
/// All fields are optional - only provided fields will be updated.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    #[validate(length(max = 256, message = "Full name must be 256 characters or less"))]
    pub full_name: Option<String>,

    #[validate(length(max = 64, message = "Display name must be 64 characters or less"))]
    pub display_name: Option<String>,

    #[validate(length(max = 512, message = "Avatar URL must be 512 characters or less"))]
    pub avatar_url: Option<String>,

    #[validate(length(equal = 2, message = "Country code must be exactly 2 characters"))]
    pub country_code: Option<String>,

    #[validate(length(max = 64, message = "Time zone must be 64 characters or less"))]
    pub time_zone: Option<String>,

    #[validate(length(max = 10, message = "Language must be 10 characters or less"))]
    pub language: Option<String>,

    #[validate(length(max = 8, message = "Region code must be 8 characters or less"))]
    pub region_code: Option<String>,

    #[validate(length(equal = 3, message = "Currency must be exactly 3 characters"))]
    pub currency_preference: Option<String>,

    pub email_marketing_opt_in: Option<bool>,
    pub email_transactional_opt_in: Option<bool>,

    #[validate(custom(function = "validate_email_frequency"))]
    pub email_frequency: Option<String>,
}

/// Validate email frequency is one of the allowed values.
fn validate_email_frequency(value: &str) -> Result<(), validator::ValidationError> {
    match value {
        "low" | "normal" | "high" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_email_frequency");
            err.message = Some("Email frequency must be 'low', 'normal', or 'high'".into());
            Err(err)
        }
    }
}

impl UpdateProfileRequest {
    /// Convert to database update struct.
    pub fn into_db_update(self) -> acme_db::account::UserProfileUpdate {
        acme_db::account::UserProfileUpdate {
            full_name: self.full_name.map(Some),
            display_name: self.display_name.map(Some),
            avatar_url: self.avatar_url.map(Some),
            country_code: self.country_code.map(Some),
            time_zone: self.time_zone.map(Some),
            language: self.language.map(Some),
            region_code: self.region_code.map(Some),
            currency_preference: self.currency_preference.map(Some),
            email_marketing_opt_in: self.email_marketing_opt_in,
            email_transactional_opt_in: self.email_transactional_opt_in,
            email_frequency: self.email_frequency,
            cookie_consent: None,
            data_processing_consent_version: None,
        }
    }
}
