use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::{ToSchema, schema};

/// Document model - represents a logical document
/// Maps to the `documents` table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Document {
    /// Primary key - UUID
    pub id: Uuid,
    
    /// Document title - VARCHAR(255) NOT NULL
    pub title: String,
    
    /// Optional category - VARCHAR(100) NULLABLE
    pub category: Option<String>,
    
    /// Soft delete timestamp - NULL means not deleted, non-NULL means soft-deleted
    pub deleted_at: Option<DateTime<Utc>>,
    
    /// Creation timestamp - TIMESTAMP WITH TIME ZONE
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp - TIMESTAMP WITH TIME ZONE
    pub updated_at: DateTime<Utc>,
}

/// DocumentVersion model - represents a physical file version
/// Maps to the `document_versions` table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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
    pub password: Option<String>, 
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Audit action enum - maps to PostgreSQL `audit_action` ENUM type
/// Represents the type of action performed on a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "audit_action", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
    /// Document uploaded
    Upload,
    /// Document downloaded
    Download,
    /// Document metadata updated
    UpdateMetadata,
    /// New version created
    CreateVersion,
    /// Document deleted (soft or hard)
    Delete,
    /// Previous version restored
    RestoreVersion,
}

/// Audit log model - represents an immutable audit record
/// Maps to the `audit_logs` table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AuditLog {
    /// Primary key - UUID
    pub id: Uuid,
    
    /// User who performed the action - VARCHAR(255) NOT NULL
    pub user_id: String,
    
    /// Type of action performed - audit_action NOT NULL
    pub action: AuditAction,
    
    /// Target document (nullable for actions that don't target a specific document)
    /// UUID REFERENCES documents(id) ON DELETE SET NULL
    pub document_id: Option<Uuid>,
    
    /// Document version affected (nullable, only relevant for version-specific actions)
    /// INTEGER NULLABLE
    pub document_version: Option<i32>,
    
    /// Additional context/metadata as JSON - JSONB DEFAULT '{}'::jsonb
    #[schema(value_type = Object)]
    pub metadata: JsonValue,
    
    /// Timestamp when action occurred (immutable) - TIMESTAMP WITH TIME ZONE NOT NULL
    pub created_at: DateTime<Utc>,
}

/// New audit log input - for creating audit logs without ID and timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAuditLog {
    /// User who performed the action
    pub user_id: String,
    
    /// Type of action performed
    pub action: AuditAction,
    
    /// Target document (optional)
    pub document_id: Option<Uuid>,
    
    /// Document version affected (optional)
    pub document_version: Option<i32>,
    
    /// Additional context/metadata as JSON (optional, defaults to empty object)
    #[serde(default)]
    pub metadata: JsonValue,
}

/// Tag model - represents a tag that can be associated with documents
/// Maps to the `tags` table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Tag {
    /// Primary key - UUID
    pub id: Uuid,
    
    /// Tag name - TEXT NOT NULL UNIQUE
    pub name: String,
    
    /// Creation timestamp - TIMESTAMP WITH TIME ZONE
    pub created_at: DateTime<Utc>,
}

/// DocumentTag model - represents the many-to-many relationship between documents and tags
/// Maps to the `document_tags` table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DocumentTag {
    /// Foreign key to documents table - UUID NOT NULL
    pub document_id: Uuid,
    
    /// Foreign key to tags table - UUID NOT NULL
    pub tag_id: Uuid,
}

/// New tag input - for creating tags without ID and timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTag {
    /// Tag name
    pub name: String,
}

/// New document tag input - for creating document-tag relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDocumentTag {
    /// Document ID
    pub document_id: Uuid,
    
    /// Tag ID
    pub tag_id: Uuid,
}