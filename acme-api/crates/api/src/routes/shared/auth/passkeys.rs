use super::*;

use crate::dto::auth::{
    PasskeyCredentialDto, PasskeyLoginFinishRequest, PasskeyLoginStartDto,
    PasskeyLoginStartRequest, PasskeyRegisterFinishRequest, PasskeyRenameRequest,
    PasskeyStartRegistrationDto, PasskeyVerifyFinishRequest, PasskeyVerifyStartRequest,
};

/// List all passkeys for the current user.
pub async fn list_passkeys(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.list_passkeys(user.user_id.0).await {
        Ok(passkeys) => {
            let data: Vec<PasskeyCredentialDto> = passkeys
                .into_iter()
                .map(|pk| PasskeyCredentialDto {
                    id: pk.credential.id.to_string(),
                    display_name: pk.display_name,
                    metadata: serde_json::to_value(&pk.credential.metadata).unwrap_or_default(),
                    created_at: pk.credential.created_at,
                    last_used_at: pk.credential.last_used_at,
                })
                .collect();
            (StatusCode::OK, Json(ListResponse { data })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Rename a passkey.
pub async fn rename_passkey(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(credential_id): Path<String>,
    Json(payload): Json<PasskeyRenameRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let credential_uuid = match Uuid::parse_str(&credential_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::bad_request(
                "validation.invalid_credential_id",
                "Invalid credential ID",
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .rename_passkey(user.user_id.0, credential_uuid, &payload.display_name)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Delete a passkey.
pub async fn delete_passkey(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Path(credential_id): Path<String>,
) -> impl IntoResponse {
    let credential_uuid = match Uuid::parse_str(&credential_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::bad_request(
                "validation.invalid_credential_id",
                "Invalid credential ID",
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .delete_passkey(user.user_id.0, credential_uuid)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Start passkey registration for the current user.
pub async fn passkey_register_start(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state
        .local_auth
        .passkey_register_start(user.user_id.0)
        .await
    {
        Ok(response) => {
            let dto = PasskeyStartRegistrationDto {
                options: response.options,
                state_id: response.state_id,
            };
            (StatusCode::OK, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Finish passkey registration.
pub async fn passkey_register_finish(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyRegisterFinishRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let state_id = match Uuid::parse_str(&payload.state_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::bad_request("validation.invalid_state_id", "Invalid state ID")
                .into_response();
        }
    };

    match state
        .local_auth
        .passkey_register_finish(
            user.user_id.0,
            state_id,
            payload.credential,
            payload.display_name.as_deref(),
        )
        .await
    {
        Ok(_credential) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Start passkey login (unauthenticated).
pub async fn passkey_login_start(
    State(state): State<AppState>,
    Json(payload): Json<PasskeyLoginStartRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    match state
        .local_auth
        .passkey_login_start(payload.email.as_deref())
        .await
    {
        Ok(response) => {
            let dto = PasskeyLoginStartDto {
                options: response.options,
                state_id: response.state_id,
            };
            (StatusCode::OK, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Finish passkey login (unauthenticated).
pub async fn passkey_login_finish(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyLoginFinishRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let state_id = match Uuid::parse_str(&payload.state_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::bad_request("validation.invalid_state_id", "Invalid state ID")
                .into_response();
        }
    };

    let session_fp = extract_session_fingerprint(&headers, &state.trusted_proxy_config);

    match state
        .local_auth
        .passkey_login_finish(state_id, payload.credential, Some(session_fp))
        .await
    {
        Ok(session) => {
            let role = state
                .local_auth
                .me(session.user.id)
                .await
                .map(|(_, r)| r)
                .unwrap_or_else(|_| "user".to_string());

            let refresh_token = session.refresh_token.clone();
            let dto = auth_session_dto_from_session(session, role);

            let mut response_headers = HeaderMap::new();
            if let Err(e) =
                set_auth_cookies(&mut response_headers, &refresh_token, &state.cookie_config)
            {
                tracing::warn!("Failed to set auth cookies: {}", e);
            }

            (
                StatusCode::OK,
                response_headers,
                Json(SingleResponse { data: dto }),
            )
                .into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Start passkey verification (for 2FA gates).
pub async fn passkey_verify_start(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyVerifyStartRequest>,
) -> impl IntoResponse {
    let purpose = payload.purpose.to_db_purpose();

    match state
        .local_auth
        .passkey_verify_start(user.user_id.0, purpose)
        .await
    {
        Ok(response) => {
            let dto = PasskeyLoginStartDto {
                options: response.options,
                state_id: response.state_id,
            };
            (StatusCode::OK, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Finish passkey verification (for 2FA gates).
pub async fn passkey_verify_finish(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<PasskeyVerifyFinishRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let state_id = match Uuid::parse_str(&payload.state_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::bad_request("validation.invalid_state_id", "Invalid state ID")
                .into_response();
        }
    };

    match state
        .local_auth
        .passkey_verify_finish(user.user_id.0, state_id, payload.credential)
        .await
    {
        Ok(session) => {
            let response = crate::dto::auth::PasskeyVerifyResponse {
                verification_session_id: session.id.to_string(),
                expires_at: session.expires_at,
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}
