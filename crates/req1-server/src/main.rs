use std::net::SocketAddr;

use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tokio::net::TcpSocket;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use req1_server::config::Config;
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

    let app = routes::router()
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = config.listen_addr.parse()?;
    let socket = TcpSocket::new_v4()?;
    socket.set_reuseaddr(true)?;
    socket.bind(addr)?;
    let listener = socket.listen(1024)?;
    tracing::info!("Listening on {}", config.listen_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
