//! PostgreSQL COPY writer for high-performance batch inserts
//!
//! Uses sqlx's PgCopyIn API for efficient bulk insertion of operation_logs.
//! COPY is 5-10x faster than INSERT for batch operations.
//!
//! ## Key Features
//! - Reuses SeaORM's sqlx connection pool (no separate pool needed)
//! - Uses CSV format for proper bytea hex encoding support
//! - Proper escaping for special characters
//! - Handles nullable fields (storage_id, params)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use sqlx::postgres::PgPoolCopyExt;
use tracing::{debug, error};
use uuid::Uuid;

/// Operation data for COPY insertion
#[derive(Debug, Clone)]
pub struct CopyOperationData {
    pub doc_id: Uuid,
    pub rev: i64,
    pub user_id: String,
    pub mutation_id: String,
    pub storage_id: Option<Uuid>,
    pub params: Option<Vec<u8>>,
    pub op_id: String,
    pub created_at: DateTime<Utc>,
}

/// Version update data for atomic COPY + UPDATE
#[derive(Debug, Clone)]
pub struct VersionUpdate {
    pub doc_id: Uuid,
    pub max_rev: i64,
}

/// Execute batch insert using PostgreSQL COPY (CSV format)
///
/// This is significantly faster than INSERT for large batches:
/// - INSERT: ~50-100ms for 100 rows
/// - COPY: ~5-10ms for 100 rows
///
/// # Arguments
/// * `db` - SeaORM database connection (sqlx pool will be extracted)
/// * `ops` - Operations to insert
///
/// # Returns
/// Number of rows inserted
pub async fn copy_operations(
    db: &DatabaseConnection,
    ops: &[CopyOperationData],
) -> Result<u64> {
    if ops.is_empty() {
        return Ok(0);
    }

    let pool = db.get_postgres_connection_pool();
    let mut copy_in = pool
        .copy_in_raw(COPY_STATEMENT)
        .await
        .context("Failed to start COPY")?;

    let buffer = build_csv_buffer(ops);

    copy_in
        .send(buffer.as_bytes())
        .await
        .context("Failed to send COPY data")?;

    let rows = match copy_in.finish().await {
        Ok(r) => r,
        Err(e) => {
            error!("COPY finish failed: {:?}", e);
            return Err(anyhow::anyhow!("COPY finish failed: {}", e));
        }
    };

    debug!("COPY inserted {} rows", rows);
    Ok(rows)
}

/// Execute batch insert + version update atomically in a single PostgreSQL transaction
///
/// Wraps COPY of operation_logs and UPDATE of documents.current_version
/// in a single transaction, eliminating the consistency window between the two.
/// If either operation fails, both are rolled back.
pub async fn copy_operations_atomic(
    db: &DatabaseConnection,
    ops: &[CopyOperationData],
    version_updates: &[VersionUpdate],
) -> Result<u64> {
    if ops.is_empty() {
        return Ok(0);
    }

    let pool = db.get_postgres_connection_pool();
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // COPY within transaction — borrow tx mutably via DerefMut to PgConnection
    let mut copy_in = (&mut *tx)
        .copy_in_raw(COPY_STATEMENT)
        .await
        .context("Failed to start COPY in transaction")?;

    let buffer = build_csv_buffer(ops);

    copy_in
        .send(buffer.as_bytes())
        .await
        .context("Failed to send COPY data")?;

    let rows = copy_in.finish().await.map_err(|e| {
        error!("COPY finish failed in transaction: {:?}", e);
        anyhow::anyhow!("COPY finish failed: {}", e)
    })?;
    // copy_in is consumed here, releasing the mutable borrow on tx

    // UPDATE document versions within the same transaction
    for update in version_updates {
        sqlx::query(
            "UPDATE documents SET current_version = $1, updated_at = NOW() \
             WHERE id = $2 AND current_version < $1",
        )
        .bind(update.max_rev)
        .bind(update.doc_id)
        .execute(&mut *tx)
        .await
        .context(format!(
            "Failed to update version for doc {}",
            update.doc_id
        ))?;
    }

    // Atomic commit — both COPY and UPDATEs succeed or fail together
    tx.commit().await.context("Transaction commit failed")?;

    debug!(
        "Atomic COPY + UPDATE: {} rows inserted, {} versions updated",
        rows,
        version_updates.len()
    );
    Ok(rows)
}

const COPY_STATEMENT: &str = "COPY operation_logs (doc_id, rev, user_id, mutation_id, storage_id, params, op_id, created_at) \
    FROM STDIN WITH (FORMAT csv)";

/// Build CSV buffer for COPY data
fn build_csv_buffer(ops: &[CopyOperationData]) -> String {
    let mut buffer = String::with_capacity(ops.len() * 256);

    for op in ops {
        // doc_id (UUID)
        buffer.push_str(&op.doc_id.to_string());
        buffer.push(',');

        // rev (i64)
        buffer.push_str(&op.rev.to_string());
        buffer.push(',');

        // user_id (quoted string)
        write_csv_string(&op.user_id, &mut buffer);
        buffer.push(',');

        // mutation_id (quoted string)
        write_csv_string(&op.mutation_id, &mut buffer);
        buffer.push(',');

        // storage_id (nullable UUID, empty for NULL in CSV)
        match &op.storage_id {
            Some(id) => buffer.push_str(&id.to_string()),
            None => {} // Empty field = NULL in CSV
        }
        buffer.push(',');

        // params (nullable bytea as hex)
        match &op.params {
            Some(p) => {
                // PostgreSQL bytea hex format: \x followed by hex digits
                // In CSV, we need to quote this since it contains special chars
                buffer.push('"');
                buffer.push_str("\\x");
                for byte in p {
                    buffer.push_str(&format!("{:02x}", byte));
                }
                buffer.push('"');
            }
            None => {} // Empty field = NULL in CSV
        }
        buffer.push(',');

        // op_id (quoted string)
        write_csv_string(&op.op_id, &mut buffer);
        buffer.push(',');

        // created_at (ISO 8601 timestamp with timezone)
        buffer.push_str(&op.created_at.to_rfc3339());
        buffer.push('\n');
    }

    buffer
}

/// Write a string value in CSV format (quoted, with internal quotes doubled)
fn write_csv_string(s: &str, buffer: &mut String) {
    buffer.push('"');
    for c in s.chars() {
        if c == '"' {
            buffer.push_str("\"\""); // Double quotes for escaping
        } else {
            buffer.push(c);
        }
    }
    buffer.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_csv_string() {
        let mut buffer = String::new();
        write_csv_string("hello world", &mut buffer);
        assert_eq!(buffer, "\"hello world\"");

        buffer.clear();
        write_csv_string("say \"hello\"", &mut buffer);
        assert_eq!(buffer, "\"say \"\"hello\"\"\"");

        buffer.clear();
        write_csv_string("line1\nline2", &mut buffer);
        assert_eq!(buffer, "\"line1\nline2\"");
    }
}
