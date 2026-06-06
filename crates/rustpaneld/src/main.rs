mod auth;
mod config;
mod handlers;
mod i18n;
mod services;
mod state;
mod system_metrics;
mod templates;
mod updates;
mod web;

use anyhow::Context;
use config::DaemonConfig;
use state::AppState;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = DaemonConfig::from_env()?;
    let state = AppState::from_config(&config);
    let app = web::router(&config.base_path, state);

    let listener = TcpListener::bind(config.addr)
        .await
        .with_context(|| format!("failed to bind {}", config.addr))?;
    tracing::info!(addr = %config.addr, base_path = %config.base_path, "rustpaneld listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustpaneld=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
