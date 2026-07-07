use askama::Template;
use axum::Router;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum_extra::{TypedHeader, headers};
use std::time;

use crate::database::db;
use crate::{Content, Error, Note, assets};

async fn extra_8h() -> Result<(), Box<dyn std::error::Error>> {
    let db = db().await?;

    const SCHEMA: &str = r#"
        CREATE TABLE IF NOT EXISTS extra_8h (
            id TEXT PRIMARY KEY,
            content TEXT,
            ts BIGINT
        );
        "#;

    sqlx::query(SCHEMA).execute(db).await?;
    Ok(())
}

async fn reader_8h(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Result<impl IntoResponse, Error> {
    let db = db().await?;

    let ts = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        - 28800;

    const QUERY: &str = "SELECT content FROM extra_8h WHERE id = '28800' AND ts >= $1";

    let content: String = sqlx::query_scalar(QUERY)
        .bind(ts)
        .fetch_optional(db)
        .await?
        .unwrap_or_default();

    let note = Note {
        id: "".to_string(),
        content,
    };

    const CLI: [&str; 2] = ["curl", "wget"];
    let is_cli = CLI.iter().any(|agent| user_agent.as_str().contains(agent));

    if is_cli {
        Ok((
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            note.content,
        )
            .into_response())
    } else {
        let html = note.render()?;
        Ok(Html(html).into_response())
    }
}

async fn writer_8h(Content(content): Content) -> Result<impl IntoResponse, Error> {
    let db = db().await?;

    let ts = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    const QUERY: &str = r#"
            INSERT INTO extra_8h (id, content, ts) VALUES ('28800', $1, $2)
            ON CONFLICT(id) DO UPDATE
            SET content = excluded.content, ts = excluded.ts
            "#;

    sqlx::query(QUERY)
        .bind(&content)
        .bind(ts)
        .execute(db)
        .await?;

    Ok(StatusCode::OK)
}

pub async fn extra_init() -> Result<(), Box<dyn std::error::Error>> {
    extra_8h().await
}

pub fn extra_router() -> Router {
    Router::new()
        .route("/ex/8h", get(reader_8h).post(writer_8h))
        .route("/ex/assets/{file}", get(assets))
}
