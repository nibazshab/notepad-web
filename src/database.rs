use sqlx::ConnectOptions;
use std::str::FromStr;
use tokio::sync::OnceCell;

#[cfg(feature = "serverless")]
use {
    sqlx::{
        PgPool,
        postgres::{PgConnectOptions, PgPoolOptions},
    },
    std::time::Duration,
};

#[cfg(feature = "serverless")]
static DB: OnceCell<PgPool> = OnceCell::const_new();

#[cfg(feature = "serverless")]
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

#[cfg(feature = "serverless")]
pub async fn db() -> Result<&'static PgPool, sqlx::Error> {
    DB.get_or_try_init(db_backend).await
}

#[cfg(feature = "server")]
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
};

#[cfg(feature = "server")]
static DB: OnceCell<SqlitePool> = OnceCell::const_new();

#[cfg(feature = "server")]
async fn db_backend() -> Result<SqlitePool, sqlx::Error> {
    let dir = {
        let mut path = std::env::current_exe()?;
        path.pop();
        path.display().to_string()
    };

    let db_str = std::path::Path::new(format!("sqlite:{dir}").as_str())
        .join("note.db")
        .display()
        .to_string();

    let options = SqliteConnectOptions::from_str(&db_str)?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .create_if_missing(true)
        .log_statements(log::LevelFilter::Off);

    Ok(SqlitePool::connect_with(options).await?)
}

#[cfg(feature = "server")]
pub async fn db() -> Result<&'static SqlitePool, sqlx::Error> {
    DB.get_or_try_init(db_backend).await
}
