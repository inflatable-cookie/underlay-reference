//! DB-backed hardening tests for the local auth service (g01.008).
//!
//! These tests exercise the foundation-aligned refresh-replay revocation and
//! per-user 2FA throttling. They follow the workspace pattern of skipping
//! when no test database is configured (DATABASE_URL / TEST_DATABASE_URL).

use super::*;

const TEST_PASSWORD: &str = "Str0ng!Passw0rd!42";

fn ensure_jwt_env() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        if std::env::var("AUTH_JWT_PRIVATE_KEY").is_err() {
            let (_config, key_pair) =
                underlay_auth_jwt::JwtConfig::generate().expect("generate JWT key pair");
            std::env::set_var("AUTH_JWT_PRIVATE_KEY", key_pair.private_key_pkcs8_der_b64);
            std::env::set_var("AUTH_JWT_PUBLIC_KEY", key_pair.public_key_raw_b64);
        }
    });
}

async fn test_service() -> Option<(AcmeLocalAuthService, sqlx::PgPool)> {
    let Ok(url) = std::env::var("DATABASE_URL").or_else(|_| std::env::var("TEST_DATABASE_URL"))
    else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return None;
    };

    let pool = sqlx::PgPool::connect(&url)
        .await
        .expect("connect to test database");

    ensure_jwt_env();
    let service = AcmeLocalAuthService::from_env(pool.clone()).expect("build auth service");
    Some((service, pool))
}

fn unique_email(prefix: &str) -> String {
    format!("{}-{}@example.test", prefix, Uuid::new_v7())
}

#[tokio::test]
async fn refresh_replay_revokes_session_family() {
    let Some((service, _pool)) = test_service().await else {
        return;
    };

    let email = unique_email("replay");
    let session = service
        .register(&email, TEST_PASSWORD, "Replay Test")
        .await
        .expect("register");

    let first_token = session.refresh_token.clone();

    let refreshed = service.refresh(&first_token).await.expect("first refresh");
    let current_token = refreshed.refresh_token.clone();

    // Replaying the superseded token must be rejected...
    let replay_err = service
        .refresh(&first_token)
        .await
        .expect_err("replayed superseded token must be rejected");
    assert!(
        matches!(
            replay_err,
            AuthError::TokenFingerprintMismatch | AuthError::TokenInvalid
        ),
        "unexpected replay error: {replay_err:?}"
    );

    // ...and must revoke the whole session family: the freshest token from
    // the legitimate rotation chain is now unusable too.
    let family_err = service
        .refresh(&current_token)
        .await
        .expect_err("session family must be revoked after replay");
    assert!(
        matches!(family_err, AuthError::SessionRevoked),
        "unexpected post-replay error: {family_err:?}"
    );
}

#[tokio::test]
async fn legitimate_refresh_chain_stays_valid() {
    let Some((service, _pool)) = test_service().await else {
        return;
    };

    let email = unique_email("chain");
    let session = service
        .register(&email, TEST_PASSWORD, "Chain Test")
        .await
        .expect("register");

    // A well-behaved client rotating sequentially never trips reuse detection.
    let mut token = session.refresh_token.clone();
    for _ in 0..3 {
        let refreshed = service.refresh(&token).await.expect("sequential refresh");
        token = refreshed.refresh_token;
    }
}

#[tokio::test]
async fn passkey_internal_credential_read_traverses_all_batches() {
    let Some((service, pool)) = test_service().await else {
        return;
    };

    let registered = service
        .register(
            &unique_email("passkey-batch"),
            TEST_PASSWORD,
            "Passkey Batch Test",
        )
        .await
        .expect("register fixture user");
    let owner_id = registered.user.id.into_inner();
    let foreign_owner_id = uuid::Uuid::now_v7();
    sqlx::query("INSERT INTO auth.users (id, email) VALUES ($1, $2)")
        .bind(foreign_owner_id)
        .bind(unique_email("passkey-foreign"))
        .execute(&pool)
        .await
        .expect("insert foreign owner");

    let credential_ids = (0..205).map(|_| uuid::Uuid::now_v7()).collect::<Vec<_>>();
    let mut expected_ids = credential_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    expected_ids.sort();
    sqlx::query(
        r#"
        INSERT INTO auth.credentials (id, user_id, type, secret_encrypted, metadata, verified)
        SELECT id, $1, 'passkey',
               jsonb_build_object(
                   'credential_id', id::text,
                   'passkey_json', '{}',
                   'counter', NULL::text
               )::text,
               '{}'::jsonb, TRUE
        FROM UNNEST($2::uuid[]) AS credential_ids(id)
        "#,
    )
    .bind(owner_id)
    .bind(&credential_ids)
    .execute(&pool)
    .await
    .expect("insert passkey batch fixtures");

    let excluded_ids = [uuid::Uuid::now_v7(), uuid::Uuid::now_v7()];
    sqlx::query(
        r#"
        INSERT INTO auth.credentials (id, user_id, type, secret_encrypted, metadata, verified)
        SELECT id, user_id, 'passkey',
               jsonb_build_object(
                   'credential_id', id::text,
                   'passkey_json', '{}',
                   'counter', NULL::text
               )::text,
               '{}'::jsonb, verified
        FROM (VALUES ($1::uuid, $2::uuid, FALSE), ($3::uuid, $4::uuid, TRUE))
             AS excluded(id, user_id, verified)
        "#,
    )
    .bind(excluded_ids[0])
    .bind(owner_id)
    .bind(excluded_ids[1])
    .bind(foreign_owner_id)
    .execute(&pool)
    .await
    .expect("insert excluded passkey fixtures");

    let passkeys = service
        .find_passkey_credentials(registered.user.id)
        .await
        .expect("internal passkey read should traverse every batch");
    let actual_ids = passkeys
        .into_iter()
        .map(|passkey| passkey.credential_id)
        .collect::<Vec<_>>();
    assert_eq!(actual_ids, expected_ids);

    sqlx::query("DELETE FROM auth.users WHERE id = $1 OR id = $2")
        .bind(owner_id)
        .bind(foreign_owner_id)
        .execute(&pool)
        .await
        .expect("cleanup passkey batch fixtures");
}

#[tokio::test]
async fn totp_verification_is_throttled_per_user() {
    let Some((service, pool)) = test_service().await else {
        return;
    };

    let email = unique_email("throttle");
    let session = service
        .register(&email, TEST_PASSWORD, "Throttle Test")
        .await
        .expect("register");
    let user_id = session.user.id;

    // Install a TOTP credential directly; without ENCRYPTION_KEY the secret
    // is stored as plaintext base32, which the service accepts in dev/test.
    let credential_id = Uuid::new_v7();
    sqlx::query(
        r#"
        INSERT INTO auth.credentials (
            id, user_id, type, secret_encrypted, metadata, verified,
            created_at, updated_at
        ) VALUES ($1, $2, 'totp', $3, $4, TRUE, NOW(), NOW())
        "#,
    )
    .bind(credential_id.into_inner())
    .bind(user_id.into_inner())
    .bind("JBSWY3DPEHPK3PXP")
    .bind(serde_json::json!({}))
    .execute(&pool)
    .await
    .expect("insert totp credential");

    sqlx::query(
        r#"
        INSERT INTO auth.totp_credential (credential_id, last_counter, backup_code_hashes)
        VALUES ($1, 0, '[]'::jsonb)
        "#,
    )
    .bind(credential_id.into_inner())
    .execute(&pool)
    .await
    .expect("insert totp rotation row");

    // Default per-user 2FA budget is max_totp_attempts (5). Wrong codes must
    // start returning RateLimited before exhausting the outer login limit.
    let mut rate_limited = false;
    for _ in 0..6 {
        match service
            .login_with_password_and_ip(&email, TEST_PASSWORD, Some("000000"), None)
            .await
        {
            Err(AuthError::RateLimited { .. }) => {
                rate_limited = true;
                break;
            }
            Err(_) => {}
            Ok(_) => panic!("wrong TOTP code was accepted"),
        }
    }

    assert!(
        rate_limited,
        "2FA verification was never rate limited after repeated wrong codes"
    );

    // The throttle is per-user: an unrelated user still verifies normally
    // (i.e. fails with a non-rate-limit error on a wrong code).
    let other_email = unique_email("throttle-other");
    let other = service
        .register(&other_email, TEST_PASSWORD, "Throttle Other")
        .await
        .expect("register other");

    let other_credential_id = Uuid::new_v7();
    sqlx::query(
        r#"
        INSERT INTO auth.credentials (
            id, user_id, type, secret_encrypted, metadata, verified,
            created_at, updated_at
        ) VALUES ($1, $2, 'totp', $3, $4, TRUE, NOW(), NOW())
        "#,
    )
    .bind(other_credential_id.into_inner())
    .bind(other.user.id.into_inner())
    .bind("JBSWY3DPEHPK3PXP")
    .bind(serde_json::json!({}))
    .execute(&pool)
    .await
    .expect("insert totp credential (other)");

    sqlx::query(
        r#"
        INSERT INTO auth.totp_credential (credential_id, last_counter, backup_code_hashes)
        VALUES ($1, 0, '[]'::jsonb)
        "#,
    )
    .bind(other_credential_id.into_inner())
    .execute(&pool)
    .await
    .expect("insert totp rotation row (other)");

    let err = service
        .login_with_password_and_ip(&other_email, TEST_PASSWORD, Some("000000"), None)
        .await
        .expect_err("wrong TOTP code must fail");
    assert!(
        !matches!(err, AuthError::RateLimited { .. }),
        "fresh user was rate limited by another user's 2FA attempts"
    );
}
