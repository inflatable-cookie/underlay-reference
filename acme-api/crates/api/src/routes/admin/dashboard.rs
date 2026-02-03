//! Admin dashboard routes.
//!
//! Provides aggregate statistics for the admin dashboard.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use acme_db::stats;

use crate::state::{AdminUser, AppState};

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCountsDto {
    pub active: i64,
    pub suspended: i64,
    pub deleted: i64,
    pub total: i64,
}

impl From<stats::UserCounts> for UserCountsDto {
    fn from(counts: stats::UserCounts) -> Self {
        Self {
            active: counts.active,
            suspended: counts.suspended,
            deleted: counts.deleted,
            total: counts.total,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStatsResponse {
    pub user_counts: UserCountsDto,
    pub media_count: i64,
    pub recent_registrations: i64,
    pub active_sessions: i64,
}

// ============================================================================
// Handlers
// ============================================================================

/// Get dashboard statistics.
///
/// GET /v1/admin/dashboard/stats
pub async fn get_dashboard_stats(
    AdminUser(_user): AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = state.local_auth.pool();

    // Fetch all stats in parallel-ish manner
    let user_counts = match stats::get_user_counts(pool).await {
        Ok(counts) => counts,
        Err(e) => {
            tracing::error!("Failed to get user counts: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let media_count = match stats::get_media_count(pool).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to get media count: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let recent_registrations = match stats::get_recent_registrations(pool, 7).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to get recent registrations: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let active_sessions = match stats::get_active_sessions_count(pool).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to get active sessions count: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let response = DashboardStatsResponse {
        user_counts: user_counts.into(),
        media_count,
        recent_registrations,
        active_sessions,
    };

    Json(serde_json::json!({ "data": response })).into_response()
}
