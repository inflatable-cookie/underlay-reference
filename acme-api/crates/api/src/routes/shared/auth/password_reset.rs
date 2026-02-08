use super::*;

pub async fn change_password_with_verification(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordWithVerificationRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let verification_session_id = match Uuid::parse_str(&payload.verification_session_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::bad_request(
                "validation.invalid_session_id",
                "Invalid verification session ID",
            )
            .into_response();
        }
    };

    match state
        .local_auth
        .change_password_with_verification(
            user.user_id.0,
            verification_session_id,
            &payload.new_password,
        )
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(underlay_auth::AuthError::BadRequest(msg)) if msg.contains("verification session") => {
            ApiError::new(
                StatusCode::FORBIDDEN,
                "auth.password.invalid_verification_session",
                "Invalid, expired, or already used verification session",
            )
            .into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Request a password reset email.
///
/// Sends a verification code to the provided email if the account exists.
/// Always returns success to prevent email enumeration attacks.
pub async fn password_reset_request(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetRequestRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    // Validate input
    if let Err(validation_err) = payload.validate() {
        ensure_min_response_time(start).await;
        return validation_error_response(validation_err).into_response();
    }

    let email = payload.email.trim();

    // Check if user exists (without revealing to the client)
    match state.local_auth.get_user_for_password_reset(email).await {
        Ok(Some((user_id, user_email))) => {
            // User exists - send the reset code
            if let Err(e) = state
                .email_totp
                .request_code(
                    user_id,
                    &user_email,
                    acme_db::auth::EmailTotpPurpose::PasswordReset,
                )
                .await
            {
                // Log error but don't reveal to client
                tracing::error!("Failed to send password reset email: {}", e);
            }
        }
        Ok(None) => {
            // User doesn't exist - don't reveal this to the client
            tracing::debug!("Password reset requested for non-existent email: {}", email);
        }
        Err(e) => {
            // Error looking up user - don't reveal to client
            tracing::error!("Error looking up user for password reset: {:?}", e);
        }
    }

    // Always return success to prevent email enumeration
    ensure_min_response_time(start).await;
    StatusCode::NO_CONTENT.into_response()
}

/// Verify a password reset code.
///
/// Returns a reset token that can be used to complete the password reset.
pub async fn password_reset_verify(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetVerifyRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let email = payload.email.trim();
    let code = payload.code.trim();

    match state
        .local_auth
        .verify_password_reset_code(email, code, &state.email_totp)
        .await
    {
        Ok(session) => {
            let response = PasswordResetVerifyResponse {
                reset_token: session.session_id.to_string(),
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(underlay_auth::AuthError::BadRequest(msg)) => {
            ApiError::bad_request("auth.password_reset.invalid_code", msg).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

/// Complete a password reset using a reset token.
///
/// Sets the new password and revokes all existing sessions.
pub async fn password_reset_complete(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetCompleteRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let reset_token = match Uuid::parse_str(payload.reset_token.trim()) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::bad_request("validation.invalid_reset_token", "Invalid reset token")
                .into_response();
        }
    };

    match state
        .local_auth
        .complete_password_reset(reset_token, &payload.new_password)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(underlay_auth::AuthError::BadRequest(msg)) if msg.contains("reset token") => {
            ApiError::bad_request(
                "auth.password_reset.invalid_token",
                "Invalid or expired reset token",
            )
            .into_response()
        }
        Err(underlay_auth::AuthError::PasswordTooWeak) => ApiError::bad_request(
            "auth.password.too_weak",
            "Password does not meet strength requirements",
        )
        .into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}
