use axum::Router;
use crate::state::AppState;

pub mod upload;
pub mod documents;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(upload::routes())
        .merge(documents::routes())
        .with_state(state)
}