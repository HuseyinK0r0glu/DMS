# Database Models Documentation

This document describes the Rust models that correspond to the database tables.

## Type Mappings

The following PostgreSQL types map to Rust types:

| PostgreSQL Type | Rust Type | Notes |
|----------------|-----------|-------|
| `UUID` | `uuid::Uuid` | Primary keys and foreign keys |
| `VARCHAR(n)` | `String` | Text fields |
| `VARCHAR(n)` (nullable) | `Option<String>` | Optional text fields |
| `TEXT` | `String` | Large text fields |
| `TEXT` (nullable) | `Option<String>` | Optional large text fields |
| `INTEGER` | `i32` | Version numbers |
| `BIGINT` | `i64` | File sizes |
| `TIMESTAMP WITH TIME ZONE` | `chrono::DateTime<chrono::Utc>` | Timestamps |

## Models

### Document

Represents a logical document in the system.

```rust
pub struct Document {
    pub id: Uuid,                    // Primary key
    pub title: String,               // Document title (required)
    pub category: Option<String>,    // Optional category
    pub created_at: DateTime<Utc>,   // Creation timestamp
    pub updated_at: DateTime<Utc>,   // Last update timestamp
}
```

**Database Table:** `documents`

### DocumentVersion

Represents a physical file version associated with a document.

```rust
pub struct DocumentVersion {
    pub id: Uuid,                    // Primary key
    pub document_id: Uuid,           // Foreign key to documents
    pub version_number: i32,         // Version number (1, 2, 3, ...)
    pub file_name: String,           // Original file name
    pub file_path: String,           // Storage path
    pub file_size: i64,              // File size in bytes
    pub mime_type: Option<String>,   // MIME type (e.g., "application/pdf")
    pub checksum: Option<String>,    // File checksum (MD5 or SHA-256)
    pub created_at: DateTime<Utc>,   // Creation timestamp
}
```

**Database Table:** `document_versions`

**Constraints:**
- Unique constraint on `(document_id, version_number)`
- Foreign key constraint on `document_id` with CASCADE delete

### DocumentMetadata

Represents key-value metadata pairs for documents.

```rust
pub struct DocumentMetadata {
    pub id: Uuid,                    // Primary key
    pub document_id: Uuid,           // Foreign key to documents
    pub key: String,                  // Metadata key
    pub value: Option<String>,        // Metadata value (optional)
    pub created_at: DateTime<Utc>,    // Creation timestamp
}
```

**Database Table:** `document_metadata`

**Constraints:**
- Unique constraint on `(document_id, key)`
- Foreign key constraint on `document_id` with CASCADE delete

## Helper Structs

### NewDocument

Used for creating new documents without specifying ID or timestamps.

```rust
pub struct NewDocument {
    pub title: String,
    pub category: Option<String>,
}
```

### NewDocumentVersion

Used for creating new document versions without specifying ID or timestamps.

```rust
pub struct NewDocumentVersion {
    pub document_id: Uuid,
    pub version_number: i32,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
}
```

### NewDocumentMetadata

Used for creating new metadata entries without specifying ID or timestamps.

```rust
pub struct NewDocumentMetadata {
    pub document_id: Uuid,
    pub key: String,
    pub value: Option<String>,
}
```

## Features

All models include:
- `Debug` - For debugging output
- `Clone` - For cloning instances
- `Serialize` / `Deserialize` - For JSON serialization (via serde)
- `FromRow` - For automatic mapping from SQL query results (via sqlx)

## Usage Example

See `src/models_example.rs` for example functions demonstrating how to use these models with sqlx.

## Dependencies

The models require the following dependencies (already in `Cargo.toml`):

- `sqlx` - Async SQL toolkit
- `uuid` - UUID support
- `chrono` - Date/time handling
- `serde` - Serialization support

