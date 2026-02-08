use super::*;

pub async fn totp_setup(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.totp_setup(user.user_id.0).await {
        Ok(setup) => (StatusCode::OK, Json(SingleResponse { data: setup })).into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn totp_enable(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<TotpEnableRequest>,
) -> impl IntoResponse {
    let setup_id = match Uuid::parse_str(&payload.setup_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::bad_request("validation.invalid_setup_id", "Invalid setup id")
                .into_response();
        }
    };

    match state
        .local_auth
        .totp_enable(user.user_id.0, setup_id, &payload.code)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn totp_disable(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.totp_disable(user.user_id.0).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn totp_status(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.totp_is_enabled(user.user_id.0).await {
        Ok(enabled) => (
            StatusCode::OK,
            Json(SingleResponse {
                data: TotpStatusResponse { enabled },
            }),
        )
            .into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Get comprehensive 2FA status for the current user.
pub async fn two_factor_status(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.get_2fa_status(user.user_id.0).await {
        Ok(status) => (
            StatusCode::OK,
            Json(SingleResponse {
                data: TwoFactorStatusResponse {
                    has_totp_configured: status.has_totp_configured,
                    has_passkey_configured: status.has_passkey_configured,
                    totp_enabled_at: status.totp_enabled_at,
                    passkey_count: status.passkey_count,
                },
            }),
        )
            .into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn totp_verify(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<TotpVerifyRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let purpose = payload.purpose.to_db_purpose();

    match state
        .local_auth
        .verify_totp_for_verification(user.user_id.0, &payload.code, purpose)
        .await
    {
        Ok(session) => {
            let response = EmailTotpVerifyResponse {
                verification_session_id: session.id.to_string(),
                expires_at: session.expires_at,
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(underlay_auth::AuthError::TwoFactorNotSetUp) => ApiError::bad_request(
            "auth.totp.not_configured",
            "TOTP is not configured for this account",
        )
        .into_response(),
        Err(underlay_auth::AuthError::TwoFactorInvalid) => {
            ApiError::bad_request("auth.totp.invalid_code", "Invalid TOTP code").into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Get password requirements.
///
/// Returns the password requirements configuration so UIs can display
/// accurate feedback without hardcoding values.
pub async fn password_requirements(State(state): State<AppState>) -> impl IntoResponse {
    let requirements = state.local_auth.password_requirements();
    // Return as JSON directly - the PasswordRequirements type is already Serialize
    (StatusCode::OK, Json(json!({ "data": requirements })))
}
