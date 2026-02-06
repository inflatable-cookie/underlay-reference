//! Activity log routes.
//!
//! Provides endpoints for viewing activity history (audit log).

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use underlay_http::ApiError;
use uuid::Uuid;

use acme_db::activity;

use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

/// Activity entry response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ActivityResponse {
    pub id: String,
    pub occurred_at: String,
    pub actor: Option<ActorResponse>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub details: JsonValue,
}

/// Actor (user) info in activity response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ActorResponse {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
}

impl From<activity::ActivityWithActorRow> for ActivityResponse {
    fn from(row: activity::ActivityWithActorRow) -> Self {
        let actor = row.user_id.map(|user_id| ActorResponse {
            id: user_id.to_string(),
            email: row.actor_email.unwrap_or_else(|| "Unknown".to_string()),
            display_name: row.actor_display_name,
        });

        Self {
            id: row.id.to_string(),
            occurred_at: row.occurred_at.to_rfc3339(),
            actor,
            action: row.action,
            resource_type: row.resource_type,
            resource_id: row.resource_id.to_string(),
            details: row.details,
        }
    }
}

/// Query parameters for activity listing.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListActivityQuery {
    /// Limit (default 50)
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
}

// ============================================================================
// Handlers
// ============================================================================

/// List all activity (global feed).
///
/// GET /v1/admin/activity
pub async fn list_activity(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Query(query): Query<ListActivityQuery>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match activity::list_activity(pool, limit, offset).await {
        Ok(response) => {
            let items: Vec<ActivityResponse> = response.data.into_iter().map(Into::into).collect();
            Ok(Json(serde_json::json!({
                "data": items,
                "has_more": response.has_more,
                "total": response.total
            }))
            .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list activity: {}", e);
            Err(
                ApiError::internal("activity.list_failed", "Failed to list activity")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "activity.list",
                        "limit": limit,
                        "offset": offset
                    })),
            )
        }
    }
}

/// List activity for a specific entity.
///
/// GET /v1/admin/activity/entity/:entity_type/:entity_id
pub async fn list_activity_for_entity(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(query): Query<ListActivityQuery>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match activity::list_activity_for_entity(pool, &entity_type, entity_id, limit, offset).await {
        Ok(response) => {
            let items: Vec<ActivityResponse> = response.data.into_iter().map(Into::into).collect();
            Ok(Json(serde_json::json!({
                "data": items,
                "has_more": response.has_more,
                "total": response.total
            }))
            .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list entity activity: {}", e);
            Err(
                ApiError::internal("activity.entity_list_failed", "Failed to list activity")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "activity.list_for_entity",
                        "entity_type": entity_type,
                        "entity_id": entity_id,
                        "limit": limit,
                        "offset": offset
                    })),
            )
        }
    }
}

/// List activity performed by a specific user.
///
/// GET /v1/admin/users/:user_id/activity
pub async fn list_activity_for_user(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ListActivityQuery>,
) -> Result<Response, ApiError> {
    let pool = state.local_auth.pool();

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match activity::list_activity_for_user(pool, user_id, limit, offset).await {
        Ok(response) => {
            let items: Vec<ActivityResponse> = response.data.into_iter().map(Into::into).collect();
            Ok(Json(serde_json::json!({
                "data": items,
                "has_more": response.has_more,
                "total": response.total
            }))
            .into_response())
        }
        Err(e) => {
            tracing::error!("Failed to list user activity: {}", e);
            Err(
                ApiError::internal("activity.user_list_failed", "Failed to list activity")
                    .with_cause(&e)
                    .with_context(serde_json::json!({
                        "operation": "activity.list_for_user",
                        "user_id": user_id,
                        "limit": limit,
                        "offset": offset
                    })),
            )
        }
    }
}
