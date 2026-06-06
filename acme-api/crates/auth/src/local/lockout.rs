use super::*;
use acme_db::activity::{log_activity, LogActivityParams};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::Row;
use std::sync::LazyLock;
use tracing::warn;
use underlay_security_alerts::{
    evaluate_alerts, has_recent_alert_in_table, insert_alert_event_into_table,
    load_ip_signal_counts_from_table, LoginAttemptsTable, SecurityAlertEventInput,
    SecurityAlertEventsTable,
};

static LOGIN_ATTEMPTS_TABLE: LazyLock<LoginAttemptsTable> = LazyLock::new(|| {
    LoginAttemptsTable::parse("auth.login_attempts").expect("valid login-attempts table")
});
static SECURITY_ALERT_EVENTS_TABLE: LazyLock<SecurityAlertEventsTable> = LazyLock::new(|| {
    SecurityAlertEventsTable::parse("auth.security_alert_events").expect("valid alerts table")
});

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
                if let Some(ip) = ip {
                    self.emit_security_alerts_for_ip(ip).await;
                }
                return Ok(Some(remaining));
            }
        }

        if let Some(ip) = ip {
            self.emit_security_alerts_for_ip(ip).await;
        }

        Ok(None)
    }

    /// Reset failed login attempts after successful login.
    pub(super) async fn reset_failed_logins(
        &self,
        user_id: Uuid,
        ip: Option<&str>,
    ) -> AuthResult<()> {
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

    pub(super) async fn record_locked_login_attempt(
        &self,
        user_id: Uuid,
        ip: Option<&str>,
    ) -> AuthResult<()> {
        sqlx::query(
            r#"
            INSERT INTO auth.login_attempts (user_id, ip_address, success, failure_reason)
            VALUES ($1, $2::inet, FALSE, 'account_locked')
            "#,
        )
        .bind(user_id.into_inner())
        .bind(ip)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthError::Internal("DB error logging locked login attempt".into()))?;

        if let Some(ip) = ip {
            self.emit_security_alerts_for_ip(ip).await;
        }

        Ok(())
    }

    async fn emit_security_alerts_for_ip(&self, ip: &str) {
        if ip.trim().is_empty() {
            return;
        }

        let now = Utc::now();
        let config = self.config.security_alert_config();
        let since = now - config.window();

        let counts =
            match load_ip_signal_counts_from_table(&self.pool, &LOGIN_ATTEMPTS_TABLE, ip, since)
                .await
            {
                Ok(counts) => counts,
                Err(err) => {
                    warn!(
                        error = %err,
                        ip_address = %ip,
                        "failed to load login-attempt signal counts"
                    );
                    return;
                }
            };

        let alert_types = evaluate_alerts(counts, &config);
        for alert_type in alert_types {
            let already_emitted = match has_recent_alert_in_table(
                &self.pool,
                &SECURITY_ALERT_EVENTS_TABLE,
                alert_type,
                ip,
                config.cooldown(),
                now,
            )
            .await
            {
                Ok(value) => value,
                Err(err) => {
                    warn!(
                        error = %err,
                        ip_address = %ip,
                        alert_type = alert_type.as_str(),
                        "failed to check security alert cooldown"
                    );
                    continue;
                }
            };

            if already_emitted {
                continue;
            }

            let event = SecurityAlertEventInput {
                alert_type,
                ip_address: ip.to_string(),
                window_started_at: since,
                window_ended_at: now,
                counts,
                details: json!({
                    "source": "acme-auth",
                    "window_seconds": config.window().num_seconds(),
                    "cooldown_seconds": config.cooldown().num_seconds(),
                }),
            };

            let event_id = match insert_alert_event_into_table(
                &self.pool,
                &SECURITY_ALERT_EVENTS_TABLE,
                &event,
            )
            .await
            {
                Ok(id) => id,
                Err(err) => {
                    warn!(
                        error = %err,
                        ip_address = %ip,
                        alert_type = alert_type.as_str(),
                        "failed to persist security alert event"
                    );
                    continue;
                }
            };

            warn!(
                alert_event_id = %event_id,
                alert_type = alert_type.as_str(),
                ip_address = %ip,
                failed_attempts = counts.failed_attempts,
                distinct_users = counts.distinct_users,
                lockouts = counts.lockouts,
                "security alert emitted for suspicious login activity"
            );

            if let Err(err) = log_activity(
                &self.pool,
                LogActivityParams {
                    user_id: None,
                    action: "auth.security_alert_emitted",
                    resource_type: "security_alert_event",
                    resource_id: event_id,
                    details: Some(json!({
                        "alert_type": alert_type.as_str(),
                        "ip_address": ip,
                        "failed_attempts": counts.failed_attempts,
                        "distinct_users": counts.distinct_users,
                        "lockouts": counts.lockouts,
                        "window_started_at": since.to_rfc3339(),
                        "window_ended_at": now.to_rfc3339(),
                    })),
                    correlation_id: None,
                    ip_address: Some(ip),
                },
            )
            .await
            {
                warn!(
                    error = %err,
                    alert_event_id = %event_id,
                    "failed to write audit log for security alert event"
                );
            }
        }
    }
}
