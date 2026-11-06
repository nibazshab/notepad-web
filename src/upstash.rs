use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::fmt::Display;
use tokio::sync::OnceCell;

use crate::{Error, Result};

static CONN: OnceCell<Conn> = OnceCell::const_new();

struct Conn {
    client: Client, // 内部自带连接池，跨请求复用
    url: String,    // Upstash REST 端点，来自环境变量
    token: String,  // 读写权限的 REST token，绝不写进源码
}

// 零大小类型：本身不存任何数据，只是 NoteStore trait 的实现载体。
// 真正的状态都在上面的 REST_CONFIG 里，所以它可以随意 Clone / 传进 axum 的 State。
#[derive(Clone)]
pub struct Upstash;

fn err_format(message: impl Display) -> Error {
    Error::Database(message.to_string())
}

#[derive(Debug, Deserialize)]
struct Command {
    #[serde(default)]
    result: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Rest {
    Batch(Vec<Command>),
    Single(Command),
}

impl Rest {
    fn into_reply(self) -> Result<Command> {
        let reply = match self {
            Rest::Single(reply) => reply,

            Rest::Batch(mut replies) => replies
                .pop()
                .ok_or_else(|| err_format("Upstash returned no command reply"))?,
        };

        if let Some(error) = reply.error.as_deref() {
            return Err(err_format(error));
        }

        Ok(reply)
    }
}

fn first_env(names: &[&str]) -> Result<String> {
    names
        .iter()
        .find_map(|name| env::var(name).ok())
        .ok_or_else(|| err_format(format!("none of {} is set", names.join(" / "))))
}

// 懒加载 REST 配置：第一次请求时才读环境变量、建客户端；之后所有请求共用这一份。
// 用 get_or_try_init 而不是 unwrap —— 缺配置时返回 500，而不是让函数 panic。
async fn rest_config() -> Result<&'static Conn> {
    CONN.get_or_try_init(|| async {
        let url = first_env(&["KV_REST_API_URL"])?.trim_end_matches('/').to_owned();

        let token = first_env(&["KV_REST_API_TOKEN"])?;

        Ok(Conn {
            client: Client::new(),
            url,
            token,
        })
    })
    .await
}

// 执行一条 Redis 命令。Upstash REST 接受 JSON 数组，例如 ["SET", "abcd", "正文"]。
//
// 特意不用 /set/abcd 这种把值塞进 URL 的写法：笔记正文可能是空串、
// 含换行或各种特殊字符，走 JSON body 由 serde 负责转义，
// 不会出现「空值被当成没传参数」或 URL 编码出错的问题。
async fn execute_redis_command(config: &Conn, command: &[&str]) -> Result<Command> {
    let response: Rest = config
        .client
        .post(&config.url) // POST 到 REST 根地址
        .bearer_auth(&config.token) // 等价于 Authorization: Bearer <token>
        .json(command) // 序列化成 JSON 数组，并自动设置 Content-Type
        .send()
        .await
        .map_err(err_format)? // 网络层错误
        .error_for_status() // 把 4xx/5xx 也变成错误，否则会被当成正常响应去解析
        .map_err(err_format)?
        .json() // 把响应体解析成 RestResponse
        .await
        .map_err(err_format)?;

    response.into_reply()
}

impl crate::NoteStore for UpstashRest {
    // Redis GET：取不到（result 为 null）就返回空字符串，
    // 和旧版 PG 的 fetch_optional(...).unwrap_or_default() 行为一致。
    async fn load(&self, id: &str) -> Result<String> {
        let config = rest_config().await?;
        let command_reply = execute_redis_command(config, &["GET", id]).await?;
        Ok(command_reply.result.unwrap_or_default())
    }

    // Redis SET：整值覆盖。等价于旧版 SQL 的 ON CONFLICT DO UPDATE，
    // 不需要先查 key 是否存在；前端每停顿 500ms 就会整篇提交一次。
    async fn save(&self, id: &str, content: &str) -> Result<()> {
        let config = rest_config().await?;
        execute_redis_command(config, &["SET", id, content]).await?;
        Ok(())
    }
}
