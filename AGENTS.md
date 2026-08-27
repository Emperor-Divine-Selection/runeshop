2# runeshop

基于 Rust + Axum 的电商项目（学习项目，边学边写）。

## 技术栈

- Rust（edition 2024）+ Axum（Web 框架）
- PostgreSQL 17（主数据库，docker compose 提供）
- Valkey 8（Redis 兼容，缓存 / 会话 / 限流，docker compose 提供）
- 数据库访问层：sea-orm（ORM，底层由 sqlx 驱动）
- Valkey 客户端：`valkey` crate（官方库，支持 `valkey://` scheme）

## 本地环境（docker compose）

| 服务 | 镜像 | 端口 | 说明 |
|---|---|---|---|
| postgres | `postgres:17-alpine` | 5432 | 用户/密码/库：`runeshop` / `runeshop` / `runeshop` |
| valkey | `valkey/valkey:8-alpine` | 6379 | 无密码（仅本地） |

- 启动：`docker compose up -d`　查看：`docker compose ps`
- 数据卷：`pgdata`、`valkeydata`（`docker compose down -v` 会删数据，慎用）
- 两个服务都配置了 healthcheck
- 应用跑在宿主机，连接用 `localhost:5432` / `localhost:6379`（走宿主机端口映射）

## 项目结构

```
runeshop/
├── docker-compose.yml      # postgres + valkey
├── .env                    # 本地配置（不入 git）
├── AGENTS.md
├── Cargo.toml              # workspace：[workspace] members = ["migration"]
├── migration/              # 独立 crate：数据库迁移
│   ├── Cargo.toml          # sea-orm-migration（runtime-tokio-rustls + sqlx-postgres）
│   └── src/
│       ├── lib.rs          # Migrator 注册表
│       └── m20220101_000001_create_table.rs  # users 表迁移
└── src/
    ├── main.rs             # 入口：连接 DB + Axum 服务
    ├── config/
    │   └── mod.rs          # Config 结构体 + from_env 加载
    └── model/
        ├── mod.rs          # entity 模块声明
        ├── prelude.rs      # 别名（Entity as Users）
        └── users.rs        # users 实体（sea-orm-cli 自动生成）
```

## 进度

### ✅ 已完成

- [x] `docker-compose.yml` 亲手编写（经过多轮报错修复）
- [x] 镜像拉取 + `docker compose up -d` 两个容器启动成功
- [x] Zed 全局设置：保存自动格式化（`format_on_save: "on"`）
- [x] Cargo 依赖：axum / tokio / sea-orm / valkey / serde / serde_json / dotenvy（全部用 `cargo add`）
- [x] 配置层：`.env` + `src/config/mod.rs`（目录模块写法，`Config::new()` 加载）
- [x] sea-orm migration：`migrate init` + workspace 整合 + `users` 表迁移成功（id/username/email/password_hash/created_at/updated_at）
- [x] entity 生成：`sea-orm-cli generate entity -o src/model`（目录名可自定义，不必须叫 entity）
- [x] 数据库连接验证：`Database::connect` + `Entity::find().count()` 跑通（users 表 0 行）

### 🚧 进行中 / 下一步

- [ ] `main.rs` 完整化：Axum + 共享状态（AppState 传数据库连接）+ `/health` 接口
- [ ] git 推送到 GitHub（HTTPS + token 认证，`Emperor-Divine-Selection/runeshop`）
- [ ] valkey 连接验证（`PING` → `PONG`）
- [ ] 业务建模（商品 / 订单 / 订单项）

## 踩坑记录（重要教训）

### 基础设施

1. **Docker 镜像名**：官方镜像裸名（`postgres`），第三方带命名空间（`valkey/valkey`），大小写敏感
2. **compose 顶层结构**：`services` / `volumes` / `networks` 是兄弟，全部顶格；`ports` / `volumes` / `healthcheck` 是服务的子键，与 `environment` 同级
3. **卷名一致性**：服务内引用和顶层声明必须一字不差
4. **YAML**：缩进用空格不用 Tab；键名大小写敏感
5. **国内网络拉镜像失败**（`EOF` / `failed to fetch anonymous token`）= 网络问题，需配镜像加速器或代理

### Rust / 配置

6. **`dotenv().ok()` 必须调用**：`.env` 只是磁盘文件，不调用就不会加载进环境变量，`env::var` 读到空 → `expect` panic
7. **`.env` 键名与代码一致**：`env::var("VALKEY_URL")` 与 `.env` 里的键必须一字不差（引用与声明一致原则）
8. **`cargo add` 代替手写依赖**：手写 `version = "latest"` / 乱用 `full` feature 必翻车；版本号是语义化版本（数字开头），features 每个包各不相同
9. **Rust 模块两种写法等价**：`src/config.rs` 与 `src/config/mod.rs`，`mod config;` 都能找到
10. **JSON 顶层结构**：不允许裸 `{...}` 对象块，设置必须是 `"键": 值`（否则报 `Property expected`）

### Git

11. **没有 commit 就没有分支**：分支是"指向提交的指针"，`git push` 报 `src refspec master does not match any` 说明还没 commit，先 `git add .` + `git commit`
12. **GitHub 认证用 token 不用密码**：密码认证已被禁用，需 Personal Access Token（勾选 `repo` 权限），`Password` 提示处粘贴 token；Linux 终端粘贴用 `Ctrl+Shift+V`
13. **sea-orm-cli 的工作目录**：默认找 `./migration`，必须在项目根目录运行；`migrate up` 要 `-u` 传 DATABASE_URL（CLI 不读 .env）
14. **Cargo workspace**：`migration/` 是独立 crate，根 Cargo.toml 需 `[workspace] members = ["migration"]`，否则 rust-analyzer 代码提示失灵
15. **psql 分页器**：长输出显示 `--More--`，`q` 退出、空格翻页；可用 `-P pager=off` 关闭
16. **连接超时排查顺序**：容器在跑吗（`docker compose ps`）→ 端口通吗 → URL 对吗

## 约定

- 镜像名 / 服务名 / 容器名一律小写
- 依赖一律用 `cargo add`，不手写 Cargo.toml 版本
- 本地开发凭据（`runeshop`/`runeshop`）仅限本地，上线前必须更换为环境变量注入
- `.env` 不入 git（`.gitignore` 已加 `*.env`）
- 代码保持格式化后再保存（Zed 已开启自动格式化）
