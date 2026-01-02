use crate::auth::{check_permission, CurrentUser, StorageAction};
use crate::dtos::{CreateFolderRequest, CreateFolderResponse, FolderInfo, ListFoldersResponse};
use crate::error::AppError;
use crate::state::AppState;
use anyhow;
use axum::{extract::State, routing::get, routing::post, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct FolderMetadata {
    folder_name: String,
    created_by: Uuid,
    created_by_username: String,
    created_at: DateTime<Utc>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/folders", post(create_folder))
        .route("/folders", get(list_folders))
}

#[utoipa::path(
    get,
    path = "/folders",
    tag = "folders",
    responses(
        (status = 200, description = "List of all folders", body = ListFoldersResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("api_key" = []))
)]
pub async fn list_folders(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<ListFoldersResponse>, AppError> {
    check_permission(&current_user, StorageAction::Read)?;

    info!("Listing all folders");

    let entries = state.storage.list("").await.map_err(|e| {
        warn!(error = ?e, "Failed to list storage entries");
        AppError::Storage(e)
    })?;

    let mut folders = Vec::new();

    for entry in entries {
        let path = entry.path();

        if path.ends_with('/') {
            let folder_name = path.trim_end_matches('/').to_string();

            if folder_name.is_empty() {
                continue;
            }

            let metadata_path = format!("{}/.folder_metadata.json", folder_name);

            match state.storage.read(&metadata_path).await {
                Ok(metadata_bytes) => {
                    match serde_json::from_slice::<FolderMetadata>(&metadata_bytes.to_vec()) {
                        Ok(metadata) => {
                            folders.push(crate::dtos::FolderInfo {
                                folder_name: metadata.folder_name,
                                created_by: metadata.created_by,
                                created_by_username: metadata.created_by_username,
                                created_at: metadata.created_at,
                            });
                        }
                        Err(e) => {
                            warn!(
                                folder = %folder_name,
                                error = ?e,
                                "Failed to parse folder metadata JSON, skipping"
                            );
                            // TODO: Add folder without metadata ??
                        }
                    }
                }
                Err(e) => {
                    // Metadata file doesn't exist - folder was created by upload without metadata
                    warn!(
                        folder = %folder_name,
                        error = ?e,
                        "Folder metadata not found, skipping folder"
                    );
                }
            }
        }
    }

    info!(
        total_folders = folders.len(),
        "Retrieved folders successfully"
    );

    let len = folders.len();

    Ok(Json(crate::dtos::ListFoldersResponse {
        folders,
        total: len,
    }))
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
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
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
