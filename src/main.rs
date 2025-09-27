// src/main.rs
use axum::Router;
use std::{net::SocketAddr, sync::Arc};
use tokio;

use crate::daemon::writer::WriterManager;

mod database;
mod models;
mod routes;
mod config;
mod daemon;
mod state;
mod auth;
mod error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build app state
    let state = state::init_state().await?;
    let writer_for_shutdown = state.writer_manager.clone();

    // Build app with routers
    let app = Router::new()
        .merge(routes::routes())
        .layer(axum::middleware::from_fn_with_state(
            std::sync::Arc::new(state.clone()),
            auth::jwt_auth_middleware
        ))
        .layer(axum::middleware::from_fn(error::handle_unexpected_errors))
        .with_state(state);

    // Run application with graceful shutdown
    let config = config::Config::from_env()?;
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    println!("Listening on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .with_graceful_shutdown(shutdown_signal(writer_for_shutdown))
        .await?;

    Ok(())
}



// flush data before shutdown
async fn shutdown_signal(writer_manager: Arc<WriterManager>) {
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("Failed to listen for shutdown signal: {}", e);
        return;
    }
    println!("Shutdown signal received, flushing writer...");
    writer_manager.shutdown_all();
}
