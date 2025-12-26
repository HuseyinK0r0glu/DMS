use sqlx::PgPool;
use crate::models::{AuditLog, NewAuditLog, AuditAction};
use crate::error::AppError;
use uuid::Uuid;
use tracing::{info, warn, error};

pub async fn log_action(
    pool: &PgPool,
    log_entry: NewAuditLog,
) -> Result<AuditLog, AppError> {

    let audit_log = sqlx::query_as::<_, AuditLog>(
    r#"
        INSERT INTO audit_logs (user_id, action, document_id, document_version, metadata)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, action, document_id, document_version, metadata, created_at
        "#
    )
    .bind(&log_entry.user_id)
    .bind(&log_entry.action) 
    .bind(&log_entry.document_id)
    .bind(&log_entry.document_version)
    .bind(&log_entry.metadata)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = ?e, "Failed to insert audit log");
        AppError::Db(e)
    })?;

    info!(
        user_id = %log_entry.user_id,
        action = ?log_entry.action,
        document_id = ?log_entry.document_id,
        "Audit log created"
    );

    Ok(audit_log)
}

pub async fn log_upload(
    pool: &PgPool,
    user_id: String,
    document_id: Uuid,
    document_version: i32,
    metadata: Option<serde_json::Value>,
) -> Result<AuditLog, AppError> {
    log_action(
        pool,
        NewAuditLog {
            user_id,
            action: AuditAction::Upload,
            document_id: Some(document_id),
            document_version: Some(document_version),
            metadata: metadata.unwrap_or_else(|| serde_json::json!({})),
        },
    )
    .await
}

pub async fn log_download(
    pool: &PgPool,
    user_id: String,
    document_id: Uuid,
    document_version: Option<i32>,
) -> Result<AuditLog, AppError> {
    log_action(
        pool,
        NewAuditLog {
            user_id,
            action: AuditAction::Download,
            document_id: Some(document_id),
            document_version,
            metadata: serde_json::json!({}),
        },
    )
    .await
}

pub async fn log_delete(
    pool: &PgPool,
    user_id: String,
    document_id: Uuid,
    metadata: Option<serde_json::Value>,
) -> Result<AuditLog, AppError> {
    log_action(
        pool,
        NewAuditLog {
            user_id,
            action: AuditAction::Delete,
            document_id: Some(document_id),
            document_version: None,
            metadata: metadata.unwrap_or_else(|| serde_json::json!({})),
        },
    )
    .await
}

