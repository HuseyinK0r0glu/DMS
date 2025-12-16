use axum::{routing::get, Router};
use axum::extract::{Query, State};
use axum::Json;
use crate::{state::AppState, dtos::{ListDocumentsQuery, ListDocumentsResponse, DocumentWithLatest}, error::AppError};

pub fn routes() -> Router<AppState> {
    Router::new().route("/documents", get(list_documents))
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
    Ok(Json(resp))
}