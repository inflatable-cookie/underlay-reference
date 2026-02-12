//! Account route handlers.
//!
//! Endpoints for user profile management.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use underlay_core::SingleResponse;
use underlay_http::ApiError;
use validator::Validate;

use crate::dto::account::{UpdateProfileRequest, UserProfileDto};
use crate::state::{AppState, AuthenticatedUser};
use acme_db::account::{get_or_create_user_profile, upsert_user_profile};

// ============================================================================
// Profile Endpoints
// ============================================================================

/// Get the current user's profile.
///
/// Creates a profile with defaults if one doesn't exist.
pub async fn get_profile(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let user_id = user.user_id.0.into_inner();

    match get_or_create_user_profile(state.local_auth.pool(), user_id).await {
        Ok(profile) => {
            let dto = UserProfileDto::from(profile);
            let body = SingleResponse { data: dto };
            Ok((StatusCode::OK, Json(body)).into_response())
        }
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to get/create user profile");
            Err(crate::db_errors::internal_with_diagnostics(
                "profile.load_failed",
                "Failed to load profile",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "account.get_profile",
                "user_id": user_id
            })))
        }
    }
}

/// Update the current user's profile.
///
/// Only provided fields are updated. Creates a profile if one doesn't exist.
pub async fn update_profile(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Response, ApiError> {
    let user_id = user.user_id.0.into_inner();

    // Validate request
    if let Err(errors) = request.validate() {
        return Err(ApiError::bad_request(
            "validation.failed",
            format!("Validation failed: {}", errors),
        ));
    }

    // Convert to DB update and apply
    let db_update = request.into_db_update();

    match upsert_user_profile(state.local_auth.pool(), user_id, db_update).await {
        Ok(profile) => {
            let dto = UserProfileDto::from(profile);
            let body = SingleResponse { data: dto };
            Ok((StatusCode::OK, Json(body)).into_response())
        }
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to update user profile");
            Err(crate::db_errors::internal_with_diagnostics(
                "profile.update_failed",
                "Failed to update profile",
                &e,
            )
            .with_context(serde_json::json!({
                "operation": "account.update_profile",
                "user_id": user_id
            })))
        }
    }
}
