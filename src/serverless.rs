use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::OnceCell;
use tower::ServiceBuilder;
use vercel_runtime::axum::VercelLayer;

use crate::{init, router};

static DB: OnceCell<PgPool> = OnceCell::const_new();

async fn db_backend() -> Result<PgPool, sqlx::Error> {
    let db_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/postgres".to_string());

    let options = PgConnectOptions::from_str(&db_str)?.log_statements(log::LevelFilter::Off);

    PgPoolOptions::new()
        .max_connections(1)
        .min_connections(0)
        .idle_timeout(Duration::from_secs(30))
        .connect_with(options)
        .await
}

pub async fn db() -> Result<&'static PgPool, sqlx::Error> {
    DB.get_or_try_init(db_backend).await
}

#[tokio::main]
pub async fn app() -> Result<(), vercel_runtime::Error> {
    init().await.map_err(|e| e.to_string())?;

    let router = router();

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);

    vercel_runtime::run(app).await
}
