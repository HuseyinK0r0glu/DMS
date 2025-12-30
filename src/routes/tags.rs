use crate::error::AppError;
use crate::auth::{CurrentUser, check_permission, StorageAction};
use crate::state::AppState;
use crate::dtos::{AddTagToDocumentRequest, TagInfo, AddTagToDocumentResponse};
use tracing::{info, warn, debug};
use axum::{routing::post, Router, extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow;
use crate::models::Tag; 
use sqlx;

pub fn routes() -> Router<AppState> {
    Router::new().route("/tags", post(add_tags_to_document))
}

#[utoipa::path(
    post,
    path = "/tags",
    tag = "tags",
    request_body = AddTagToDocumentRequest,
    responses(
        (status = 200, description = "Tags added to document successfully", body = AddTagToDocumentResponse),
        (status = 400, description = "Bad request - invalid document_id or empty tags"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - write permission required")
    ),
    security(("api_key" = []))
)]
pub async fn add_tags_to_document(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(request): Json<AddTagToDocumentRequest>,
) -> Result<Json<AddTagToDocumentResponse>, AppError> {

    check_permission(&current_user, StorageAction::Write)?;

    if request.tags.is_empty() {
        return Err(AppError::BadRequest("Tags list cannot be empty"));
    }

    let document_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND deleted_at IS NULL)"
    )
    .bind(request.document_id)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::Db)?;

    if !document_exists {
        return Err(AppError::NotFound("Document not found or has been deleted"));
    }

    let mut tag_infos = Vec::new();

    for tag_name in &request.tags {
        let tag_name = tag_name.trim();
        
        if tag_name.is_empty() {
            continue; // Skip empty tag names
        }
        
        let (tag, tag_was_created): (Tag, bool) = match sqlx::query_as::<_, Tag>(
            "SELECT id, name, created_at FROM tags WHERE name = $1"
        )
        .bind(tag_name)
        .fetch_optional(&state.pool)
        .await
        .map_err(AppError::Db)?
        {
            Some(existing_tag) => {
                (existing_tag, false)
            }
            None => {
                let new_tag = sqlx::query_as::<_, Tag>(
                    "INSERT INTO tags (name) VALUES ($1) RETURNING id, name, created_at"
                )
                .bind(tag_name)
                .fetch_one(&state.pool)
                .await
                .map_err(AppError::Db)?;
                (new_tag, true)
            }
        };
        
        let relationship_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM document_tags WHERE document_id = $1 AND tag_id = $2)"
        )
        .bind(request.document_id)
        .bind(tag.id)
        .fetch_one(&state.pool)
        .await
        .map_err(AppError::Db)?;
        
        if !relationship_exists {
            sqlx::query(
                "INSERT INTO document_tags (document_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(request.document_id)
            .bind(tag.id)
            .execute(&state.pool)
            .await
            .map_err(AppError::Db)?;
        }
        
        tag_infos.push(TagInfo {
            tag_id: tag.id,
            tag_name: tag.name,
            tag_created: tag_was_created,
        });
    }

    if tag_infos.is_empty() {
        return Err(AppError::BadRequest("No valid tags were processed"));
    }

    let total = tag_infos.len();
    let response = AddTagToDocumentResponse {
        document_id: request.document_id,
        tags: tag_infos,
        total,
    };

    info!(
        document_id = %request.document_id,
        tags_count = response.total,
        user_id = %current_user.id,
        "Tags added to document successfully"
    );

    Ok(Json(response))

}