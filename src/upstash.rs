use reqwest::Client;
use serde::Deserialize;
use std::fmt::Display;
use tokio::sync::OnceCell;

use crate::{Database, Error, Result};

#[derive(Clone)]
pub struct Upstash;

static CONN: OnceCell<Conn> = OnceCell::const_new();

struct Conn {
    client: Client,
    url: String,
    token: String,
}

#[derive(Debug, Deserialize)]
struct Req {
    #[serde(default)]
    result: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Rest {
    Batch(Vec<Req>),
    Single(Req),
}

impl Rest {
    fn one(self) -> Result<Req> {
        let reply = match self {
            Rest::Single(reply) => reply,
            Rest::Batch(mut replies) => replies
                .pop()
                .ok_or_else(|| err("Upstash returned no command reply"))?,
        };

        if let Some(e) = reply.error.as_deref() {
            return Err(err(e));
        }

        Ok(reply)
    }
}

async fn conn() -> Result<&'static Conn> {
    CONN.get_or_try_init(|| async {
        let url = std::env::var("KV_REST_API_URL").map_err(err)?;
        let token = std::env::var("KV_REST_API_TOKEN").map_err(err)?;

        Ok(Conn {
            client: Client::new(),
            url,
            token,
        })
    })
    .await
}

async fn call(c: &Conn, cmd: &[&str]) -> Result<Req> {
    let rest: Rest = c
        .client
        .post(&c.url)
        .bearer_auth(&c.token)
        .json(cmd)
        .send()
        .await
        .map_err(err)?
        .error_for_status()
        .map_err(err)?
        .json()
        .await
        .map_err(err)?;

    rest.one()
}

fn err(message: impl Display) -> Error {
    Error::Database(message.to_string())
}

impl Database for Upstash {
    async fn read(&self, id: &str) -> Result<String> {
        let c = conn().await?;
        let r = call(c, &["GET", id]).await?;
        Ok(r.result.unwrap_or_default())
    }

    async fn write(&self, id: &str, content: &str) -> Result<()> {
        let c = conn().await?;
        call(c, &["SET", id, content]).await?;
        Ok(())
    }
}
