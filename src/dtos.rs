use crate::models::AuditLog;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct UploadResponse {
    pub document_id: Uuid,
    pub version_id: Uuid,
    pub stored_path: String,
    pub metadata_message: String,
}

#[derive(Serialize, FromRow, ToSchema)]
pub struct DocumentWithLatest {
    pub id: Uuid,
    pub title: String,
    pub category: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub latest_version_number: Option<i32>,
    pub latest_file_name: Option<String>,
    pub latest_file_size: Option<i64>,
    pub latest_mime_type: Option<String>,
    pub latest_created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct ListDocumentsResponse {
    pub data: Vec<DocumentWithLatest>,
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct ListDocumentsQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub title: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct DownloadQuery {
    pub version: Option<i32>,
}

#[derive(Serialize, ToSchema)]
pub struct AuditResponse {
    pub data: Vec<AuditLog>,
    pub total: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateFolderRequest {
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateFolderResponse {
    pub folder_name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct AddTagToDocumentRequest {
    pub document_id: Uuid,
    pub tags: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct TagInfo {
    pub tag_id: Uuid,
    pub tag_name: String,
    pub tag_created: bool,
}

#[derive(Serialize, ToSchema)]
pub struct AddTagToDocumentResponse {
    pub document_id: Uuid,
    pub tags: Vec<TagInfo>,
    pub total: usize,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub api_key: String,
    pub username: String,
    pub user_id: Uuid,
    pub role: String,
}
