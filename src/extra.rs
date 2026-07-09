use axum::Router;
use axum::routing::get;

mod extra_8h {
    use askama::Template;
    use axum::extract::Path;
    use axum::http::{StatusCode, header};
    use axum::response::{Html, IntoResponse};
    use axum_extra::{TypedHeader, headers};
    use std::time;

    use crate::database::lib::db;
    use crate::{Content, Error, Note};

    pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
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

    pub async fn reader(
        TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    ) -> Result<impl IntoResponse, Error> {
        let db = db().await?;

        let ts = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
            - 28800;

        const SS: &str = "SELECT content FROM extra_8h WHERE id = '28800' AND ts >= $1";

        let content: String = sqlx::query_scalar(SS)
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

    pub async fn writer(Content(content): Content) -> Result<impl IntoResponse, Error> {
        let db = db().await?;

        let ts = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        const SS: &str = r#"
            INSERT INTO extra_8h (id, content, ts) VALUES ('28800', $1, $2)
            ON CONFLICT(id) DO UPDATE
            SET content = excluded.content, ts = excluded.ts
            "#;

        sqlx::query(SS).bind(&content).bind(ts).execute(db).await?;

        Ok(StatusCode::OK)
    }

    pub async fn assets(Path(file): Path<String>) -> impl IntoResponse {
        crate::assets(Path(file)).await
    }
}

mod extra_message_board {
    use askama::Template;
    use axum::extract::Query;
    use axum::http::StatusCode;
    use axum::response::{Html, IntoResponse};
    use serde::Deserialize;
    use std::time;

    use crate::database::lib::db;
    use crate::{Content, Error};

    #[derive(Debug, Template)]
    #[template(path = "message.html")]
    struct Message {
        list: String,
        pages: String,
    }

    #[derive(Deserialize)]
    pub struct Params {
        page: Option<i64>,
    }

    pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
        let db = db().await?;

        const SCHEMA: &str = r#"
        CREATE TABLE IF NOT EXISTS extra_message_board (
            id INT PRIMARY KEY,
            message TEXT
        );
        "#;

        sqlx::query(SCHEMA).execute(db).await?;
        Ok(())
    }

    pub async fn reader(Query(params): Query<Params>) -> Result<impl IntoResponse, Error> {
        let page = params.page.unwrap_or(1).max(1);
        let record = 100;
        let offset = (page - 1) * record;

        let db = db().await?;

        const SS: &str =
            "SELECT message FROM extra_message_board ORDER BY id DESC LIMIT $1 OFFSET $2";

        let rows: Vec<Vec<u8>> = sqlx::query_scalar(SS)
            .bind(record)
            .bind(offset)
            .fetch_all(db)
            .await?;

        let mut list = String::new();
        for hex_string in rows {
            let bytes = hex::decode(hex_string).unwrap_or_default();
            let msg = String::from_utf8(bytes).unwrap_or_default();
            let txt = askama_escape::escape(&msg, askama_escape::Html).to_string();
            list.push_str(&format!("<li>{txt}</li>"));
        }

        let n = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM extra_message_board")
            .fetch_one(db)
            .await?;

        let mut pages = String::new();
        let g = ((n as f64) / (record as f64)).ceil() as i64;

        if g > 1 {
            pages.push_str("页码 ");
            for i in 1..=g {
                if i == page {
                    pages.push_str(&format!("<span>{i}</span> "));
                } else {
                    pages.push_str(&format!("<a href='?page={i}'>{i}</a> "));
                }
            }
        }

        let txt = Message { list, pages }
            .render()
            .map_err(|e| Error::BadRequest(e.to_string()))?;

        Ok(Html(txt))
    }

    pub async fn writer(Content(content): Content) -> Result<impl IntoResponse, Error> {
        let db = db().await?;

        let ts = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let hex_string = hex::encode(content);

        const SS: &str = "INSERT INTO extra_message_board (id, message) VALUES ($1, $2)";

        sqlx::query(SS)
            .bind(ts)
            .bind(hex_string)
            .execute(db)
            .await?;

        Ok(StatusCode::OK)
    }
}

pub async fn extra_init() -> Result<(), Box<dyn std::error::Error>> {
    extra_8h::init().await?;
    extra_message_board::init().await
}

pub fn extra_router() -> Router {
    Router::new()
        .route("/ex/8h/", get(extra_8h::reader).post(extra_8h::writer))
        .route("/ex/8h/assets/{file}", get(extra_8h::assets))
        .route(
            "/ex/msg/",
            get(extra_message_board::reader).post(extra_message_board::writer),
        )
}
