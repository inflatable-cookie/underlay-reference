use super::helpers::map_credential_row;
use super::*;

impl AcmeLocalAuthService {
    /// Validate password strength.
    pub(super) fn validate_password(&self, password: &str) -> AuthResult<()> {
        if password.trim().is_empty() {
            return Err(AuthError::BadRequest("Password is required".into()));
        }

        self.password_analyzer.validate(password).map_err(|msg| {
            if msg.contains("common") {
                AuthError::BadRequest(
                    "This password is too common. Please choose a stronger password.".into(),
                )
            } else {
                AuthError::PasswordTooWeak
            }
        })?;

        Ok(())
    }

    /// Get password requirements configuration.
    pub fn password_requirements(&self) -> underlay_auth_password::PasswordRequirements {
        self.password_analyzer.requirements()
    }

    pub(super) async fn find_password_credential(
        &self,
        user_id: Uuid,
    ) -> AuthResult<Option<Credential>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, type as credential_type, secret_encrypted, metadata, verified,
                   created_at, updated_at, last_used_at
            FROM auth.credentials
            WHERE user_id = $1 AND type = 'password'
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(row.map(map_credential_row))
    }

    pub(super) async fn find_password_credential_and_role(
        &self,
        user_id: Uuid,
    ) -> AuthResult<Option<(Credential, String)>> {
        let row = sqlx::query(
            r#"
            SELECT c.id, c.user_id, c.type as credential_type, c.secret_encrypted, c.metadata, c.verified,
                   c.created_at, c.updated_at, c.last_used_at,
                   u.role
            FROM auth.credentials c
            JOIN auth.users u ON u.id = c.user_id
            WHERE c.user_id = $1 AND c.type = 'password'
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let role: String = row.get("role");
        let credential = map_credential_row(row);
        Ok(Some((credential, role)))
    }

    pub(super) async fn set_password(
        &self,
        user_id: Uuid,
        password: &str,
    ) -> AuthResult<Credential> {
        self.validate_password(password)?;

        let hash =
            hash_password_blocking(self.password_hasher.clone(), password.as_bytes().to_vec())
                .await?;

        let id = Uuid::new_v7();

        let metadata = CredentialMetadata::Password {
            algorithm: "argon2id".to_string(),
            memory_kb: self.argon2_memory_kb,
            iterations: self.argon2_iterations,
            parallelism: self.argon2_parallelism,
        };

        let metadata_json = serde_json::to_value(&metadata)
            .map_err(|_| AuthError::Internal("Failed to encode credential metadata".into()))?;

        let row = sqlx::query(
            r#"
            INSERT INTO auth.credentials (
                id, user_id, type, secret_encrypted, metadata, verified
            ) VALUES ($1, $2, 'password', $3, $4, TRUE)
            RETURNING id, user_id, type as credential_type, secret_encrypted, metadata, verified,
                      created_at, updated_at, last_used_at
            "#,
        )
        .bind(id.into_inner())
        .bind(user_id.into_inner())
        .bind(&hash)
        .bind(metadata_json)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(map_credential_row(row))
    }

    pub(super) async fn update_credential_last_used(&self, credential_id: Uuid) -> AuthResult<()> {
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE auth.credentials
            SET last_used_at = $2,
                updated_at = $2
            WHERE id = $1
            "#,
        )
        .bind(credential_id.into_inner())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error".into()))?;

        Ok(())
    }
}
