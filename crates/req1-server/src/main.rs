use std::net::SocketAddr;

use axum::http::{HeaderValue, Method};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tokio::net::TcpSocket;
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use req1_server::config::Config;
use req1_server::middleware;
use req1_server::routes;
use req1_server::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "req1_server=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    tracing::info!("Connecting to database...");
    let db = Database::connect(&config.database_url).await?;

    tracing::info!("Running migrations...");
    migration::Migrator::up(&db, None).await?;

    let state = AppState {
        db,
        config: config.clone(),
    };

    let cors = build_cors_layer(&config);

    let mut app = routes::router()
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(middleware::cache_control));

    if let Some(ref dir) = config.static_dir {
        let path = std::path::Path::new(dir);
        if path.is_dir() {
            let index = path.join("index.html");
            let serve = ServeDir::new(dir).not_found_service(ServeFile::new(index));
            app = app.fallback_service(serve);
            tracing::info!("Serving static files from {dir}");
        } else {
            tracing::warn!("STATIC_DIR={dir} is not a directory, skipping static file serving");
        }
    }

    let addr: SocketAddr = config.listen_addr.parse()?;
    let socket = TcpSocket::new_v4()?;
    socket.set_reuseaddr(true)?;
    socket.bind(addr)?;
    let listener = socket.listen(1024)?;
    tracing::info!("Listening on {}", config.listen_addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn build_cors_layer(config: &Config) -> CorsLayer {
    let methods = vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
    ];

    match config.cors_origin.as_deref() {
        None | Some("*") => CorsLayer::permissive(),
        Some(origins) => {
            let parsed: Vec<HeaderValue> = origins
                .split(',')
                .filter_map(|o| o.trim().parse().ok())
                .collect();
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(parsed))
                .allow_methods(methods)
                .allow_headers(Any)
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        let _ = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down gracefully");
        }
        () = terminate => {
            tracing::info!("Received SIGTERM, shutting down gracefully");
        }
    }
}
