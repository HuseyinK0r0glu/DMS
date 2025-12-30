use sqlx::PgPool;
use opendal::Operator;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub storage: Operator,
}