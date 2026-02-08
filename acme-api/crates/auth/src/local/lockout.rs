use super::*;

impl AcmeLocalAuthService {
    /// Check if a user is currently locked out.
    pub(super) async fn check_lockout(&self, user_id: Uuid) -> AuthResult<Option<u64>> {
        let row = sqlx::query(
            r#"
            SELECT lockout_until
            FROM auth.users
            WHERE id = $1
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error checking lockout".into()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let lockout_until: Option<DateTime<Utc>> = row.get("lockout_until");

        if let Some(until) = lockout_until {
            let now = Utc::now();
            if until > now {
                let remaining = (until - now).num_seconds().max(0) as u64;
                return Ok(Some(remaining));
            }
        }

        Ok(None)
    }

    /// Record a failed login attempt.
    pub(super) async fn record_failed_login(
        &self,
        user_id: Uuid,
        ip: Option<&str>,
        reason: &str,
    ) -> AuthResult<Option<u64>> {
        // Log the attempt
        sqlx::query(
            r#"
            INSERT INTO auth.login_attempts (user_id, ip_address, success, failure_reason)
            VALUES ($1, $2::inet, FALSE, $3)
            "#,
        )
        .bind(user_id.into_inner())
        .bind(ip)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error logging login attempt".into()))?;

        // Increment failed count and check for lockout
        let row = sqlx::query(
            r#"
            UPDATE auth.users
            SET
                failed_login_count = failed_login_count + 1,
                lockout_until = CASE
                    WHEN failed_login_count + 1 >= $2
                    THEN NOW() + ($3 || ' seconds')::interval
                    ELSE lockout_until
                END
            WHERE id = $1
            RETURNING failed_login_count, lockout_until
            "#,
        )
        .bind(user_id.into_inner())
        .bind(self.config.max_failed_logins as i32)
        .bind(self.config.lockout_duration_secs().to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error recording failed login".into()))?;

        let lockout_until: Option<DateTime<Utc>> = row.get("lockout_until");

        if let Some(until) = lockout_until {
            let now = Utc::now();
            if until > now {
                let remaining = (until - now).num_seconds().max(0) as u64;
                return Ok(Some(remaining));
            }
        }

        Ok(None)
    }

    /// Reset failed login attempts after successful login.
    pub(super) async fn reset_failed_logins(
        &self,
        user_id: Uuid,
        ip: Option<&str>,
    ) -> AuthResult<()> {
        // Log the successful attempt
        sqlx::query(
            r#"
            INSERT INTO auth.login_attempts (user_id, ip_address, success)
            VALUES ($1, $2::inet, TRUE)
            "#,
        )
        .bind(user_id.into_inner())
        .bind(ip)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error logging login attempt".into()))?;

        // Reset the counters
        sqlx::query(
            r#"
            UPDATE auth.users
            SET failed_login_count = 0, lockout_until = NULL
            WHERE id = $1
            "#,
        )
        .bind(user_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error resetting failed logins".into()))?;

        Ok(())
    }
}
