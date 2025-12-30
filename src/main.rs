#![allow(unused_imports, unused_variables, non_snake_case, unused_mut, dead_code)]
mod models;
mod dtos;
mod error;
mod state;
mod routes;
mod auth;
mod audit;
mod openapi;

use axum::Router;
use sqlx::PgPool;
use std::{fs, path::PathBuf};
use tokio::net::TcpListener;
use tracing_subscriber;
use tracing::{info, debug, warn};

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
    // let upload_dir = PathBuf::from("uploads");
    // fs::create_dir_all(&upload_dir)?;

    // Build OpenDAL operator for local filesystem storage rooted at `uploads/`.
    //
    // Old manual filesystem approach:
    // let stored_file_name = format!("{}_{}", Uuid::new_v4(), file_name);
    // let stored_path = upload_dir.join(&stored_file_name);
    // fs::write(&stored_path, &file_bytes)?;
    //
    // New approach: use OpenDAL `Operator` with an FS service.
    // let mut builder = opendal::services::Fs::default();
    // Use the uploads directory as the root for all storage operations.
    // builder = builder.root(&upload_dir.to_string_lossy());
    // let storage = opendal::Operator::new(builder)?.finish();

    let endpoint = std::env::var("SEAWEEDFS_ENDPOINT")
       .unwrap_or_else(|_| "http://localhost:8333".to_string());
    let access_key = std::env::var("SEAWEEDFS_ACCESS_KEY")
        .unwrap_or_else(|_| "".to_string());
    let secret_key = std::env::var("SEAWEEDFS_SECRET_KEY")
        .unwrap_or_else(|_| "".to_string());
    let bucket = std::env::var("SEAWEEDFS_BUCKET")
        .unwrap_or_else(|_| "dms-documents".to_string());
    
    let mut builder = opendal::services::S3::default();
    builder = builder
        .endpoint(&endpoint)
        .bucket(&bucket)
        .access_key_id(&access_key)
        .secret_access_key(&secret_key)
        .region("us-east-1");
        
    let storage = opendal::Operator::new(builder)?.finish();

    // Create bucket and warm up SeaweedFS S3 API connection
    // Retry until bucket is created and connection is established
    // Because without this it gives access denied first be sure the bucket exists then continue 
    info!("Initializing SeaweedFS bucket: {}", bucket);
    let mut retries = 10;
    let mut bucket_created = false;
    
    while retries > 0 && !bucket_created {
        // Try to create bucket via HTTP PUT request
        let bucket_url = format!("{}/{}", endpoint, bucket);
        match reqwest::Client::new()
            .put(&bucket_url)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                if status.is_success() || status.as_u16() == 409 {
                    // 200/201 = created, 409 = already exists (both are OK)
                    info!("Bucket '{}' is ready (status: {})", bucket, status);
                    bucket_created = true;
                    
                    // Now warm up the OpenDAL connection by trying to list/stat
                    match storage.stat("/").await {
                        Ok(_) => {
                            info!("SeaweedFS storage connection established and ready");
                            break;
                        }
                        Err(e) => {
                            debug!("OpenDAL connection not ready yet, but bucket exists: {}", e);
                            // Bucket exists, connection will work on first real request
                            break;
                        }
                    }
                } else {
                    warn!("Failed to create bucket, status: {}", status);
                    retries -= 1;
                }
            }
            Err(e) => {
                retries -= 1;
                if retries > 0 {
                    warn!(
                        "SeaweedFS S3 API not ready yet (retries left: {}), error: {}",
                        retries, e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                } else {
                    warn!(
                        "Could not create bucket '{}' after retries, continuing anyway: {}",
                        bucket, e
                    );
                }
            }
        }
    }
    
    if !bucket_created {
        warn!("Bucket '{}' may not exist, uploads might fail on first request", bucket);
    }

    let state = AppState { pool, storage };
    let app = routes::router(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}