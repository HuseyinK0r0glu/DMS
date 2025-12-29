use serde::{Deserialize , Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use crate::models::AuditLog;

#[derive(Serialize)]
pub struct UploadResponse {
    pub document_id: Uuid,
    pub version_id: Uuid,
    pub stored_path: String,
    pub metadata_message: String,
}

#[derive(Serialize,FromRow)]
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

#[derive(Serialize)]
pub struct ListDocumentsResponse {
    pub data : Vec<DocumentWithLatest>,
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
}

#[derive(Deserialize)]
pub struct ListDocumentsQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub title: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct DownloadQuery {
    pub version: Option<i32>, 
}

#[derive(Serialize)]
pub struct AuditResponse {
    pub data: Vec<AuditLog>,
    pub total: i64,
}