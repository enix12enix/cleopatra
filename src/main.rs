// src/main.rs
use axum::Router;
use std::net::SocketAddr;
use tokio;

mod db;
mod models;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the database
    let pool = db::init_db().await?;

    // Build our application by composing routes
    let app = Router::new()
        .merge(routes::routes())
        .with_state(pool.clone());

    // Run our application
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}