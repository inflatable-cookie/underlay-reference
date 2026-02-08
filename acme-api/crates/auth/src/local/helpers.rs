use super::*;

pub(super) fn roles_for_user(role: &str) -> Vec<String> {
    match role {
        "superadmin" => vec!["superadmin".to_string()],
        "admin" => vec!["admin".to_string()],
        "support" => vec!["support".to_string()],
        "tester" => vec!["tester".to_string()],
        _ => vec!["user".to_string()],
    }
}

pub(super) fn map_user_row(row: sqlx::postgres::PgRow) -> User {
    let id: sqlx::types::Uuid = row.get("id");
    let status: String = row.get("status");

    let status = match status.as_str() {
        "active" => UserStatus::Active,
        "suspended" => UserStatus::Suspended,
        "deleted" => UserStatus::Deleted,
        _ => UserStatus::Active,
    };

    let display_name: Option<String> = row.get("display_name");

    User {
        id: Uuid(id),
        email: row.get("email"),
        display_name,
        status,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub(super) fn map_credential_row(row: sqlx::postgres::PgRow) -> Credential {
    let id: sqlx::types::Uuid = row.get("id");
    let user_id: sqlx::types::Uuid = row.get("user_id");
    let credential_type: String = row.get("credential_type");
    let metadata: serde_json::Value = row.get("metadata");

    let credential_type = match credential_type.as_str() {
        "password" => CredentialType::Password,
        "totp" => CredentialType::Totp,
        "passkey" => CredentialType::Passkey,
        "oauth_google" => CredentialType::OAuthGoogle,
        _ => CredentialType::Password,
    };

    let metadata = serde_json::from_value::<CredentialMetadata>(metadata).unwrap_or(
        CredentialMetadata::Password {
            algorithm: "argon2id".to_string(),
            memory_kb: 65536,
            iterations: 3,
            parallelism: 4,
        },
    );

    Credential {
        id: Uuid(id),
        user_id: Uuid(user_id),
        credential_type,
        secret_encrypted: row.get("secret_encrypted"),
        metadata,
        verified: row.get("verified"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        last_used_at: row.get("last_used_at"),
    }
}

pub(super) fn map_session_row(row: sqlx::postgres::PgRow) -> DbSession {
    let id: sqlx::types::Uuid = row.get("id");
    let user_id: sqlx::types::Uuid = row.get("user_id");
    let roles_value: serde_json::Value = row.get("roles");
    let roles = serde_json::from_value::<Vec<String>>(roles_value).unwrap_or_default();

    let status: String = row.get("status");
    let status = match status.as_str() {
        "active" => SessionStatus::Active,
        "revoked" => SessionStatus::Revoked,
        "expired" => SessionStatus::Expired,
        _ => SessionStatus::Active,
    };

    DbSession {
        id: Uuid(id),
        user_id: Uuid(user_id),
        roles,
        is_active: row.get("is_active"),
        access_token_fingerprint: row.get("access_token_fingerprint"),
        refresh_token_fingerprint: row.get("refresh_token_fingerprint"),
        refresh_token_id: Uuid(row.get("refresh_token_id")),
        refresh_token_version: row.get("refresh_token_version"),
        access_token_expires_at: row.get("access_token_expires_at"),
        refresh_token_expires_at: row.get("refresh_token_expires_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        last_used_at: row.get("last_used_at"),
        ip_address: row.get("ip_address"),
        user_agent: row.get("user_agent"),
        status,
        revocation_reason: row.get("revocation_reason"),
        revoked_at: row.get("revoked_at"),
    }
}

pub(super) fn timestamp_to_datetime(ts: u64) -> DateTime<Utc> {
    Utc.timestamp_opt(ts as i64, 0)
        .single()
        .unwrap_or_else(Utc::now)
}

pub(super) fn session_status_db(status: &SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "active",
        SessionStatus::Revoked => "revoked",
        SessionStatus::Expired => "expired",
    }
}
