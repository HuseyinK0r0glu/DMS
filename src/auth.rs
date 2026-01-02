use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::User;
use crate::state::AppState;

/// Represents the currently authenticated user
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub username: String,
    pub role: String,
}

/// Extract CurrentUser from the X-API-Key header
#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Get AppState from the router state
        let app_state = AppState::from_ref(state);

        // Extract X-API-Key header
        let api_key = parts
            .headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                warn!("Missing X-API-Key header");
                AppError::BadRequest("Missing X-API-Key header")
            })?;

        debug!(api_key = %api_key, "Authenticating user with API key");

        // Query database for user with this API key
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, api_key, password, role, created_at
            FROM users
            WHERE api_key = $1
            "#,
        )
        .bind(api_key)
        .fetch_optional(&app_state.pool)
        .await
        .map_err(AppError::Db)?;

        match user {
            Some(u) => {
                debug!(user_id = %u.id, username = %u.username, role = %u.role, "User authenticated");
                Ok(CurrentUser {
                    id: u.id,
                    username: u.username,
                    role: u.role,
                })
            }
            None => {
                warn!(api_key = %api_key, "Invalid API key");
                Err(AppError::BadRequest("Invalid API key"))
            }
        }
    }
}

/// Storage actions that require permission checks
#[derive(Debug, Clone, Copy)]
pub enum StorageAction {
    Read,
    Write,
    Delete,
    Stat,
    GetActions,
}

/// Check if a user has permission for a specific storage action
pub fn check_permission(user: &CurrentUser, action: StorageAction) -> Result<(), AppError> {
    match action {
        StorageAction::Read => {
            // viewer, editor, admin can all read
            if user.role == "viewer" || user.role == "editor" || user.role == "admin" {
                Ok(())
            } else {
                Err(AppError::BadRequest(
                    "Permission denied: read access required",
                ))
            }
        }
        StorageAction::Write => {
            // only editor and admin can write
            if user.role == "editor" || user.role == "admin" {
                Ok(())
            } else {
                Err(AppError::BadRequest(
                    "Permission denied: write access required",
                ))
            }
        }
        StorageAction::Delete => {
            // only admin can delete
            if user.role == "admin" {
                Ok(())
            } else {
                Err(AppError::BadRequest(
                    "Permission denied: admin access required",
                ))
            }
        }
        StorageAction::Stat => {
            // same as read for now
            check_permission(user, StorageAction::Read)
        }
        StorageAction::GetActions => {
            // only admin can reach to actions
            if user.role == "admin" {
                Ok(())
            } else {
                Err(AppError::BadRequest(
                    "Permission denied: admin access required",
                ))
            }
        }
    }
}
