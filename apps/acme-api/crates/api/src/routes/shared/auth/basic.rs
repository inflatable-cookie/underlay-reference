use super::*;

pub async fn register(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let email = payload.email.trim();
    let password = payload.password.trim();
    let display_name = payload.display_name.trim();

    let client_ip = acme_infra::extract_client_ip(&headers, &state.trusted_proxy_config);

    match state
        .local_auth
        .register_with_ip(email, password, display_name, client_ip.as_deref())
        .await
    {
        Ok(session) => {
            // Enqueue welcome email job
            if let Some(ref job_repo) = state.job_repository {
                let job_payload = serde_json::json!({
                    "user_id": session.user.id,
                    "email": email,
                    "display_name": display_name,
                });
                if let Err(e) = job_repo
                    .create("email.welcome", job_payload, &Default::default())
                    .await
                {
                    tracing::warn!("Failed to enqueue welcome email job: {}", e);
                }
            }

            let refresh_token = session.refresh_token.clone();
            let include_refresh_token = include_refresh_token_in_body(&headers);
            let dto =
                auth_session_dto_from_session(session, "user".to_string(), include_refresh_token);

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

pub async fn login(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let email = payload.email.trim();
    let password = payload.password.trim();

    let client_ip = acme_infra::extract_client_ip(&headers, &state.trusted_proxy_config);

    match state
        .local_auth
        .login_with_password_and_ip(
            email,
            password,
            payload.code.as_deref(),
            client_ip.as_deref(),
        )
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
            let include_refresh_token = include_refresh_token_in_body(&headers);
            let dto = auth_session_dto_from_session(session, role, include_refresh_token);

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

pub async fn login_start(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LoginStartRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let email = payload.email.trim();
    let password = payload.password.trim();

    let fingerprint = login_client_fingerprint(&headers, &state.trusted_proxy_config);
    let client_ip = acme_infra::extract_client_ip(&headers, &state.trusted_proxy_config);

    // Use the method that supports email fallback
    match state
        .local_auth
        .login_start_with_email_fallback(
            email,
            password,
            &fingerprint,
            client_ip.as_deref(),
            payload.enforce_email_fallback,
        )
        .await
    {
        Ok(acme_auth::LoginStartOutcome::Complete { session, role }) => {
            let refresh_token = session.refresh_token.clone();
            let include_refresh_token = include_refresh_token_in_body(&headers);
            let dto = auth_session_dto_from_session(*session, role, include_refresh_token);
            let response = LoginStartResponse {
                requires_two_factor: false,
                is_email_verification: None,
                login_state_id: None,
                session: Some(dto),
            };

            let mut response_headers = HeaderMap::new();
            if let Err(e) =
                set_auth_cookies(&mut response_headers, &refresh_token, &state.cookie_config)
            {
                tracing::warn!("Failed to set auth cookies: {}", e);
            }

            (
                StatusCode::OK,
                response_headers,
                Json(SingleResponse { data: response }),
            )
                .into_response()
        }
        Ok(acme_auth::LoginStartOutcome::TwoFactorRequired { login_state_id }) => {
            let response = LoginStartResponse {
                requires_two_factor: true,
                is_email_verification: Some(false),
                login_state_id: Some(login_state_id.to_string()),
                session: None,
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Ok(acme_auth::LoginStartOutcome::EmailVerificationRequired {
            login_state_id,
            user_id,
            email,
        }) => {
            // Send the email verification code
            if let Err(e) = state
                .email_totp
                .request_code(user_id, &email, acme_db::auth::EmailTotpPurpose::Login)
                .await
            {
                tracing::error!("Failed to send email code: {}", e);
                return crate::db_errors::internal_with_diagnostics(
                    "auth.email_send_failed",
                    "Failed to send verification email",
                    &e,
                )
                .with_context(json!({
                    "operation": "auth.login_start",
                    "user_id": user_id
                }))
                .into_response();
            }

            let response = LoginStartResponse {
                requires_two_factor: true,
                is_email_verification: Some(true),
                login_state_id: Some(login_state_id.to_string()),
                session: None,
            };
            (StatusCode::OK, Json(SingleResponse { data: response })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn login_finish(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LoginFinishRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    let code = payload.code.trim();

    let state_id = match Uuid::parse_str(payload.login_state_id.trim()) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::bad_request("validation.invalid_login_state", "Invalid login state")
                .into_response();
        }
    };

    let fingerprint = login_client_fingerprint(&headers, &state.trusted_proxy_config);
    let session_fp = extract_session_fingerprint(&headers, &state.trusted_proxy_config);

    // Try TOTP first
    match state
        .local_auth
        .login_finish_with_totp(state_id, code, &fingerprint, Some(session_fp.clone()))
        .await
    {
        Ok((session, role)) => {
            let refresh_token = session.refresh_token.clone();
            let include_refresh_token = include_refresh_token_in_body(&headers);
            let dto = auth_session_dto_from_session(session, role, include_refresh_token);

            let mut response_headers = HeaderMap::new();
            if let Err(e) =
                set_auth_cookies(&mut response_headers, &refresh_token, &state.cookie_config)
            {
                tracing::warn!("Failed to set auth cookies: {}", e);
            }

            return (
                StatusCode::OK,
                response_headers,
                Json(SingleResponse { data: dto }),
            )
                .into_response();
        }
        Err(underlay_auth::AuthError::TwoFactorNotSetUp) => {
            // Not a TOTP state - try email verification
        }
        Err(err) => {
            return map_auth_error_to_response(err);
        }
    }

    // Try email verification
    // First validate the state and code
    let user_id = match state
        .local_auth
        .get_email_login_state(state_id, &fingerprint)
        .await
    {
        Ok(uid) => uid,
        Err(err) => return map_auth_error_to_response(err),
    };

    // Verify the email code
    match state
        .email_totp
        .verify_code(user_id, acme_db::auth::EmailTotpPurpose::Login, code)
        .await
    {
        Ok(_session) => {
            // Email verified - complete the login
            match state
                .local_auth
                .login_finish_email_verified(state_id, Some(session_fp))
                .await
            {
                Ok((session, role)) => {
                    let refresh_token = session.refresh_token.clone();
                    let include_refresh_token = include_refresh_token_in_body(&headers);
                    let dto = auth_session_dto_from_session(session, role, include_refresh_token);

                    let mut response_headers = HeaderMap::new();
                    if let Err(e) = set_auth_cookies(
                        &mut response_headers,
                        &refresh_token,
                        &state.cookie_config,
                    ) {
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
        Err(acme_auth::EmailTotpError::InvalidCode) => {
            let _ = state
                .local_auth
                .increment_email_login_attempts(state_id)
                .await;
            ApiError::bad_request("auth.invalid_code", "Invalid verification code").into_response()
        }
        Err(acme_auth::EmailTotpError::CodeExpired) => {
            ApiError::bad_request("auth.code_expired", "Verification code has expired")
                .into_response()
        }
        Err(acme_auth::EmailTotpError::TooManyAttempts) => {
            ApiError::bad_request("auth.too_many_attempts", "Too many invalid attempts")
                .into_response()
        }
        Err(e) => {
            tracing::error!("Email TOTP verification error: {}", e);
            ApiError::bad_request("auth.verification_failed", "Verification failed")
                .with_cause(&e)
                .with_context(json!({
                    "operation": "auth.login_finish_email_verify",
                    "state_id": state_id
                }))
                .into_response()
        }
    }
}

pub async fn refresh(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    // Extract fingerprint for rate limiting and session validation
    let fingerprint = extract_session_fingerprint(&headers, &state.trusted_proxy_config);

    // Check rate limit before processing
    if let Err(err) = state
        .local_auth
        .check_refresh_rate_limit(&fingerprint)
        .await
    {
        return map_auth_error_to_response(err);
    }

    // Accept refresh token from body (mobile) or cookie (browser)
    let refresh_token = if !payload.refresh_token.is_empty() {
        payload.refresh_token.clone()
    } else if let Some(cookie_token) = extract_refresh_token(&headers, &state.cookie_config) {
        cookie_token
    } else {
        return ApiError::new(
            StatusCode::UNAUTHORIZED,
            "auth.missing_refresh_token",
            "No refresh token provided",
        )
        .into_response();
    };

    match state
        .local_auth
        .refresh_with_fingerprint(&refresh_token, Some(fingerprint))
        .await
    {
        Ok(session) => {
            let role = state
                .local_auth
                .me(session.user.id)
                .await
                .map(|(_, r)| r)
                .unwrap_or_else(|_| "user".to_string());

            let new_refresh_token = session.refresh_token.clone();
            let include_refresh_token = include_refresh_token_in_body(&headers);
            let dto = auth_session_dto_from_session(session, role, include_refresh_token);

            let mut response_headers = HeaderMap::new();
            if let Err(e) = set_auth_cookies(
                &mut response_headers,
                &new_refresh_token,
                &state.cookie_config,
            ) {
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

pub async fn logout(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> impl IntoResponse {
    // Accept refresh token from body (mobile) or cookie (browser)
    let refresh_token = if !payload.refresh_token.is_empty() {
        payload.refresh_token.clone()
    } else if let Some(cookie_token) = extract_refresh_token(&headers, &state.cookie_config) {
        cookie_token
    } else {
        // No token provided - just clear cookies and return success
        // This handles the case where cookies were already cleared
        let mut response_headers = HeaderMap::new();
        let _ = clear_auth_cookies(&mut response_headers, &state.cookie_config);
        return (StatusCode::NO_CONTENT, response_headers).into_response();
    };

    match state.local_auth.logout(&refresh_token).await {
        Ok(()) => {
            let mut response_headers = HeaderMap::new();
            if let Err(e) = clear_auth_cookies(&mut response_headers, &state.cookie_config) {
                tracing::warn!("Failed to clear auth cookies: {}", e);
            }
            (StatusCode::NO_CONTENT, response_headers).into_response()
        }
        Err(err) => {
            // Even on error, try to clear cookies
            let mut response_headers = HeaderMap::new();
            let _ = clear_auth_cookies(&mut response_headers, &state.cookie_config);
            (
                StatusCode::BAD_REQUEST,
                response_headers,
                ApiError::new(StatusCode::BAD_REQUEST, err.code(), err.message()).into_response(),
            )
                .into_response()
        }
    }
}

pub async fn me(
    AuthenticatedUser(principal): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.local_auth.me(principal.user_id.0).await {
        Ok((user, role)) => {
            // Use display_name if present, otherwise fall back to email username
            let display_name = user
                .display_name
                .unwrap_or_else(|| user.email.split('@').next().unwrap_or("User").to_string());

            let dto = AuthUserDto {
                user_id: user.id.to_string(),
                email: user.email,
                display_name,
                roles: roles_for_user(&role),
            };
            (StatusCode::OK, Json(SingleResponse { data: dto })).into_response()
        }
        Err(err) => map_auth_error_to_response(err),
    }
}

pub async fn change_password(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(validation_err) = payload.validate() {
        return validation_error_response(validation_err).into_response();
    }

    match state
        .local_auth
        .change_password(
            user.user_id.0,
            &payload.current_password,
            &payload.new_password,
        )
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error_to_response(err),
    }
}
