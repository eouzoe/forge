//! Entry point for the `forge-gateway` HTTP server.

use std::sync::Arc;

use forge_gateway::{pool::SandboxPool, routes::create_router};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr = std::env::var("FORGE_LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3456".to_owned());

    let pool = Arc::new(SandboxPool::new());
    let app = create_router(pool);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(addr = %addr, error = %e, "failed to bind");
            std::process::exit(1);
        }
    };

    info!(addr = %addr, "forge-gateway listening");

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!(error = %e, "server error");
        std::process::exit(1);
    }
}
