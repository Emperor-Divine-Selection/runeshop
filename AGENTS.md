# runeshop

基于 Rust + Axum 的电商项目（学习项目，边学边写）。

## 技术栈

- Rust（edition 2024）+ Axum（Web 框架）
- PostgreSQL 17（主数据库）
- Valkey 8（Redis 兼容，用于缓存 / 会话 / 限流）
- 数据库访问层：sea-orm（ORM，底层由 sqlx 驱动；docker compose 提供数据库服务，与 ORM 选择无关）

## 本地环境（docker compose）

| 服务 | 镜像 | 端口 | 说明 |
|---|---|---|---|
| postgres | `postgres:17-alpine` | 5432 | 用户/密码/库：`runeshop` / `runeshop` / `runeshop` |
| valkey | `valkey/valkey:8-alpine` | 6379 | 无密码 |

- 启动：`docker compose up -d`　查看：`docker compose ps`
- 数据卷：`pgdata`、`valkeydata`（`docker compose down -v` 会删数据，慎用）
- 两个服务都配置了 healthcheck

## 进度

### ✅ 已完成

- [x] `docker-compose.yml` 亲手编写（经过多轮报错修复）
- [x] 镜像拉取成功（postgres:17-alpine、valkey/valkey:8-alpine）
- [x] Zed 全局设置：保存自动格式化（`format_on_save: "on"`）

### 🚧 下一步

- [ ] `docker compose up -d` 验证两个服务 healthy
- [ ] 初始化 Cargo 依赖（axum / tokio / sqlx / redis / serde / dotenv / tower-http）
- [ ] `main.rs` 骨架 + `/health` 接口（顺带验证 PG 连接 + Valkey ping）
- [ ] `.env` + 配置管理
- [ ] sqlx migration（建表）
- [ ] 业务建模（用户 / 商品 / 订单）

## 踩坑记录（重要教训）

1. **Docker 镜像名**：官方镜像裸名（`postgres`），第三方要带命名空间（`valkey/valkey`），大小写敏感
2. **compose 顶层结构**：`services` / `volumes` / `networks` 是兄弟，全部顶格；`ports` / `volumes` / `healthcheck` 是服务的子键，与 `environment` 同级
3. **卷名一致性**：服务内引用和顶层声明必须一字不差
4. **YAML**：缩进用空格不用 Tab；键名大小写敏感
5. **国内网络拉镜像失败**（`EOF` / `failed to fetch anonymous token`）= 网络问题，需配镜像加速器或代理，不是配置错误
6. **JSON 顶层结构**：不允许裸 `{...}` 对象块，设置必须是 `"键": 值`（否则报 `Property expected`）

## 约定

- 镜像名 / 服务名 / 容器名一律小写
- 本地开发凭据（`runeshop`/`runeshop`）仅限本地，上线前必须更换为环境变量注入
- 代码保持格式化后再保存（Zed 已开启自动格式化）
