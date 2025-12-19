use std::path::PathBuf;
use sqlx::PgPool;
use opendal::Operator;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub upload_dir: PathBuf,
    pub storage: Operator,
}