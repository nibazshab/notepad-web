## 说明

本质上这只是一个练手项目，用来学习用的

- api

URL|method
-|-
/xx | get/post/put
/d/xx | get

- 存储

数据存放在 Upstash Redis，走 REST over HTTPS，不需要建表，也没有数据库连接数限制。
笔记 id 直接作为 Redis key，值为正文；key 不存在时返回空字符串，与原来的行为一致。

- 环境变量

name|info
-|-
KV_REST_API_URL | Upstash REST 地址，也可用 UPSTASH_REDIS_REST_URL
KV_REST_API_TOKEN | 具备读写权限的 REST token，也可用 UPSTASH_REDIS_REST_TOKEN

写入必须使用读写权限的 token，不能填 KV_REST_API_READ_ONLY_TOKEN。
KV_URL / REDIS_URL 是 TCP 连接串，本项目走 REST，无需配置。

- 编译

```sh
cargo fmt --all
cargo clippy -- -D warnings
cargo test --lib
```

## 参考

- pereorga/minimalist-web-notepad
