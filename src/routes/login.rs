use crate::dtos::{LoginRequest, LoginResponse};
use crate::error::AppError;
use crate::models::User;
use crate::state::AppState;
use axum::{extract::State, routing::post, Json, Router};
use tracing::{debug, info, warn};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new().route("/auth/login", post(login))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid username or password"),
        (status = 400, description = "Bad request - missing username or password")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // Validate input
    if request.username.trim().is_empty() {
        return Err(AppError::BadRequest("Username cannot be empty"));
    }

    if request.password.trim().is_empty() {
        return Err(AppError::BadRequest("Password cannot be empty"));
    }

    debug!(username = %request.username, "Attempting login");

    // Query database for user by username
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, api_key, password, role, created_at
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(request.username.trim())
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::Db)?;

    match user {
        Some(u) => {
            // Check if password matches (plain text comparison)
            if let Some(db_password) = &u.password {
                if db_password != request.password.trim() {
                    warn!(username = %request.username, "Invalid password");
                    return Err(AppError::BadRequest("Invalid username or password"));
                }
            } else {
                // If password is NULL in database, reject login
                warn!(username = %request.username, "User has no password set");
                return Err(AppError::BadRequest("Invalid username or password"));
            }

            info!(
                user_id = %u.id,
                username = %u.username,
                role = %u.role,
                "User logged in successfully"
            );

            Ok(Json(LoginResponse {
                api_key: u.api_key,
                username: u.username,
                user_id: u.id,
                role: u.role,
            }))
        }
        None => {
            warn!(username = %request.username, "User not found");
            Err(AppError::BadRequest("Invalid username or password"))
        }
    }
}
