// src/main.rs
use axum::Router;
use std::net::SocketAddr;
use tokio;

mod db;
mod models;
mod routes;
mod config;
mod writer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = config::Config::from_env()?;

    // Initialize the database pools (main pool for routes, writer pool for writer)
    let (main_pool, writer_pool) = db::init_db(&config).await?;

    // Start the writer with the dedicated connection pool
    let writer = writer::start_writer(&config.writer, writer_pool).await;

    // Clone for shutdown handling
    let writer_for_shutdown = writer.clone();

    // Create application state
    let state = models::AppState { pool: main_pool, writer };

    // Build our application by composing routes
    let app = Router::new()
        .merge(routes::routes())
        .with_state(state);

    // Run our application with graceful shutdown
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    println!("Listening on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .with_graceful_shutdown(shutdown_signal(writer_for_shutdown))
        .await?;

    Ok(())
}

async fn shutdown_signal(writer: writer::Writer) {
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("Failed to listen for shutdown signal: {}", e);
        return;
    }
    println!("Shutdown signal received, flushing writer...");
    writer.shutdown();
}
