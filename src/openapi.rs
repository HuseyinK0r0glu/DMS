use utoipa::OpenApi;
use crate::models::{Document, DocumentVersion, AuditLog, AuditAction};
use crate::dtos::{UploadResponse, ListDocumentsResponse, ListDocumentsQuery, DownloadQuery, AuditResponse, DocumentWithLatest};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::upload::upload_file,
        crate::routes::documents::list_documents,
        crate::routes::documents::download_document,
        crate::routes::documents::soft_delete_document,
        crate::routes::documents::hard_delete_document,
        crate::routes::audit::get_actions,
    ),
    components(schemas(
        Document,
        DocumentVersion,
        AuditLog,
        AuditAction,
        UploadResponse,
        DocumentWithLatest,
        ListDocumentsResponse,
        ListDocumentsQuery,
        DownloadQuery,
        AuditResponse,
    )),
    tags(
        (name = "documents", description = "Document management endpoints"),
        (name = "upload", description = "File upload endpoints"),
        (name = "audit", description = "Audit log endpoints (admin only)"),
    ),
    info(
        title = "Document Management System API",
        version = "1.0.0",
        description = "REST API for managing documents, versions, and audit logs"
    ),
    servers(
        (url = "http://localhost:3000", description = "Development server")
    )
)]
pub struct ApiDoc;