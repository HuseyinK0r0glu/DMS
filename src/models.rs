use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Document model - represents a logical document
/// Maps to the `documents` table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    /// Primary key - UUID
    pub id: Uuid,
    
    /// Document title - VARCHAR(255) NOT NULL
    pub title: String,
    
    /// Optional category - VARCHAR(100) NULLABLE
    pub category: Option<String>,
    
    /// Creation timestamp - TIMESTAMP WITH TIME ZONE
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp - TIMESTAMP WITH TIME ZONE
    pub updated_at: DateTime<Utc>,
}

/// DocumentVersion model - represents a physical file version
/// Maps to the `document_versions` table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentVersion {
    /// Primary key - UUID
    pub id: Uuid,
    
    /// Foreign key to documents table - UUID NOT NULL
    pub document_id: Uuid,
    
    /// Version number (1, 2, 3, ...) - INTEGER NOT NULL
    pub version_number: i32,
    
    /// Original file name - VARCHAR(255) NOT NULL
    pub file_name: String,
    
    /// File path on storage - TEXT NOT NULL
    pub file_path: String,
    
    /// File size in bytes - BIGINT NOT NULL
    pub file_size: i64,
    
    /// MIME type (e.g., "application/pdf") - VARCHAR(100) NULLABLE
    pub mime_type: Option<String>,
    
    /// File checksum (MD5 or SHA-256) - VARCHAR(128) NULLABLE
    pub checksum: Option<String>,
    
    /// Creation timestamp - TIMESTAMP WITH TIME ZONE
    pub created_at: DateTime<Utc>,
}

/// DocumentMetadata model - represents key-value metadata pairs
/// Maps to the `document_metadata` table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentMetadata {
    /// Primary key - UUID
    pub id: Uuid,
    
    /// Foreign key to documents table - UUID NOT NULL
    pub document_id: Uuid,
    
    /// Metadata key - VARCHAR(255) NOT NULL
    pub key: String,
    
    /// Metadata value - TEXT NULLABLE
    pub value: Option<String>,
    
    /// Creation timestamp - TIMESTAMP WITH TIME ZONE
    pub created_at: DateTime<Utc>,
}

/// New document input - for creating documents without ID and timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDocument {
    pub title: String,
    pub category: Option<String>,
}

/// New document version input - for creating versions without ID and timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDocumentVersion {
    pub document_id: Uuid,
    pub version_number: i32,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
}

/// New document metadata input - for creating metadata without ID and timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDocumentMetadata {
    pub document_id: Uuid,
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub api_key: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}