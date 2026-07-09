use sqlx::ConnectOptions;
use std::str::FromStr;
use tokio::sync::OnceCell;

#[cfg(feature = "serverless")]
pub mod lib {
    use super::*;
    use sqlx::{
        PgPool,
        postgres::{PgConnectOptions, PgPoolOptions},
    };
    use std::time::Duration;

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
}

#[cfg(feature = "server")]
pub mod lib {
    use super::*;
    use sqlx::{
        SqlitePool,
        sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    };
    use std::time::Duration;

    static DB: OnceCell<SqlitePool> = OnceCell::const_new();

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

        SqlitePoolOptions::new()
            .max_connections(1)
            .min_connections(0)
            .idle_timeout(Duration::from_secs(30))
            .connect_with(options)
            .await
    }

    pub async fn db() -> Result<&'static SqlitePool, sqlx::Error> {
        DB.get_or_try_init(db_backend).await
    }
}
