# migration-identity

`identity_db` 的**建库 + 建表**。既是部署步骤，也是本地开发命令。

## 跑

```sh
cargo run -p migration-identity
```

它按顺序做两件事：

1. **`ensure_database`** —— 连 `POSTGRES_URL_MAINTENANCE`（维护库 `runeshop_ops`），
   `SELECT 1 FROM pg_database` 查一下，缺 `identity_db` 才 `CREATE DATABASE`
2. **`Migrator::up`** —— 连 `POSTGRES_URL_IDENTITY`，把还没执行过的迁移跑完

两步都幂等，**可以反复跑**。部署时它就是一个「跑完退出」的容器，
应用用 `depends_on: migrate: { condition: service_completed_successfully }` 等它。

## 不做的事

- **不接命令行参数**。`up` / `down` / `fresh` / `status` 是 `sea_orm_migration::cli::run_cli` 的行为，
  我们没用它（因为要先 `ensure_database` 再迁移，`run_cli` 管不了前半段）。
  传了参数也会被忽略，直接跑一遍迁移。
- **迁移只向前**。本地要重来：

  ```sh
  docker compose exec postgres psql -U runeshop -d runeshop_ops -c 'DROP DATABASE identity_db'
  cargo run -p migration-identity
  ```

## 新增一张表

1. 在 `src/` 下建 `m20261113_000001_create_xxx.rs`（手写比 `migrate generate` 稳 —— 见 AGENTS.md 踩坑 120）
2. 在 `src/lib.rs` 的 `migrations()` 里注册 —— **不注册就不会跑，且一声不响**（踩坑 21）
3. `cargo run -p migration-identity`
4. 验收：`\d xxx` + `select * from seaql_migrations`

## 依赖为什么这么写

```toml
sea-orm-migration = { version = "2.0.3", features = ["runtime-tokio-rustls", "sqlx-postgres"] }
```

`sea-orm-cli` 生成的模板里这两个 feature 是**注释掉的**，等于没有异步运行时、没有数据库驱动：
`cargo check` 全绿，`cargo run` 一连库就挂（踩坑 121）。
