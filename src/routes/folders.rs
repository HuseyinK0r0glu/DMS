use crate::error::AppError;
use crate::auth::{CurrentUser, check_permission, StorageAction};
use crate::state::AppState;
use crate::dtos::{CreateFolderRequest, CreateFolderResponse};
use tracing::{info, warn, debug};
use axum::{routing::post, Router, extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow;

#[derive(Serialize, Deserialize)]
struct FolderMetadata {
    folder_name: String,
    created_by: Uuid,
    created_by_username: String,
    created_at: DateTime<Utc>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/folders", post(create_folder))
}

#[utoipa::path(
    post,
    path = "/folders",
    tag = "folders",
    request_body = CreateFolderRequest,
    responses(
        (status = 200, description = "Folder created successfully", body = CreateFolderResponse),
        (status = 400, description = "Bad request - folder name invalid or already exists"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - write permission required")
    ),
    security(("api_key" = []))
)]
pub async fn create_folder(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(request): Json<CreateFolderRequest>,
) -> Result<Json<CreateFolderResponse>, AppError> {
    check_permission(&current_user, StorageAction::Write)?;

    let folder_name = request.name.trim();
    if folder_name.is_empty() {
        return Err(AppError::BadRequest("Folder name cannot be empty"));
    }

    // Sanitize folder name
    let sanitized_name = folder_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect::<String>();

    // Check if folder exists by listing it and checking if it has any entries
    let folder_path = format!("{}/", sanitized_name);
    let folder_exists = match state.storage.list(&folder_path).await {
        Ok(entries) => {
            // Folder exists if it has at least one entry
            !entries.is_empty()
        }
        Err(e) => {
            // If list fails with NotFound, folder doesn't exist
            if e.kind() != opendal::ErrorKind::NotFound {
                warn!(error = ?e, folder_name = %sanitized_name, "Error checking folder existence");
            }
            false
        }
    };

    // Also check metadata file
    let metadata_path = format!("{}/.folder_metadata.json", sanitized_name);
    let metadata_exists = state.storage.stat(&metadata_path).await.is_ok();

    if folder_exists || metadata_exists {
        return Err(AppError::BadRequest("Folder already exists"));
    }

    debug!(folder_name = %sanitized_name, "Folder does not exist, proceeding with creation");

    let metadata = FolderMetadata {
        folder_name: sanitized_name.clone(),                                                                                                                                    
        created_by: current_user.id,
        created_by_username: current_user.username.clone(),
        created_at: Utc::now(),
    };

    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| AppError::Other(anyhow::anyhow!("Failed to serialize metadata: {}", e)))?;                      

    let metadata_bytes = metadata_json.into_bytes();
    state
        .storage
        .write(&metadata_path, metadata_bytes)
        .await
        .map_err(|e| {
            warn!(
                error = ?e,
                folder_name = %sanitized_name,
                "Failed to create folder metadata"
            );
            AppError::Storage(e)
        })?;

    info!(
        folder_name = %sanitized_name,
        user_id = %current_user.id,
        username = %current_user.username,
        "Folder created successfully"
    );

    let response = CreateFolderResponse {
        folder_name: sanitized_name,
        created_at: metadata.created_at,
        created_by: metadata.created_by,
    };

    Ok(Json(response))
}