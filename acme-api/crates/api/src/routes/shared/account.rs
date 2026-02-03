//! Account route handlers.
//!
//! Endpoints for user profile management.

use acme_core::AppError;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use underlay_core::SingleResponse;
use validator::Validate;

use crate::dto::account::{UpdateProfileRequest, UserProfileDto};
use crate::error::error_response;
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
) -> impl IntoResponse {
    let user_id = user.user_id.0.into_inner();

    match get_or_create_user_profile(state.local_auth.pool(), user_id).await {
        Ok(profile) => {
            let dto = UserProfileDto::from(profile);
            let body = SingleResponse { data: dto };
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to get/create user profile");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::new("profile.load_failed", "Failed to load profile"),
            )
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
) -> impl IntoResponse {
    let user_id = user.user_id.0.into_inner();

    // Validate request
    if let Err(errors) = request.validate() {
        return error_response(
            StatusCode::BAD_REQUEST,
            AppError::new(
                "validation.failed",
                format!("Validation failed: {}", errors),
            ),
        );
    }

    // Convert to DB update and apply
    let db_update = request.into_db_update();

    match upsert_user_profile(state.local_auth.pool(), user_id, db_update).await {
        Ok(profile) => {
            let dto = UserProfileDto::from(profile);
            let body = SingleResponse { data: dto };
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to update user profile");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::new("profile.update_failed", "Failed to update profile"),
            )
        }
    }
}
