use sqlx::ConnectOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqliteSynchronous};
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;

use crate::{init, router};

static DB: OnceCell<SqlitePool> = OnceCell::const_new();

async fn db_backend() -> Result<SqlitePool, sqlx::Error> {
    let dir = {
        let mut path = env::current_exe()?;
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

pub async fn db() -> Result<&'static SqlitePool, sqlx::Error> {
    DB.get_or_try_init(db_backend).await
}

#[tokio::main]
pub async fn app() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1).peekable();
    let mut port: Option<u16> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" | "-p" => {
                port = args.next().map(|v| v.parse::<u16>()).transpose()?;
            }
            _ => {
                return Err(format!("unknown argument: {arg}").into());
            }
        }
    }

    let _port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let port = port.unwrap_or(_port);

    init().await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    let router = router().into_make_service_with_connect_info::<SocketAddr>();

    axum::serve(listener, router).await?;
    Ok(())
}
