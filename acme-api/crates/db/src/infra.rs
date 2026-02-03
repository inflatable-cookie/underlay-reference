//! Infrastructure utilities including email capture.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DbPool;

// ============================================================================
// Captured Emails (Development)
// ============================================================================

/// Raw DB representation of a captured email.
#[derive(Debug, Clone, FromRow)]
pub struct CapturedEmailRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub to_addresses: Vec<String>,
    pub from_address: String,
    pub reply_to: Option<String>,
    pub cc_addresses: Vec<String>,
    pub bcc_addresses: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub headers_json: Option<serde_json::Value>,
    pub captured_at: DateTime<Utc>,
    pub was_delivered: bool,
    pub delivery_error: Option<String>,
}

/// Insert a new captured email.
#[allow(clippy::too_many_arguments)]
pub async fn insert_captured_email(
    pool: &DbPool,
    email_id: Uuid,
    to_addresses: &[String],
    from_address: &str,
    reply_to: Option<&str>,
    cc_addresses: &[String],
    bcc_addresses: &[String],
    subject: &str,
    text_body: Option<&str>,
    html_body: Option<&str>,
    headers_json: Option<serde_json::Value>,
    was_delivered: bool,
    delivery_error: Option<&str>,
) -> Result<CapturedEmailRow, sqlx::Error> {
    sqlx::query_as::<_, CapturedEmailRow>(
        r#"
        INSERT INTO platform.captured_email (
            email_id,
            to_addresses,
            from_address,
            reply_to,
            cc_addresses,
            bcc_addresses,
            subject,
            text_body,
            html_body,
            headers_json,
            was_delivered,
            delivery_error
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING
            id,
            email_id,
            to_addresses,
            from_address,
            reply_to,
            cc_addresses,
            bcc_addresses,
            subject,
            text_body,
            html_body,
            headers_json,
            captured_at,
            was_delivered,
            delivery_error
        "#,
    )
    .bind(email_id)
    .bind(to_addresses)
    .bind(from_address)
    .bind(reply_to)
    .bind(cc_addresses)
    .bind(bcc_addresses)
    .bind(subject)
    .bind(text_body)
    .bind(html_body)
    .bind(headers_json)
    .bind(was_delivered)
    .bind(delivery_error)
    .fetch_one(pool)
    .await
}

/// Filters for listing captured emails.
pub struct CapturedEmailFilters {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub to_address: Option<String>,
    pub from_address: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl Default for CapturedEmailFilters {
    fn default() -> Self {
        Self {
            since: None,
            until: None,
            to_address: None,
            from_address: None,
            limit: 50,
            offset: 0,
        }
    }
}

/// List captured emails with filtering and pagination (newest first).
pub async fn list_captured_emails(
    pool: &DbPool,
    filters: CapturedEmailFilters,
) -> Result<Vec<CapturedEmailRow>, sqlx::Error> {
    let mut query = String::from(
        r#"
        SELECT
            id,
            email_id,
            to_addresses,
            from_address,
            reply_to,
            cc_addresses,
            bcc_addresses,
            subject,
            text_body,
            html_body,
            headers_json,
            captured_at,
            was_delivered,
            delivery_error
        FROM platform.captured_email
        "#,
    );

    let mut clauses: Vec<&str> = Vec::new();

    if filters.since.is_some() {
        clauses.push("captured_at >= $1");
    }
    if filters.until.is_some() {
        clauses.push("captured_at <= $2");
    }
    if filters.to_address.is_some() {
        clauses.push("$3 = ANY(to_addresses)");
    }
    if filters.from_address.is_some() {
        clauses.push("from_address = $4");
    }

    if !clauses.is_empty() {
        query.push_str("WHERE ");
        query.push_str(&clauses.join(" AND "));
    }

    query.push_str(" ORDER BY captured_at DESC LIMIT $5 OFFSET $6");

    sqlx::query_as::<_, CapturedEmailRow>(&query)
        .bind(filters.since)
        .bind(filters.until)
        .bind(filters.to_address)
        .bind(filters.from_address)
        .bind(filters.limit)
        .bind(filters.offset)
        .fetch_all(pool)
        .await
}

/// Get a single captured email by ID.
pub async fn get_captured_email_by_id(
    pool: &DbPool,
    id: Uuid,
) -> Result<Option<CapturedEmailRow>, sqlx::Error> {
    sqlx::query_as::<_, CapturedEmailRow>(
        r#"
        SELECT
            id,
            email_id,
            to_addresses,
            from_address,
            reply_to,
            cc_addresses,
            bcc_addresses,
            subject,
            text_body,
            html_body,
            headers_json,
            captured_at,
            was_delivered,
            delivery_error
        FROM platform.captured_email
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Delete a captured email by ID.
pub async fn delete_captured_email(pool: &DbPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM platform.captured_email WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Delete captured emails older than a certain date.
pub async fn delete_captured_emails_before(
    pool: &DbPool,
    before: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM platform.captured_email WHERE captured_at < $1")
        .bind(before)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

// ============================================================================
// EmailStore Implementation for DevCaptureAdapter
// ============================================================================

use underlay_email::{CapturedEmail, EmailResult, EmailStore};

/// Database-backed email store for the DevCaptureAdapter.
///
/// This implements the `EmailStore` trait from underlay-email,
/// storing captured emails in the `platform.captured_email` table.
#[derive(Clone)]
pub struct DbEmailStore {
    pool: DbPool,
}

impl DbEmailStore {
    /// Create a new database-backed email store.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EmailStore for DbEmailStore {
    async fn store(&self, email: CapturedEmail) -> EmailResult<()> {
        let headers_json = email
            .headers_json
            .as_ref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());

        insert_captured_email(
            &self.pool,
            email.email_id,
            &email.to_addresses,
            &email.from_address,
            email.reply_to.as_deref(),
            &email.cc_addresses,
            &email.bcc_addresses,
            &email.subject,
            email.text_body.as_deref(),
            email.html_body.as_deref(),
            headers_json,
            email.was_delivered,
            email.delivery_error.as_deref(),
        )
        .await
        .map_err(|e| underlay_email::EmailError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
