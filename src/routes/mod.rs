use axum::Router;
use crate::state::AppState;
use tower_http::trace::TraceLayer;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;

pub mod upload;
pub mod documents;
pub mod audit;

use crate::openapi::openapi_with_security; 

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", openapi_with_security()))
        .merge(upload::routes())
        .merge(documents::routes())
        .merge(audit::routes())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                .on_request(|_request: &axum::http::Request<_>, _span: &tracing::Span| {
                    tracing::debug!("request started");
                })
                .on_response(|response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::info!(
                        status = %response.status(),
                        latency_ms = latency.as_millis(),
                        "request completed"
                    );
                })
        )
        .with_state(state)
}