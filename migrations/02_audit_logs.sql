-- ==========================================
--  AUDIT LOGS TABLE (Immutable Audit Trail)
-- ==========================================
-- 
-- This table stores immutable audit logs for compliance and tracking.
-- Records are never updated or deleted once created.
-- 
-- Actions tracked:
--   - UPLOAD: Document uploaded
--   - DOWNLOAD: Document downloaded
--   - UPDATE_METADATA: Document metadata updated
--   - CREATE_VERSION: New version created
--   - DELETE: Document deleted (soft or hard)
--   - RESTORE_VERSION: Previous version restored
--

-- Create ENUM type for audit actions
CREATE TYPE audit_action AS ENUM (
    'UPLOAD',
    'DOWNLOAD',
    'UPDATE_METADATA',
    'CREATE_VERSION',
    'DELETE',
    'RESTORE_VERSION'
);

-- ==========================================
--  AUDIT LOGS TABLE
-- ==========================================
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- WHO performed the action
    user_id VARCHAR(255) NOT NULL,
    
    -- WHAT action was performed
    action audit_action NOT NULL,
    
    -- Target document (nullable for actions that don't target a specific document)
    document_id UUID REFERENCES documents(id) ON DELETE SET NULL,
    
    -- Version affected (nullable, only relevant for version-specific actions)
    document_version INTEGER,
    
    -- Extra context/metadata (JSONB for flexible structure)
    metadata JSONB DEFAULT '{}'::jsonb,
    
    -- WHEN the action occurred (immutable timestamp)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- ==========================================
--  INDEXES for efficient querying
-- ==========================================

-- Index for querying by user
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id 
    ON audit_logs(user_id);

-- Index for querying by action type
CREATE INDEX IF NOT EXISTS idx_audit_logs_action 
    ON audit_logs(action);

-- Index for querying by document
CREATE INDEX IF NOT EXISTS idx_audit_logs_document_id 
    ON audit_logs(document_id);

-- Index for querying by timestamp (for time-range queries)
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at 
    ON audit_logs(created_at DESC);

-- Composite index for common queries: user + action
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_action 
    ON audit_logs(user_id, action);

-- Composite index for document history: document + timestamp
CREATE INDEX IF NOT EXISTS idx_audit_logs_document_created 
    ON audit_logs(document_id, created_at DESC);

-- Index for metadata queries (GIN index for JSONB)
CREATE INDEX IF NOT EXISTS idx_audit_logs_metadata 
    ON audit_logs USING GIN (metadata);

-- ==========================================
--  COMMENTS for documentation
-- ==========================================
COMMENT ON TABLE audit_logs IS 'Immutable audit trail for compliance and tracking';
COMMENT ON COLUMN audit_logs.id IS 'Unique audit record identifier';
COMMENT ON COLUMN audit_logs.user_id IS 'User who performed the action';
COMMENT ON COLUMN audit_logs.action IS 'Type of action performed';
COMMENT ON COLUMN audit_logs.document_id IS 'Target document (if applicable)';
COMMENT ON COLUMN audit_logs.document_version IS 'Document version affected (if applicable)';
COMMENT ON COLUMN audit_logs.metadata IS 'Additional context/metadata as JSON';
COMMENT ON COLUMN audit_logs.created_at IS 'Timestamp when action occurred (immutable)';

