use super::*;

pub async fn list_sessions(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.list_sessions(user.user_id.0).await {
        Ok(sessions) => {
            let data: Vec<SessionDto> = sessions.into_iter().map(SessionDto::from).collect();
            (StatusCode::OK, Json(ListResponse { data })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn revoke_session(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let session_uuid = match Uuid::parse_str(&session_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::bad_request("validation.invalid_session_id", "Invalid session id")
                .into_response();
        }
    };

    match state
        .local_auth
        .revoke_session_for_user(user.user_id.0, session_uuid, "user_revoked")
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => {
            let status = match err.code() {
                "auth.forbidden" => StatusCode::FORBIDDEN,
                "auth.bad_request" => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            ApiError::new(status, err.code(), err.message()).into_response()
        }
    }
}
