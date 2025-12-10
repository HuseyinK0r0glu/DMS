use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct UploadResponse {
    pub document_id: Uuid,
    pub version_id: Uuid,
    pub stored_path: String,
    pub metadata_message: String,
}