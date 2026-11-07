use reqwest::Client;
use serde::Deserialize;
use std::fmt::Display;

use crate::{Database, Error, Result};

#[derive(Clone)]
pub struct Upstash {
    client: Client,
    url: String,
    token: String,
}

impl Upstash {
    pub fn conn() -> Self {
        let url = std::env::var("KV_REST_API_URL").expect("缺少 KV_REST_API_URL");
        let token = std::env::var("KV_REST_API_TOKEN").expect("缺少 KV_REST_API_TOKEN");

        assert!(!url.is_empty(), "KV_REST_API_URL 不能为空");
        assert!(!token.is_empty(), "KV_REST_API_TOKEN 不能为空");

        Self {
            client: Client::new(),
            url,
            token,
        }
    }

    async fn call(&self, cmd: &[&str]) -> Result<Reply> {
        let rest: Rest = self
            .client
            .post(&self.url)
            .bearer_auth(&self.token)
            .json(cmd)
            .send()
            .await
            .map_err(into_err)?
            .error_for_status()
            .map_err(into_err)?
            .json()
            .await
            .map_err(into_err)?;

        rest.reply()
    }
}

#[derive(Debug, Deserialize)]
struct Reply {
    #[serde(default)]
    result: Option<String>,

    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Rest {
    Batch(Vec<Reply>),
    Single(Reply),
}

impl Rest {
    fn reply(self) -> Result<Reply> {
        let reply = match self {
            Rest::Single(reply) => reply,
            Rest::Batch(mut replies) => replies.pop().ok_or_else(|| into_err("Upstash Redis error"))?,
        };

        if let Some(e) = reply.error.as_deref() {
            return Err(into_err(e));
        }

        Ok(reply)
    }
}

impl Database for Upstash {
    async fn read(&self, id: &str) -> Result<String> {
        let r = self.call(&["GET", id]).await?;
        Ok(r.result.unwrap_or_default())
    }

    async fn write(&self, id: &str, content: &str) -> Result<()> {
        self.call(&["SET", id, content]).await?;
        Ok(())
    }
}

fn into_err(err: impl Display) -> Error {
    Error::Database(err.to_string())
}
