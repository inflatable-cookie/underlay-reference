//! Field validation endpoints.
//!
//! These provide async field validation for frontend forms.
//! Common use case: check slug/name uniqueness before form submission.

use anyhow::Error as AnyhowError;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use underlay_http::ApiError;

use acme_core::Uuid;
use acme_db::{categories, tasks};

use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateFieldRequest {
    /// Entity type: "category", "project", "label"
    pub entity: String,
    /// Field name: "slug", "name"
    pub field: String,
    /// Value to validate
    pub value: String,
    /// Context value (e.g., project_id for label uniqueness)
    pub context_value: Option<Uuid>,
    /// ID to exclude from uniqueness check (for edits)
    pub exclude_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResponse {
    pub valid: bool,
    pub message: Option<String>,
}

// ============================================================================
// Handler
// ============================================================================

/// Validate a field value.
///
/// Used for async form validation (e.g., checking slug uniqueness).
///
/// Request body:
/// ```json
/// {
///   "entity": "category",
///   "field": "slug",
///   "value": "my-category",
///   "excludeId": "optional-uuid-for-edits"
/// }
/// ```
///
/// Response:
/// ```json
/// {
///   "valid": true,
///   "message": "Slug is available"
/// }
/// ```
pub async fn validate_field(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<ValidateFieldRequest>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    // Route to appropriate validation based on entity and field
    let result = match (req.entity.as_str(), req.field.as_str()) {
        ("category", "slug") => validate_category_slug(pool, &req).await,
        ("label", "name") => validate_label_name(pool, &req).await,
        _ => Ok(ValidationResponse {
            valid: true,
            message: Some("Field validation not implemented".to_string()),
        }),
    };

    match result {
        Ok(response) => Ok(Json(response).into_response()),
        Err(e) => {
            tracing::error!("Validation error: {}", e);
            Err(
                ApiError::internal("validation.failed", "Field validation failed")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "validation.validate_field",
                        "entity": req.entity,
                        "field": req.field
                    })),
            )
        }
    }
}

async fn validate_category_slug(
    pool: &acme_db::DbPool,
    req: &ValidateFieldRequest,
) -> Result<ValidationResponse, AnyhowError> {
    let is_available =
        categories::is_slug_available(pool, &req.value, req.exclude_id.map(|id| id.into_inner()))
            .await?;

    Ok(if is_available {
        ValidationResponse {
            valid: true,
            message: Some("Slug is available".to_string()),
        }
    } else {
        ValidationResponse {
            valid: false,
            message: Some("This slug is already in use".to_string()),
        }
    })
}

async fn validate_label_name(
    pool: &acme_db::DbPool,
    req: &ValidateFieldRequest,
) -> Result<ValidationResponse, AnyhowError> {
    let project_id = match req.context_value {
        Some(id) => id.into_inner(),
        None => {
            return Ok(ValidationResponse {
                valid: false,
                message: Some("Project ID is required for label name validation".to_string()),
            })
        }
    };

    let is_available = tasks::is_label_name_available(
        pool,
        project_id,
        &req.value,
        req.exclude_id.map(|id| id.into_inner()),
    )
    .await?;

    Ok(if is_available {
        ValidationResponse {
            valid: true,
            message: Some("Label name is available".to_string()),
        }
    } else {
        ValidationResponse {
            valid: false,
            message: Some("A label with this name already exists in this project".to_string()),
        }
    })
}
