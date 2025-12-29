use axum::response::Response;
use uuid::Uuid;
use axum::{routing::{get, delete}, Router};
use axum::extract::{Query, State,Path};
use axum::Json;
use axum::http::{header};
use axum::body::Body;
use axum::http::StatusCode;
use crate::{state::AppState,models::{Document, DocumentVersion}, dtos::{ListDocumentsQuery, ListDocumentsResponse, DocumentWithLatest, DownloadQuery}, error::AppError};
use tracing::{info, debug, warn};

use crate::auth::{CurrentUser, check_permission, StorageAction};

use crate::audit::{log_delete,log_download};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/documents", get(list_documents))
        .route("/documents/:id/content", get(download_document))
        .route("/documents/:id", delete(soft_delete_document))
        .route("/documents/:id/hard", delete(hard_delete_document))
}

#[utoipa::path(
    get,
    path = "/documents/{id}/content",
    tag = "documents",
    params(
        ("id" = Uuid, Path, description = "Document ID"),
        ("version" = Option<i32>, Query, description = "Version number (optional, defaults to latest)")
    ),
    responses(
        (status = 200, description = "File content", content_type = "application/octet-stream"),
        (status = 404, description = "Document not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
async fn download_document(
    State(state) : State<AppState>,
    Path(document_id) : Path<Uuid>, 
    Query(query) : Query<DownloadQuery>,
    current_user: CurrentUser,
) -> Result<Response,AppError> {

    info!(user_id = %current_user.id, username = %current_user.username, role = %current_user.role, "File download request received");
    
    // Check if user has read permission
    check_permission(&current_user, StorageAction::Read)?;

    // Check if document exists and is not soft-deleted
    let document = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, title, category, deleted_at, created_at, updated_at
        FROM documents
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(document_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::Db)?;

    if document.is_none() {
        return Err(AppError::NotFound("Document not found or has been deleted"));
    }

    let version_number: i32 = if let Some(v) = query.version {
        v
    } else {
        let latest: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT MAX(version_number)
            FROM document_versions
            WHERE document_id = $1
            "#,
        )
        .bind(document_id)
        .fetch_one(&state.pool)
        .await
        .map_err(AppError::Db)?;

        let Some(v) = latest else {
            return Err(AppError::NotFound("no versions found for this document"));
        };
        v
    };

    let dv = sqlx::query_as::<_, DocumentVersion>(
        r#"
        SELECT
            id,
            document_id,
            version_number,
            file_name,
            file_path,
            file_size,
            mime_type,
            checksum,
            created_at
        FROM document_versions
        WHERE document_id = $1 AND version_number = $2
        "#,
    )
    .bind(document_id)
    .bind(version_number)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::Db)?;

    let dv = match dv {
        Some(v) => v,
        None => return Err(AppError::NotFound("document version not found")),
    };

    // OpenDAL's `read` returns a Buffer; convert it to Vec<u8> for the HTTP body.
    let data = state
        .storage
        .read(&dv.file_path)
        .await?
        .to_vec();

    let content_type = dv
        .mime_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    if let Err(e) = log_download(
        &state.pool,
        current_user.id.to_string(),
        document_id,
        Some(version_number),
    )
    .await
    {
        // Log the error but don't fail the download operation
        // Audit logging should not break the main functionality
        warn!(
            error = ?e,
            document_id = %document_id,
            user_id = %current_user.id,
            version_number = version_number,
            "Failed to create audit log for download"
        );
    }

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        // tell browser / Postman to treat it as a download; you can adjust the filename
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", dv.file_name),
        )
        .body(Body::from(data))
        .map_err(|_| AppError::Other(anyhow::anyhow!("failed to build response")))?;

    Ok(response)

}

/// Soft delete: Mark document as deleted (set deleted_at timestamp)
/// Document and its data remain in database but are hidden from users
#[utoipa::path(
    delete,
    path = "/documents/{id}",
    tag = "documents",
    params(
        ("id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "Document soft deleted successfully"),
        (status = 404, description = "Document not found"),
        (status = 400, description = "Document already deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn soft_delete_document(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(document_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check delete permission (admin only)
    check_permission(&current_user, StorageAction::Delete)?;

    // Verify document exists and is not already soft-deleted
    let document = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, title, category, deleted_at, created_at, updated_at
        FROM documents
        WHERE id = $1
        "#,
    )
    .bind(document_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::Db)?;

    let doc = match document {
        Some(d) => {
            if d.deleted_at.is_some() {
                warn!(document_id = %document_id, "Document already soft-deleted");
                return Err(AppError::BadRequest("Document is already deleted"));
            }
            info!(
                user_id = %current_user.id,
                username = %current_user.username,
                document_id = %document_id,
                title = %d.title,
                "Soft deleting document"
            );
            d
        }
        None => {
            warn!(document_id = %document_id, "Document not found for soft deletion");
            return Err(AppError::NotFound("Document not found"));
        }
    };

    // Update deleted_at timestamp
    let rows_affected = sqlx::query(
        r#"
        UPDATE documents
        SET deleted_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(document_id)
    .execute(&state.pool)
    .await
    .map_err(AppError::Db)?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::BadRequest("Document is already deleted or not found"));
    }

    if let Err(e) = log_delete(
        &state.pool,
        current_user.id.to_string(),
        document_id,
        Some(serde_json::json!({
            "delete_type": "soft",
            "title": &doc.title,
            "category": &doc.category,
        })),
    )
    .await
    {
        // Log the error but don't fail the delete operation
        // Audit logging should not break the main functionality
        warn!(
            error = ?e,
            document_id = %document_id,
            user_id = %current_user.id,
            "Failed to create audit log for soft delete"
        );
    }

    info!(
        user_id = %current_user.id,
        document_id = %document_id,
        "Document soft-deleted successfully"
    );

    Ok(Json(serde_json::json!({
        "message": "Document soft-deleted successfully",
        "document_id": document_id,
        "deleted_at": chrono::Utc::now(),
    })))
}

/// Hard delete: Permanently delete document, all versions, metadata, folder links, and files from storage
#[utoipa::path(
    delete,
    path = "/documents/{id}/hard",
    tag = "documents",
    params(
        ("id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "Document permanently deleted successfully"),
        (status = 404, description = "Document not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn hard_delete_document(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(document_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check delete permission (admin only)
    check_permission(&current_user, StorageAction::Delete)?;

    // Verify document exists
    let document = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, title, category, deleted_at, created_at, updated_at
        FROM documents
        WHERE id = $1
        "#,
    )
    .bind(document_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::Db)?;

    let doc = match document {
        Some(d) => {
            info!(
                user_id = %current_user.id,
                username = %current_user.username,
                document_id = %document_id,
                title = %d.title,
                "Hard deleting document"
            );
            d
        }
        None => {
            warn!(document_id = %document_id, "Document not found for hard deletion");
            return Err(AppError::NotFound("Document not found"));
        }
    };

    // Get all versions for this document (to delete files from OpenDAL)
    let versions = sqlx::query_as::<_, DocumentVersion>(
        r#"
        SELECT id, document_id, version_number, file_name, file_path, file_size, mime_type, checksum, created_at
        FROM document_versions
        WHERE document_id = $1
        "#,
    )
    .bind(document_id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::Db)?;

    // Delete all files from OpenDAL storage
    for version in &versions {
        debug!(
            document_id = %document_id,
            version_number = version.version_number,
            file_path = %version.file_path,
            "Deleting file from storage"
        );
        
        // Delete from OpenDAL
        // Note: If file doesn't exist, OpenDAL might return an error.
        // We log a warning but continue deletion.
        if let Err(e) = state.storage.delete(&version.file_path).await {
            warn!(
                error = ?e,
                file_path = %version.file_path,
                "Failed to delete file from storage (continuing anyway)"
            );
            // Continue deletion even if file deletion fails
        }
    }

    if let Err(e) = log_delete(
        &state.pool,
        current_user.id.to_string(),
        document_id,
        Some(serde_json::json!({
            "delete_type": "hard",
            "title": &doc.title,
            "category": &doc.category,
            "versions_deleted": versions.len(),
            "files_deleted": versions.len(),
        })),
    )
    .await
    {
        // Log the error but don't fail the delete operation
        // Audit logging should not break the main functionality
        warn!(
            error = ?e,
            document_id = %document_id,
            user_id = %current_user.id,
            "Failed to create audit log for hard delete"
        );
    }

    // Delete the document from database
    // This will CASCADE delete:
    //   - document_versions (ON DELETE CASCADE)
    //   - document_metadata (ON DELETE CASCADE)
    //   - document_folders (ON DELETE CASCADE)
    let rows_affected = sqlx::query(
        r#"
        DELETE FROM documents
        WHERE id = $1
        "#,
    )
    .bind(document_id)
    .execute(&state.pool)
    .await
    .map_err(AppError::Db)?
    .rows_affected();

    if rows_affected == 0 {
        // This shouldn't happen since we checked above, but just in case
        return Err(AppError::NotFound("Document not found"));
    }

    info!(
        user_id = %current_user.id,
        document_id = %document_id,
        versions_deleted = versions.len(),
        "Document hard-deleted successfully"
    );

    Ok(Json(serde_json::json!({
        "message": "Document hard-deleted successfully",
        "document_id": document_id,
        "versions_deleted": versions.len(),
    })))
}

#[utoipa::path(
    get,
    path = "/documents",
    tag = "documents",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<u32>, Query, description = "Page size (default: 20, max: 100)"),
        ("title" = Option<String>, Query, description = "Filter by title (partial match)"),
        ("category" = Option<String>, Query, description = "Filter by category (exact match)")
    ),
    responses(
        (status = 200, description = "List of documents", body = ListDocumentsResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
async fn list_documents(
    State(state): State<AppState>,
    Query(params): Query<ListDocumentsQuery>,
) -> Result<Json<ListDocumentsResponse>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) as i64 * page_size as i64;

    // Filters
    let title_filter = params.title.unwrap_or_default();
    let category_filter = params.category;

    debug!(
        page = page,
        page_size = page_size,
        title_filter = %title_filter,
        category_filter = ?category_filter,
        "Listing documents"
    );

    // Count total (exclude soft-deleted documents)
    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM documents d
        WHERE d.deleted_at IS NULL
          AND ($1 = '' OR d.title ILIKE '%' || $1 || '%')
          AND ($2::text IS NULL OR d.category = $2)
        "#
    )
    .bind(&title_filter)
    .bind(&category_filter)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::Db)?;

    // Fetch page with latest version
    let rows = sqlx::query_as::<_, DocumentWithLatest>(
        r#"
        WITH latest_versions AS (
            SELECT DISTINCT ON (document_id)
                document_id,
                version_number,
                file_name,
                file_size,
                mime_type,
                created_at
            FROM document_versions
            ORDER BY document_id, version_number DESC
        )
        SELECT
            d.id,
            d.title,
            d.category,
            d.created_at,
            d.updated_at,
            lv.version_number AS latest_version_number,
            lv.file_name AS latest_file_name,
            lv.file_size AS latest_file_size,
            lv.mime_type AS latest_mime_type,
            lv.created_at AS latest_created_at
        FROM documents d
        LEFT JOIN latest_versions lv ON lv.document_id = d.id
        WHERE d.deleted_at IS NULL
          AND ($1 = '' OR d.title ILIKE '%' || $1 || '%')
          AND ($2::text IS NULL OR d.category = $2)
        ORDER BY d.created_at DESC
        LIMIT $3 OFFSET $4
        "#
    )
    .bind(&title_filter)
    .bind(&category_filter)
    .bind(page_size as i64)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::Db)?;

    let resp = ListDocumentsResponse {
        data: rows,
        page,
        page_size,
        total: total.0,
    };

    info!(
        total = total.0,
        returned = resp.data.len(),
        page = page,
        page_size = page_size,
        "Documents retrieved successfully"
    );

    Ok(Json(resp))
}