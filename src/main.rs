#![allow(unused_imports, unused_variables, non_snake_case, unused_mut, dead_code)]
mod models;
mod dtos;
mod error;
mod state;
mod routes;

use axum::Router;
use sqlx::PgPool;
use std::{fs, path::PathBuf};
use tokio::net::TcpListener;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL is not set"))?;

    let pool = PgPool::connect(&database_url).await?;

    // Ensure uploads directory exists
    let upload_dir = PathBuf::from("uploads");
    fs::create_dir_all(&upload_dir)?;

    let state = AppState { pool, upload_dir };
    let app = routes::router(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}