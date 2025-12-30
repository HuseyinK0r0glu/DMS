use utoipa::OpenApi;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use crate::models::{Document, DocumentVersion, AuditLog, AuditAction};
use crate::dtos::{UploadResponse, ListDocumentsResponse, ListDocumentsQuery, DownloadQuery, AuditResponse, DocumentWithLatest, CreateFolderRequest, CreateFolderResponse,AddTagToDocumentRequest,AddTagToDocumentResponse,TagInfo, LoginRequest, LoginResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::upload::upload_file,
        crate::routes::documents::list_documents,
        crate::routes::documents::download_document,
        crate::routes::documents::soft_delete_document,
        crate::routes::documents::hard_delete_document,
        crate::routes::audit::get_actions,
        crate::routes::folders::create_folder,
        crate::routes::tags::add_tags_to_document,
        crate::routes::login::login,
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
        CreateFolderRequest,
        CreateFolderResponse,
        AddTagToDocumentRequest,
        AddTagToDocumentResponse,
        TagInfo,
        LoginRequest,
        LoginResponse,
    )),
    tags(
        (name = "documents", description = "Document management endpoints"),
        (name = "upload", description = "File upload endpoints"),
        (name = "audit", description = "Audit log endpoints (admin only)"),
        (name = "folders", description = "Folder management endpoints"),
        (name = "tags", description = "Tag management endpoints"),
        (name = "auth", description = "Authentication endpoints"),
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

pub fn openapi_with_security() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    if let Some(components) = openapi.components.as_mut() {
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
        );
    }
    openapi
}