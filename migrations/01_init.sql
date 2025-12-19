-- ==========================================
--  ENABLE EXTENSIONS
-- ==========================================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ==========================================
--  DOCUMENTS TABLE  (Logical Document)
-- ==========================================
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    category VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
--  DOCUMENT VERSIONS TABLE (Actual File)
-- ==========================================
CREATE TABLE IF NOT EXISTS document_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,             -- 1,2,3,...
    file_name VARCHAR(255) NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100),
    checksum VARCHAR(128),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Ensure version numbers are unique per document
CREATE UNIQUE INDEX IF NOT EXISTS uniq_document_versions 
    ON document_versions(document_id, version_number);

-- Index for faster queries
CREATE INDEX IF NOT EXISTS idx_doc_versions_doc_id 
    ON document_versions(document_id);

-- ==========================================
--  USERS TABLE (for API authentication & authorization)
-- ==========================================
--
-- This table models application users that will be used for
-- authorization of storage operations (read, write, delete, stat).
--
-- `role` is a simple text role for now:
--   - 'admin'  : full access (read/write/delete/stat)
--   - 'editor' : read + write
--   - 'viewer' : read-only
--
-- `api_key` is a static token that clients can send in a header
-- like `X-API-Key` to authenticate. In a real system you may want
-- stronger auth (hashed keys, JWTs, etc.), but this is sufficient
-- for development and Postman testing.

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(100) UNIQUE NOT NULL,
    api_key VARCHAR(255) UNIQUE NOT NULL,
    role VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_users_role 
    ON users(role);

-- ==========================================
--  DOCUMENT METADATA TABLE (Dynamic Key-Value)
-- ==========================================
CREATE TABLE IF NOT EXISTS document_metadata (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(document_id, key)
);

CREATE INDEX IF NOT EXISTS idx_metadata_document_id 
    ON document_metadata(document_id);

-- ==========================================
--  FOLDERS TABLE
-- ==========================================

CREATE TABLE IF NOT EXISTS folders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
--  DOCUMENT_FOLDERS TABLE (Many-to-Many)
-- ==========================================

CREATE TABLE IF NOT EXISTS document_folders (
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    folder_id UUID NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, folder_id)
);

CREATE INDEX IF NOT EXISTS idx_document_folders_document_id 
    ON document_folders(document_id);

CREATE INDEX IF NOT EXISTS idx_document_folders_folder_id 
    ON document_folders(folder_id);

-- ==========================================
--  TRIGGER: update updated_at column
-- ==========================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Attach triggers to documents only (versions/metadata do not need updated_at)
CREATE TRIGGER update_documents_updated_at 
BEFORE UPDATE ON documents
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==========================================
--  SEED DEFAULT FOLDERS
-- ==========================================

INSERT INTO folders (name) VALUES
    ('Finance'),
    ('Reports'),
    ('Others')
ON CONFLICT (name) DO NOTHING;

-- ==========================================
--  SEED SAMPLE USERS
-- ==========================================
--
-- These are development users with hard-coded API keys that you can
-- use from Postman or other clients. Example usage:
--   X-API-Key: admin-key-123
--
-- In production you would want to generate and store secure keys,
-- and never commit them to source control.

INSERT INTO users (username, api_key, role) VALUES
    ('admin_user',  'admin-key-123',  'admin'),
    ('editor_user', 'editor-key-123', 'editor'),
    ('viewer_user', 'viewer-key-123', 'viewer'),
    ('unauthorized_user', 'unauthorized-key-123', 'unauthorized')
ON CONFLICT (api_key) DO NOTHING;
