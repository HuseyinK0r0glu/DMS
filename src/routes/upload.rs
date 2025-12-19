use axum::{routing::post, Router};
use axum::extract::{Multipart, State};
use axum::Json;
use crate::{state::AppState, dtos::UploadResponse, error::AppError, models::{Document, DocumentVersion}};
use uuid::Uuid;
use serde_json::Value;
use std::{collections::HashMap, fs};
use tracing::{info, debug, warn};

pub fn routes() -> Router<AppState> {
    Router::new().route("/upload", post(upload_file))
}

async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    info!("File upload request received");

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
                            return Err(AppError::BadRequest("Invalid metadata JSON; expected an object of string values"));
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
    // NEW APPROACH (what you asked for):
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
            SELECT id, title, category, created_at, updated_at
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
        let next_version_opt: Option<i32> = sqlx::query_scalar::<_, Option<i32>>(r#"
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

        let doc = sqlx::query_as::<_, Document>(r#"
            INSERT INTO documents (title, category)
            VALUES ($1, $2)
            RETURNING id, title, category, created_at, updated_at
            "#,
        )
        .bind(&title)
        .bind(&category)
        .fetch_one(&mut *tx)
        .await?;
            (doc, 1)
        };

    // Now that we know document.id and next_version_number, build the storage key.
    // Example key: "{document_id}/v{version_number}"
    let stored_path = format!("{}/v{}", document.id, next_version_number);

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
        sqlx::query(r#"
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
    tx.commit()
        .await
        .map_err(|err| {
            warn!(error = ?err, "Failed to commit transaction");
            AppError::Db(err)
        })?;

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
