use super::*;

pub async fn email_totp_request(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<EmailTotpRequestRequest>,
) -> impl IntoResponse {
    // Get the user's email
    let email = match state.local_auth.me(user.user_id.0).await {
        Ok((user_info, _)) => user_info.email,
        Err(err) => {
            return ApiError::new(StatusCode::BAD_REQUEST, err.code(), err.message())
                .with_context(json!({
                    "operation": "auth.email_totp_request.lookup_user",
                    "user_id": user.user_id.0,
                }))
                .into_response();
        }
    };

    let purpose = payload.purpose.to_db_purpose();

    match state
        .email_totp
        .request_code(user.user_id.0, &email, purpose)
        .await
    {
        Ok(expires_at) => {
            let response = EmailTotpRequestResponse { expires_at };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(acme_auth::EmailTotpError::RateLimited) => {
            let mut response = ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "auth.email_totp.rate_limited",
                "Too many code requests. Please wait before requesting another code.",
            )
            .into_response();
            if let Ok(v) = HeaderValue::from_str("3600") {
                response.headers_mut().insert(header::RETRY_AFTER, v);
            }
            response
        }
        Err(err) => ApiError::bad_request(
            "auth.email_totp.request_failed",
            "Email verification code request failed",
        )
        .with_cause(&err)
        .with_context(json!({
            "operation": "auth.email_totp_request.request_code",
            "user_id": user.user_id.0,
            "purpose": format!("{:?}", payload.purpose.to_db_purpose()),
        }))
        .into_response(),
    }
}

pub async fn email_totp_verify(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<EmailTotpVerifyRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let purpose = payload.purpose.to_db_purpose();

    match state
        .email_totp
        .verify_code(user.user_id.0, purpose, &payload.code)
        .await
    {
        Ok(session) => {
            let response = EmailTotpVerifyResponse {
                verification_session_id: session.session_id.to_string(),
                expires_at: session.expires_at,
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(acme_auth::EmailTotpError::InvalidCode) => {
            ApiError::bad_request("auth.email_totp.invalid_code", "Invalid verification code")
                .into_response()
        }
        Err(acme_auth::EmailTotpError::CodeExpired) => ApiError::bad_request(
            "auth.email_totp.code_expired",
            "Verification code has expired. Please request a new one.",
        )
        .into_response(),
        Err(acme_auth::EmailTotpError::TooManyAttempts) => ApiError::bad_request(
            "auth.email_totp.too_many_attempts",
            "Too many invalid attempts. Please request a new code.",
        )
        .into_response(),
        Err(acme_auth::EmailTotpError::NoActiveCode) => ApiError::bad_request(
            "auth.email_totp.no_active_code",
            "No active verification code found. Please request a new one.",
        )
        .into_response(),
        Err(err) => ApiError::bad_request(
            "auth.email_totp.verify_failed",
            "Email verification failed",
        )
        .with_cause(&err)
        .with_context(json!({
            "operation": "auth.email_totp_verify.verify_code",
            "user_id": user.user_id.0,
            "purpose": format!("{purpose:?}"),
        }))
        .into_response(),
    }
}
