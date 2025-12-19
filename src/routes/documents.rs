use axum::response::Response;
use uuid::Uuid;
use axum::{routing::get, Router};
use axum::extract::{Query, State,Path};
use axum::Json;
use axum::http::{header};
use axum::body::Body;
use axum::http::StatusCode;
use crate::{state::AppState,models::{Document, DocumentVersion}, dtos::{ListDocumentsQuery, ListDocumentsResponse, DocumentWithLatest, DownloadQuery}, error::AppError};
use tracing::{info, debug};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/documents", get(list_documents))
        .route("/documents/:id/content", get(download_document))
}

async fn download_document(
    State(state) : State<AppState>,
    Path(document_id) : Path<Uuid>, 
    Query(query) : Query<DownloadQuery>
) -> Result<Response,AppError> {

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

    // Count total
    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM documents d
        WHERE ($1 = '' OR d.title ILIKE '%' || $1 || '%')
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
        WHERE ($1 = '' OR d.title ILIKE '%' || $1 || '%')
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