## 说明

本质上这只是一个练手项目，用来学习用的

- api

.|.
-|-
/xx | get/post/put
/ | post
/d/xx | get

- 环境变量

.|.
-|-
DATABASE_URL | 数据库连接字符串
BASE_URL | 网站 URL

- 编译

```sh
cargo check
cargo fmt --all -- --check
cargo clippy -- -D warnings

cargo run --no-default-features --features server
```

## 参考

- pereorga/minimalist-web-notepad
