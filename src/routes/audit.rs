use crate::models::{AuditLog};
use crate::error::AppError;
use crate::{state::AppState,dtos::AuditResponse};
use crate::auth::{CurrentUser, check_permission, StorageAction};
use tracing::{info, warn, error,debug};
use axum::{routing::get, Router};
use axum::extract::{State};
use axum::Json;

pub fn routes() -> Router<AppState> {
    Router::new().route("/audit", get(get_actions))
}

async fn get_actions(
    State(state): State<AppState>,
    current_user: CurrentUser,
)->Result<Json<AuditResponse>, AppError>{
    info!(user_id = %current_user.id, username = %current_user.username, role = %current_user.role, "Get actions request received");

    check_permission(&current_user, StorageAction::GetActions)?;

    let audit_logs = sqlx::query_as::<_, AuditLog>(
        r#"
        SELECT id, user_id, action, document_id, document_version, metadata, created_at
        FROM audit_logs
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.pool)  
    .await
    .map_err(AppError::Db)?;

    let total = audit_logs.len() as i64;

    let response = AuditResponse {
        data: audit_logs,
        total,
    };

    Ok(Json(response))

}
