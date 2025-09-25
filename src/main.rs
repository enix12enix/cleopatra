// src/main.rs
use axum::Router;
use std::net::SocketAddr;
use tokio;

mod db;
mod models;
mod routes;
mod config;
mod writer;
mod state;
mod auth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = init_state().await?;

    let writer_for_shutdown = state.writer.clone();

    // Build our application by composing routes
    let app = Router::new()
        .merge(routes::routes())
        .layer(axum::middleware::from_fn_with_state(
            std::sync::Arc::new(state.clone()),
            auth::jwt_auth_middleware
        ))
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

async fn init_state() -> Result<crate::state::AppState, Box<dyn std::error::Error>> {
    let config = config::Config::from_env()?;

    let (main_pool, writer_pool) = db::init_db(&config).await?;

    let writer = writer::start_writer(&config.writer, writer_pool).await;

    let auth_provider = if config.auth.enabled {
        let provider = auth::AuthProvider::new(&config)?;
        Some(std::sync::Arc::new(provider))
    } else {
        None
    };

    let state = crate::state::AppState { 
        pool: main_pool, 
        writer,
        auth_provider,
    };

    Ok(state)
}

// flush data before shutdown
async fn shutdown_signal(writer: writer::Writer) {
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("Failed to listen for shutdown signal: {}", e);
        return;
    }
    println!("Shutdown signal received, flushing writer...");
    writer.shutdown();
}
