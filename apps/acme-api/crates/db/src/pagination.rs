use sqlx::{Postgres, Transaction};

use crate::DbPool;

/// Keep each database fetch bounded while preserving a stable snapshot when a
/// caller needs the complete collection (for example media reconciliation).
pub(crate) const READ_BATCH_SIZE: i64 = 100;

pub(crate) async fn begin_repeatable_read(
    pool: &DbPool,
) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}
