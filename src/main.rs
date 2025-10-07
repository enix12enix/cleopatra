// src/main.rs
use axum::Router;
use std::{net::SocketAddr, sync::Arc};
use tokio;

use crate::background::{scheduler::start_scheduler, writer::WriterManager};

mod database;
mod models;
mod routes;
mod config;
mod background;
mod state;
mod auth;
mod error;
mod suggestion;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build app state
    let state = state::init_state().await?;
    let state_arc = Arc::new(state);
    let writer_for_shutdown = state_arc.writer_manager.clone();

    // Start scheduler
    start_scheduler(Arc::clone(&state_arc)).await?;

    // Build app with routers and middleware
    let app = Router::new()
        .merge(routes::routes(&state_arc))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state_arc),
            auth::jwt_auth_middleware, // expects State<Arc<AppState>>
        ))
        .layer(axum::middleware::from_fn(error::handle_unexpected_errors))
        .with_state((*state_arc).clone());

    // Run application with graceful shutdown
    let config = &state_arc.config;
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    println!("Listening on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .with_graceful_shutdown(shutdown_signal(writer_for_shutdown))
        .await?;

    Ok(())
}

// Flush data before shutdown
async fn shutdown_signal(writer_manager: Arc<WriterManager>) {
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("Failed to listen for shutdown signal: {}", e);
        return;
    }
    println!("Shutdown signal received, flushing writer...");
    writer_manager.shutdown_all();
}
