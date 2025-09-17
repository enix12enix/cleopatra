// src/main.rs
use axum::Router;
use std::net::SocketAddr;
use tokio;

mod db;
mod models;
mod routes;
mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = config::Config::from_env()?;

    // Initialize the database
    let pool = db::init_db(&config).await?;

    // Create application state
    let state = models::AppState { pool };

    // Build our application by composing routes
    let app = Router::new()
        .merge(routes::routes())
        .with_state(state);

    // Run our application
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    println!("Listening on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}