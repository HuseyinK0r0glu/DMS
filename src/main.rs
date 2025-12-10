#![allow(unused_imports, unused_variables, non_snake_case, unused_mut, dead_code)]
mod models;
mod dtos;

use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use models::*;
use dtos::{UploadResponse, DocumentWithLatest, ListDocumentsResponse, ListDocumentsQuery};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use std::{collections::HashMap, fs, path::PathBuf};
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    upload_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL is not set"))?;

    let pool = PgPool::connect(&database_url).await?;

    // Ensure uploads directory exists
    let upload_dir = PathBuf::from("uploads");
    fs::create_dir_all(&upload_dir)?;

    let app = Router::new()
        .route("/upload", post(upload_file))
        .route("/documents" , get(list_documents))
        .with_state(AppState { pool, upload_dir });

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn list_documents(
    State(state): State<AppState>,
    Query(params): Query<ListDocumentsQuery>,
) -> Result<Json<ListDocumentsResponse>, (StatusCode, &'static str)> {
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
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "count_failed"))?;

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
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "query_failed"))?;

    let resp = ListDocumentsResponse {
        data: rows,
        page,
        page_size,
        total: total.0,
    };
    Ok(Json(resp))
}

async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
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
                            return (
                                StatusCode::BAD_REQUEST,
                                "Invalid document_id (must be UUID)",
                            )
                                .into_response();
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
                            return (
                                StatusCode::BAD_REQUEST,
                                "Invalid metadata JSON; expected an object of string values",
                            )
                                .into_response();
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
        None => return (StatusCode::BAD_REQUEST, "Missing file").into_response(),
    };
    let file_name = file_name.unwrap_or_else(|| "upload.bin".to_string());

    // Persist file
    let stored_file_name = format!("{}_{}", Uuid::new_v4(), file_name);
    let stored_path = state.upload_dir.join(&stored_file_name);
    if let Err(err) = fs::write(&stored_path, &file_bytes) {
        eprintln!("Failed to write file: {err}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to store file",
        )
            .into_response();
    }

    let file_size = file_bytes.len() as i64;
    let checksum = None::<String>;

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database unavailable",
            )
                .into_response()
        }
    };

    // Create new document or append to existing
    let (document, next_version_number) = if let Some(doc_id) = document_id {
        // Existing document: ensure it exists
        let doc = match sqlx::query_as::<_, Document>(
            r#"
            SELECT id, title, category, created_at, updated_at
            FROM documents
            WHERE id = $1
            "#,
        )
        .bind(doc_id)
        .fetch_optional(&mut *tx)
        .await
        {
            Ok(Some(d)) => d,
            Ok(None) => {
                return (
                    StatusCode::BAD_REQUEST,
                    "document_id not found",
                )
                    .into_response();
            }
            Err(err) => {
                eprintln!("Fetch document failed: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to load document",
                )
                    .into_response();
            }
        };

        // Next version number
        let next_version: i32 = match sqlx::query_scalar::<_, Option<i32>>(
            r#"
            SELECT MAX(version_number) + 1
            FROM document_versions
            WHERE document_id = $1
            "#,
        )
        .bind(doc_id)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(Some(v)) => v,
            Ok(None) => 1,
            Err(err) => {
                eprintln!("Compute next version failed: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to compute version",
                )
                    .into_response();
            }
        };

        (doc, next_version)
    } else {
        // New document: require title
        let title = match title_opt {
            Some(t) if !t.is_empty() => t,
            _ => return (StatusCode::BAD_REQUEST, "Missing title").into_response(),
        };

        let doc = match sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (title, category)
            VALUES ($1, $2)
            RETURNING id, title, category, created_at, updated_at
            "#,
        )
        .bind(&title)
        .bind(&category)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(d) => d,
            Err(err) => {
                eprintln!("Insert document failed: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to save document",
                )
                    .into_response();
            }
        };

        (doc, 1)
    };

    // Insert version with computed version number
    let version = match sqlx::query_as::<_, DocumentVersion>(
        r#"
        INSERT INTO document_versions 
        (document_id, version_number, file_name, file_path, file_size, mime_type, checksum)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, document_id, version_number, file_name, file_path, file_size, mime_type, checksum, created_at
        "#,
    )
    .bind(document.id)
    .bind(next_version_number)
    .bind(&file_name)
    .bind(stored_path.to_string_lossy())
    .bind(file_size)
    .bind(&mime_type)
    .bind(&checksum)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Insert version failed: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to save document version",
            )
                .into_response();
        }
    };

    let metadata_count = metadata.len();

    // Insert metadata entries (optional). Upsert on (document_id, key)
    for (meta_key, meta_value) in metadata.into_iter() {
        if let Err(err) = sqlx::query(
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
        {
            eprintln!("Insert metadata failed (key={}): {err}", meta_key);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to save metadata",  
            )
                .into_response();
        }
    }

    if tx.commit().await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to finalize save",
        )
            .into_response();
    }

    let response = UploadResponse {
        document_id: document.id,
        version_id: version.id,
        stored_path: stored_path.to_string_lossy().to_string(),
        metadata_message: format!(
            "Inserted/updated {metadata_count} metadata entries{}",
            if metadata_count > 0 {
                format!(": {}", metadata_keys.join(","))
            } else {
                "".to_string()
            }
        ),
    };

    (StatusCode::OK, Json(response)).into_response()
}
