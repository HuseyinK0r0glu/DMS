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
