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
use tracing_subscriber;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "rust_dms=debug,axum=info".into())
    )
    .init();

    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL is not set"))?;

    let pool = PgPool::connect(&database_url).await?;

    // Ensure uploads directory exists
    let upload_dir = PathBuf::from("uploads");
    fs::create_dir_all(&upload_dir)?;

    // Build OpenDAL operator for local filesystem storage rooted at `uploads/`.
    //
    // Old manual filesystem approach:
    // let stored_file_name = format!("{}_{}", Uuid::new_v4(), file_name);
    // let stored_path = upload_dir.join(&stored_file_name);
    // fs::write(&stored_path, &file_bytes)?;
    //
    // New approach: use OpenDAL `Operator` with an FS service.
    let mut builder = opendal::services::Fs::default();
    // Use the uploads directory as the root for all storage operations.
    builder = builder.root(&upload_dir.to_string_lossy());
    let storage = opendal::Operator::new(builder)?.finish();

    let state = AppState { pool, upload_dir, storage };
    let app = routes::router(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}