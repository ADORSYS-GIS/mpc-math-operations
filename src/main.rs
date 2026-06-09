mod graphics;
mod server;
mod tools;

use axum::Router;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, session::local::LocalSessionManager, tower::StreamableHttpService,
};
use server::MathServer;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".into())
        .parse()?;

    let mut config = StreamableHttpServerConfig::default().with_stateful_mode(true);
    // rmcp's DNS-rebinding protection allows only loopback Host headers by default.
    // Extend it with any hosts in ALLOWED_HOSTS (comma-separated) so the server can
    // be reached by its container/service name. A bare host matches any port, so
    // "math" covers "math:3000".
    if let Ok(extra) = std::env::var("ALLOWED_HOSTS") {
        let mut hosts = config.allowed_hosts.clone();
        hosts.extend(
            extra
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
        );
        config = config.with_allowed_hosts(hosts);
    }

    let service = StreamableHttpService::new(
        || Ok(MathServer::new()),
        Arc::new(LocalSessionManager::default()),
        config,
    );

    let app = Router::new().nest_service("/mcp", service);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("mcp-math-operations listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
