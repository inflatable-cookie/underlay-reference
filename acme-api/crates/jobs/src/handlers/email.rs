use async_trait::async_trait;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;
use underlay_jobs::{Job, JobConfig, JobHandler, JobHandlerError};

// ============================================================================
// Job Handler: email.welcome
// ============================================================================

/// Send a welcome email to a newly registered user.
///
/// Payload: `{ "user_id": "uuid", "email": "user@example.com", "display_name": "John" }`
pub struct WelcomeEmailHandler {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl WelcomeEmailHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize)]
struct WelcomeEmailPayload {
    user_id: uuid::Uuid,
    email: String,
    display_name: Option<String>,
}

#[async_trait]
impl JobHandler for WelcomeEmailHandler {
    fn job_type(&self) -> &'static str {
        "email.welcome"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            max_attempts: 5,
            ..Default::default()
        }
    }

    async fn handle(&self, job: Job) -> Result<(), JobHandlerError> {
        let payload: WelcomeEmailPayload = serde_json::from_value(job.payload)
            .map_err(|e| JobHandlerError::permanent(format!("invalid payload: {}", e)))?;

        // In a real app, send email via email service (underlay-email)
        // For the reference implementation, we just log it
        info!(
            job_id = %job.id,
            user_id = %payload.user_id,
            email = %payload.email,
            display_name = ?payload.display_name,
            "would send welcome email"
        );

        Ok(())
    }
}
