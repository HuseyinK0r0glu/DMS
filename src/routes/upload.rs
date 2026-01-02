use crate::auth::{check_permission, CurrentUser, StorageAction};
use crate::{
    dtos::UploadResponse,
    error::AppError,
    models::{Document, DocumentVersion},
    state::AppState,
};
use axum::extract::{Multipart, State};
use axum::Json;
use axum::{routing::post, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, fs};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::audit::log_upload;

#[derive(Serialize, Deserialize)]
struct FolderMetadata {
    folder_name: String,
    created_by: Uuid,
    created_by_username: String,
    created_at: DateTime<Utc>,
}

fn sanitize_folder_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/upload", post(upload_file))
}

// for using the folders structure in the seaweed
fn build_storage_path_with_folder(
    folder_name: Option<String>,
    document_id: Uuid,
    version_number: i32,
) -> String {
    let folder_name = folder_name
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Uncategorized".to_string());

    format!("{}/{}/v{}", folder_name, document_id, version_number)
}

#[utoipa::path(
    post,
    path = "/upload",
    tag = "upload",
    request_body(content = String, content_type = "multipart/form-data", description = "File upload with title, category, and optional metadata"),
    responses(
        (status = 200, description = "Upload successful", body = UploadResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("api_key" = [])
    )
)]
async fn upload_file(
    State(state): State<AppState>,
    current_user: CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    info!(user_id = %current_user.id, username = %current_user.username, role = %current_user.role, "File upload request received");

    // Check if user has write permission
    check_permission(&current_user, StorageAction::Write)?;

    // Expect form fields:
    // - document_id (optional; if provided, add new version to existing doc)
    // - title (text)
    // - category (optional text)
    // - file (binary)
    // - metadata fields:
    //     * any field starting with "meta_" will be treated as metadata (key after prefix)
    //     * or a JSON object field named "metadata" (stringified JSON) e.g. {"owner":"alice"}
    //   e.g., meta_department=finance -> key=department, value=finance

    let mut document_id: Option<Uuid> = None;
    let mut title_opt: Option<String> = None;
    let mut category: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut mime_type: Option<String> = None;
    let mut metadata: HashMap<String, String> = HashMap::new();
    let mut metadata_keys: Vec<String> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "document_id" => {
                if let Ok(text) = field.text().await {
                    match Uuid::parse_str(text.trim()) {
                        Ok(id) => document_id = Some(id),
                        Err(_) => {
                            return Err(AppError::BadRequest("Invalid document_id (must be UUID)"));
                        }
                    }
                }
            }
            "title" => {
                title_opt = field.text().await.ok();
            }
            "category" => {
                category = field.text().await.ok();
            }
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                mime_type = field.content_type().map(|s| s.to_string());
                if let Ok(bytes) = field.bytes().await {
                    file_bytes = Some(bytes.to_vec());
                }
            }
            "metadata" => {
                if let Ok(text) = field.text().await {
                    match serde_json::from_str::<Value>(&text) {
                        Ok(Value::Object(map)) => {
                            for (k, v) in map {
                                if let Some(val) = v.as_str() {
                                    if !k.is_empty() {
                                        metadata.insert(k.clone(), val.to_string());
                                        metadata_keys.push(k);
                                    }
                                }
                            }
                        }
                        _ => {
                            return Err(AppError::BadRequest(
                                "Invalid metadata JSON; expected an object of string values",
                            ));
                        }
                    }
                }
            }
            name if name.starts_with("meta_") => {
                if let Ok(val) = field.text().await {
                    let key = name.trim_start_matches("meta_").to_string();
                    if !key.is_empty() {
                        metadata.insert(key.clone(), val);
                        metadata_keys.push(key);
                    }
                }
            }
            _ => {}
        }
    }

    let file_bytes = match file_bytes {
        Some(b) => b,
        None => {
            warn!("File upload request missing file field");
            return Err(AppError::BadRequest("Missing file"));
        }
    };
    let file_name = file_name.unwrap_or_else(|| "upload.bin".to_string());

    // NOTE ABOUT STORAGE KEY STRATEGIES
    //
    // Old approach (random UUID file name), kept for reference:
    //
    // let stored_file_name = format!("{}_{}", Uuid::new_v4(), file_name);
    //
    // // Old direct filesystem approach:
    // let stored_path = state.upload_dir.join(&stored_file_name);
    // info!(
    //     file_name = %file_name,
    //     file_size = file_bytes.len(),
    //     stored_path = %stored_path.display(),
    //     "Saving file to disk"
    // );
    // if let Err(err) = fs::write(&stored_path, &file_bytes) {
    //     warn!(error = ?err, "Failed to write file to disk");
    //     return Err(AppError::Io(err));
    // }
    //
    // // New OpenDAL approach (before this change) wrote the file BEFORE we knew
    // // the document_id and version_number:
    // info!(
    //     file_name = %file_name,
    //     file_size = file_bytes.len(),
    //     stored_key = %stored_file_name,
    //     "Saving file via OpenDAL"
    // );
    // state
    //     .storage
    //     .write(&stored_file_name, file_bytes.clone())
    //     .await?;
    // let stored_path = stored_file_name.clone();
    //
    // NEW APPROACH:
    // ---------------------------------
    // We want storage keys like:
    //   {document_id}/v{version_number}
    // e.g.:
    //   47cc9638-9751-469e-943b-d8821ef8f00c/v2
    //
    // To do that we must FIRST know document.id and next_version_number.
    // So we delay the OpenDAL write until AFTER we decide whether we are
    // creating a new document or appending a new version.

    let file_size = file_bytes.len() as i64;
    let checksum = None::<String>;

    debug!("Starting database transaction");
    let mut tx = state.pool.begin().await?;

    // Create new document or append to existing
    let (document, next_version_number) = if let Some(doc_id) = document_id {
        debug!(document_id = %doc_id, "Adding new version to existing document");
        // Existing document: ensure it exists
        let doc_opt = sqlx::query_as::<_, Document>(
            r#"
            SELECT id, title, category, created_at, updated_at,deleted_at
            FROM documents
            WHERE id = $1
            "#,
        )
        .bind(doc_id)
        .fetch_optional(&mut *tx)
        .await?;

        let doc = match doc_opt {
            Some(d) => d,
            None => {
                warn!(document_id = %doc_id, "Document not found for version upload");
                return Err(AppError::BadRequest("document_id not found"));
            }
        };

        // Next version number
        let next_version_opt: Option<i32> = sqlx::query_scalar::<_, Option<i32>>(
            r#"
                SELECT MAX(version_number) + 1
                FROM document_versions
                WHERE document_id = $1
                "#,
        )
        .bind(doc_id)
        .fetch_one(&mut *tx)
        .await?; // DB error -> AppError::Db

        let next_version = next_version_opt.unwrap_or(1);

        (doc, next_version)
    } else {
        // New document: require title
        let title = match title_opt {
            Some(t) if !t.is_empty() => {
                debug!(title = %t, category = ?category, "Creating new document");
                t
            }
            _ => {
                warn!("File upload request missing title field");
                return Err(AppError::BadRequest("Missing title"));
            }
        };

        let doc = sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (title, category)
            VALUES ($1, $2)
            RETURNING id, title, category, created_at, updated_at,deleted_at
            "#,
        )
        .bind(&title)
        .bind(&category)
        .fetch_one(&mut *tx)
        .await?;
        (doc, 1)
    };

    //  FOLDER HIERARCHY MAPPING
    // We map the document's category to a folder as follows:
    // - "Finance" (any case)  -> Finance folder
    // - "Report" / "Reports"  -> Reports folder
    // - Any other non-empty   -> Others folder
    // - None or empty         -> no folder link
    //

    // for old storage
    // let folder_name_opt: Option<&str> = category
    //     .as_deref()
    //     .map(str::trim)
    //     .filter(|s| !s.is_empty())
    //     .map(|name| {
    //         let lower = name.to_lowercase();
    //         match lower.as_str() {
    //             "finance" => "Finance",
    //             "report" | "reports" => "Reports",
    //             _ => "Others",
    //         }
    //     });

    // if let Some(folder_name) = folder_name_opt {
    //     debug!(
    //         document_id = %document.id,
    //         folder = folder_name,
    //         "Linking document to folder"
    //     );

    //     let folder_id: Uuid = sqlx::query_scalar(
    //         r#"
    //         SELECT id
    //         FROM folders
    //         WHERE name = $1
    //         "#,
    //     )
    //     .bind(folder_name)
    //     .fetch_one(&mut *tx)
    //     .await?;

    //     sqlx::query(
    //         r#"
    //         INSERT INTO document_folders (document_id, folder_id)
    //         VALUES ($1, $2)
    //         ON CONFLICT (document_id, folder_id) DO NOTHING
    //         "#,
    //     )
    //     .bind(document.id)
    //     .bind(folder_id)
    //     .execute(&mut *tx)
    //     .await?;
    // } else {
    //     debug!(
    //         document_id = %document.id,
    //         "No folder mapping for this document (no or empty category)"
    //     );
    // };

    // // Example key: "{document_id}/v{version_number}"
    // let stored_path = format!("{}/v{}", document.id, next_version_number);

    let folder_name = category.map(|s| s.to_string());

    // If category is provided, ensure folder metadata exists
    if let Some(ref cat) = folder_name {
        let sanitized_name = sanitize_folder_name(cat);
        let metadata_path = format!("{}/.folder_metadata.json", sanitized_name);

        // Check if metadata file already exists
        let metadata_exists = state.storage.stat(&metadata_path).await.is_ok();

        if !metadata_exists {
            // Create folder metadata
            let folder_metadata = FolderMetadata {
                folder_name: sanitized_name.clone(),
                created_by: current_user.id,
                created_by_username: current_user.username.clone(),
                created_at: Utc::now(),
            };

            let metadata_json = serde_json::to_string(&folder_metadata).map_err(|e| {
                AppError::Other(anyhow::anyhow!(
                    "Failed to serialize folder metadata: {}",
                    e
                ))
            })?;

            let metadata_bytes = metadata_json.into_bytes();

            if let Err(e) = state.storage.write(&metadata_path, metadata_bytes).await {
                warn!(
                    error = ?e,
                    folder_name = %sanitized_name,
                    "Failed to create folder metadata during upload, continuing anyway"
                );
            } else {
                debug!(
                    folder_name = %sanitized_name,
                    "Created folder metadata during file upload"
                );
            }
        }
    }

    let stored_path = build_storage_path_with_folder(folder_name, document.id, next_version_number);

    info!(
        file_name = %file_name,
        file_size = file_bytes.len(),
        stored_key = %stored_path,
        "Saving file via OpenDAL using document/version-based key"
    );
    state
        .storage
        .write(&stored_path, file_bytes.clone())
        .await?;

    // Insert version with computed version number
    let version = sqlx::query_as::<_, DocumentVersion>(r#"
        INSERT INTO document_versions 
        (document_id, version_number, file_name, file_path, file_size, mime_type, checksum)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, document_id, version_number, file_name, file_path, file_size, mime_type, checksum, created_at
        "#,
    )
    .bind(document.id)
    .bind(next_version_number)
    .bind(&file_name)
    // `stored_path` is the OpenDAL key (e.g., "{document_id}/v{version_number}")
    // In the old filesystem-based code this was a full path on disk.
    .bind(&stored_path)
    .bind(file_size)
    .bind(&mime_type)
    .bind(&checksum)
    .fetch_one(&mut *tx)
    .await?;

    let metadata_count = metadata.len();

    // Insert metadata entries (optional). Upsert on (document_id, key)
    for (meta_key, meta_value) in metadata.into_iter() {
        sqlx::query(
            r#"
                INSERT INTO document_metadata (document_id, key, value)
                VALUES ($1, $2, $3)
                ON CONFLICT (document_id, key)
                DO UPDATE SET value = EXCLUDED.value
                "#,
        )
        .bind(document.id)
        .bind(&meta_key)
        .bind(&meta_value)
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            warn!(error = ?err, meta_key = %meta_key, "Failed to insert metadata");
            AppError::Db(err)
        })?;
    }

    debug!("Committing database transaction");
    tx.commit().await.map_err(|err| {
        warn!(error = ?err, "Failed to commit transaction");
        AppError::Db(err)
    })?;

    if let Err(e) = log_upload(
        &state.pool,
        current_user.id.to_string(),
        document.id,
        next_version_number,
        Some(json!({
            "file_name": &file_name,
            "file_size": file_size,
            "mime_type": &mime_type,
            "checksum": &checksum,
            "metadata_count": metadata_count,
        })),
    )
    .await
    {
        // Log the error but don't fail the upload operation
        // Audit logging should not break the main functionality
        warn!(
            error = ?e,
            document_id = %document.id,
            user_id = %current_user.id,
            "Failed to create audit log for upload"
        );
    }

    let response = UploadResponse {
        document_id: document.id,
        version_id: version.id,
        // In the old implementation this was a filesystem path:
        // stored_path: stored_path.to_string_lossy().to_string(),
        // Now we store the OpenDAL key (relative path) instead.
        stored_path: stored_path.to_string(),
        metadata_message: format!(
            "Inserted/updated {metadata_count} metadata entries{}",
            if metadata_count > 0 {
                format!(": {}", metadata_keys.join(","))
            } else {
                "".to_string()
            }
        ),
    };

    info!(
        document_id = %document.id,
        version_id = %version.id,
        version_number = next_version_number,
        metadata_count = metadata_count,
        "File uploaded successfully"
    );

    Ok(Json(response))
}
