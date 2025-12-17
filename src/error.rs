use axum::{http::StatusCode,response::{IntoResponse,Response},Json};

use serde::Serialize;
use thiserror::Error;
use tracing::error;

#[derive(Debug,Error)]
pub enum AppError {

    #[error("bad request: {0}")]
    BadRequest(&'static str),

    #[error("not found: {0}")]
    NotFound(&'static str),

    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("env error: {0}")]
    Env(#[from] std::env::VarError),

    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

// error --> HTTP mapping

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::BadRequest(msg) => {
                tracing::warn!(message = %msg, "Bad request");
                StatusCode::BAD_REQUEST
            },  
            AppError::NotFound(msg) => {
                tracing::info!(message = %msg, "Resource not found"); 
                StatusCode::NOT_FOUND
            },
            AppError::Db(_) | AppError::Io(_) | AppError::Env(_) | AppError::Other(_) => {
                error!(error = ?self, "Internal server error"); 
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let body = ErrorBody {
            error: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}