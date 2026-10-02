# runeshop

基于 Rust + Axum 的电商项目（学习项目，边学边写）。

> **2026-09-26 项目重启**
>
> 旧项目做到 17 张表 + 3 个 store（UserStore / WalletStore / MerchantStore），但**表铺得太开，"驾驭不了"** —— 改一行要同时顾 5 个 store，心智负担压垮进度。
>
> 改为**线性竖切**（一次只打通一条业务链路：接口 → store → 表），从零重建。
> **保留**：全部踩坑记录 + 命名规范 + 架构决策的演进理由。
> **丢弃**：17 张表、3 个 store 的旧实现（存档见文末）。

## 技术栈

- Rust（edition 2024）+ Axum 0.8（Web 框架）
- PostgreSQL 17（主数据库，docker compose 提供）
- Valkey 8（Redis 兼容，缓存 / 会话 / 限流，docker compose 提供）
- 数据库访问层：sea-orm 2.0（ORM，底层 sqlx 驱动）
- 密码哈希：argon2 0.6（纯 Rust，OWASP 当前首推）
- 缓存客户端：**fred 10.1**（异步，原生支持 Valkey 8）
- 日志：tracing + tower-http 的 `TraceLayer`

## 本地环境（docker compose）

| 服务 | 镜像 | 端口 | 说明 |
|---|---|---|---|
| postgres | `postgres:17-alpine` | 5432 | 用户/密码/库：`runeshop` / `runeshop` / `runeshop` |
| valkey | `valkey/valkey:8-alpine` | 6379 | 无密码（仅本地） |

- 启动：`docker compose up -d`　查看：`docker compose ps`
- 数据卷：`pgdata`、`valkeydata`（`docker compose down -v` 会删数据，慎用）
- 两个服务都配置了 healthcheck
- 应用跑在宿主机，连接用 `localhost:5432` / `localhost:6379`

### `.env` 键名（⚠️ 与旧项目不同）

```
POSTGRES_URL=postgres://runeshop:runeshop@localhost:5432/runeshop
VALKEY_URL=valkey://localhost:6379
```

- 旧项目用 `DATABASE_URL`，**新项目改叫 `POSTGRES_URL`**（字段名 `postgres_url` 比 `database_url` 精确）
- **代码里 `env::var("X")` 的 X 必须和这里一字不差**（见踩坑 7）
- `sea-orm-cli` 只认 `DATABASE_URL` 这个环境变量名 → 所有 CLI 命令**必须显式传 `-u`**

## 项目结构

```
runeshop/
├── docker-compose.yml      # postgres + valkey
├── .env                    # 本地配置（不入 git）
├── AGENTS.md
├── Cargo.toml              # workspace：[workspace] members = ["migration"]
├── migration/              # 独立 crate：数据库迁移
│   └── src/
│       ├── lib.rs          # Migrator + migrations() 列表（每个新迁移都要手动注册！）
│       ├── main.rs         # CLI 入口（migrate up/down 的执行体）
│       ├── m20220101_000001_create_table_users.rs
│       └── m20260926_055846_alter_users_avatar_bio_nullable.rs
└── src/
    ├── main.rs             # 入口：读 Config → 连库 → 起 Axum
    ├── config/
    │   └── mod.rs          # Config 结构体 + new()（内部调 dotenvy::dotenv()）
    ├── model/              # entity 层（sea-orm-cli 自动生成，不要手改）
    │   ├── mod.rs
    │   ├── prelude.rs      # 别名（Entity as Users 等）
    │   └── users.rs        # users 实体
    ├── store/              # 存库层：只管「数据怎么存」
    │   ├── mod.rs          # 只放 pub mod 声明，一行一个，不写逻辑
    │   ├── errors.rs       # StoreError（2 变体：NotFound / Technical）
    │   └── user/
    │       └── mod.rs      # users 表的方法
    ├── service/            # 业务层：只管「业务规则怎么判」
    │   ├── mod.rs
    │   ├── errors.rs       # ServiceError（5 变体，业务错误在这里产生）
    │   └── user.rs         # register：查重 + 编排 store
    ├── handler/            # HTTP 层：只管「JSON 怎么进出」
    │   ├── mod.rs
    │   ├── errors.rs       # ApiError（把 ServiceError 翻成 HTTP 状态码）
    │   └── register/
    │       └── mod.rs      # handler + Request/Response DTO
    ├── router/             # 路由层：只管「哪个 URL 走哪个 handler」
    │   └── mod.rs          # app(db) -> Router，组装全部路由 + 中间件
    └── cache/              # 缓存层：fred 封装（空壳，待建）
```

**依赖方向（单向向下，不可倒置）**：

```
main.rs（组合根：只管进程生命周期）
  ↓
router/   组装路由（知道所有 handler，不含业务）
  ↓
handler/  JSON 进出（DTO、脱敏、错误 → HTTP 状态码）
  ↓
service/  业务规则（查重、状态判定、跨 store 编排）
  ↓
store/    数据存取（哈希、SQL、列格式）
  ↓
model/    entity（sea-orm 自动生成）
```

**反过来的信号是某个内层模块 import 了外层的东西** —— 立刻警惕。

### ★ 层内组织的命名规律

| 位置 | 形态 | 例子 |
|---|---|---|
| **层的直属文件** | 扁平放，与 `mod.rs` 同级 | `service/errors.rs`、`store/errors.rs` |
| **按实体/操作分组的子集合** | 建子目录 | `store/user/`、`handler/register/` |
| **层本身是容器** | 直接建目录 | `router/`、`cache/` |

- `src/router.rs` 和 `src/router/mod.rs` **对 `mod router;` 完全等价**（见踩坑 9）—— 两种形态随时可互换，调用方零改动
- 目录名单数：`user/` 不是 `users/`，`register/` 不是 `registers/`
- **例外登记**：`cache/mod.rs` 是 0 字节空壳（与「用到才建」原则有出入）。已记为待建项 —— 写第一个缓存方法时填内容，或直接删掉目录

### 各层的划分依据

| 层 | 分法 | 理由 |
|---|---|---|
| `store/` `service/` | **按实体**（`user/` = users 表相关的所有操作） | 同一实体的存取和业务规则天然在一起 |
| `handler/` | **按操作**（`register/` = 注册这个接口） | 一个实体会有多个 handler（register / login / update_profile），按操作分更清晰 |
| `router/` | 暂时只有 `mod.rs`，未来按实体拆 | 路由是「URL → handler」的映射，实体多了再拆（用 `.merge()` 拼） |

## 架构决策

### ★ store 层：泛型 `ConnectionTrait`（方案乙）

store 方法**不持有 db**，连接靠泛型参数传入：

```rust
pub async fn find_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Users::Model>, StoreError>
```

**为什么**：sea-orm 的 `DatabaseConnection`（池）和 `DatabaseTransaction`（事务）是两个**完全不同的类型，彼此不能转换**。如果 store 方法签名写死 `&DatabaseConnection`，事务里就调不了它 —— 只能把查询代码重抄一遍（旧项目踩坑 28 的原罪）。用泛型后同一份代码两边都能跑：

```rust
UserStore::find_by_username(&state.db, "x").await   // 走池
UserStore::find_by_username(&txn, "x").await        // 走事务，同一份代码
```

**代价**（都已接受）：
- 编译期单态化，每种类型生成一份机器码 → 产物体积涨、编译变慢（sea-orm 自己就这么干）
- 泛型参数推不出时需显式标注 `::<DatabaseConnection>`
- **泛型挡不住"误传池"**（见踩坑 41），真正的防线是结构

**⚠️ 选过的其他方案，记录否决理由：**

| 方案 | 否决理由 |
|---|---|
| store 持有 db（`&self`） | 事务里调不了自己的方法，必须重抄查询代码 |
| store 方法收 `&AppState` | 层依赖倒置（store 是内层，却 import web 层的状态结构）；且 `state.db` 类型写死，事务能力全丢 |
| `&DatabaseConnection` 具体类型 | 最简单，但事务里用不了 → 重抄。**将来可低成本升级为泛型**（只改签名，调用方不动） |

### 目录 / 命名规范

| 东西 | 规范 | 正例 | 反例 |
|---|---|---|---|
| 目录、模块、文件 | `snake_case` | `store/user/mod.rs` | ~~`store/UserStore/`~~ |
| 类型（struct/enum/trait） | `UpperCamelCase` | `struct UserStore` | ~~目录也叫 UserStore~~ |
| 函数、方法、变量 | `snake_case` | `find_by_username` | |

- **目录名别 stutter**：`store::user::find_by_username()` 够了，`store::user_store::` 里 store 说了两遍 —— 父目录已经说明层次
- **目录名单数**：`user/` 而不是 `users/`（将来是 `user/addresses.rs`、`user/xp.rs`）
- **拆分时机**：一个 store 跨到第 2 张表才拆子文件，不为对称性拆。`pub mod` 保证对外路径 `store::user::` 在拆分前后**完全不变**
- **类型名和目录名是两件事**：`UserStore` 作类型名永远正确，勿拿去当目录名

### store 方法命名（沿用旧项目规范）

- `create_xxx`（凭空造本体）/ `create_for_宿主`（造附属物）
- `find_by_xxx`（单查返 `Option`）/ `list_xxx`（批量返 `Vec`，复数）
- `update_xxx_by_id` / `delete_by_xxx`（多变体时才带后缀）
- 全部方法补文档注释：**一句话：干什么 + 查不到什么行为**

## 进度

### ✅ 已完成（新项目）

- [x] `docker-compose.yml` + 容器启动（postgres 17 / valkey 8，均 healthy）
- [x] 依赖：`axum` / `tokio` / `sea-orm` / `serde` / `serde_json` / `dotenvy` / `argon2` / `fred` / `tracing-subscriber` / `tower-http`（全部用 `cargo add`）
- [x] `migration/` crate 初始化 + workspace 整合 + features 打开（`runtime-tokio-rustls` + `sqlx-postgres`）
- [x] `users` 表迁移（id / create_time / update_time / username / password_hash / email / avatar / bio）
  - 时间列用 `timestamp_with_time_zone`（**不用 `timestamp`** —— 无时区语义，跨时区必错）
  - `username` / `email` 加 unique 约束
- [x] 第二条迁移：`avatar` / `bio` 改可空（`Table::alter().modify_column(...).null()`，用 sea-query DSL 不写原生 SQL）
- [x] `sea-orm-cli generate entity -o src/model` → `users.rs`（`avatar` / `bio` 确认为 `Option<String>`）
- [x] 配置层：`src/config/mod.rs`（`Config::new()` 内部调 `dotenvy::dotenv()`）
- [x] `.gitignore` 补 `*.env`（防密码入 git 历史）
- [x] **store 层**：`find_by_username` / `find_by_email` / `create_user` + `NewUser` 入参 struct
  - `create_user`：argon2 哈希（`hash_password` 自动带随机盐）+ ActiveModel + `Set`
  - `avatar` / `bio` 走 `Set(data.avatar)` 透传，`None` 写 NULL
  - `..Default::default()` 让 `id` / `create_time` / `update_time` 走数据库默认值
  - 文档注释写明「不查重（查重是 service 层的业务规则）」
- [x] **错误三层架构**（每层只管自己的语言，**展平转换不嵌套**）
  - `StoreError`（`src/store/errors.rs`）2 变体：`NotFound` / `Technical` —— 只管存储
  - `ServiceError`（`src/service/errors.rs`）5 变体 —— 业务错误在这里产生
  - `ApiError`（`src/handler/errors.rs`）`Service(ServiceError)` + `IntoResponse` —— 翻成 HTTP
  - 转换链：`DbErr → StoreError → ServiceError → ApiError → HTTP`，每层一个 `From` impl
  - **为什么不嵌套**（`ServiceError::Store(StoreError)`）：嵌套会让 handler 的 match 变成三层 `ApiError::Service(ServiceError::Store(StoreError::Technical(e)))`，展平后只有两层
- [x] **service 层**：`register`（username/email 查重 → 409；再编排 store 插库）
- [x] **handler 层**：`src/handler/register/mod.rs`（`RegisterRequest` / `UserResponse` + handler）
  - **脱敏**：`impl From<Model> for UserResponse` 里**故意不搬 `password_hash`**
  - DTO 与 entity 分离：字段一改不会破 API 契约
- [x] **`main.rs` 完整化**：`Router` + `TraceLayer` + `.with_state(db)` + `axum::serve`
  - 顺序不能错：`route` → `layer` → `with_state`
  - `DatabaseConnection` 本身已 `Clone`，**不需要 `AppState` 结构体**（见踩坑 43）
  - `tracing_subscriber::fmt::init()` 必须在挂 `TraceLayer` 之前，否则 TraceLayer 是哑巴
- [x] **端到端实测通过**：`POST /users` → 201 Created
  - `password_hash = $argon2id$v=19$m=19456,...`（argon2id，非明文）
  - `avatar` / `bio` 返回 `null`（`Set(None)` → NULL 生效）
  - `create_time` 由数据库默认值填充（`..Default::default()` 生效）
- [x] **抽出 `router/` 层**：`src/router/mod.rs` 的 `app(db) -> Router`
  - 动机：`main.rs` 原本混着「进程生命周期」和「HTTP 装配」两件事
  - `TraceLayer` 放在 `app()` 里 → 返回的是**套好中间件的完整 Router**，`main.rs` 零 HTTP 知识
  - `main.rs` 从 37 行降到 29 行，净减 4 行 import（`axum::Router` / `routing::post` / `TraceLayer` / `register_handler` 全部搬走）
  - 剩下的全是进程生命周期：init 日志 → 读 Config → 连库 → 装配 → 绑端口 → serve
  - 生长路径：路由超 3 组实体时 `router/` 下按实体拆 `user.rs` / `auth.rs`，用 `.merge()` 拼，`main.rs` 零改动

### 🚧 进行中 / 下一步（线性竖切：一次一条链路）

- [ ] **`GET /users/{id}`** —— 用上零使用的 `NotFound` 变体，走通「`Option` → 404」路径，顺便验证 axum 路径参数（`Path<i32>`）
- [ ] **补输入校验**：密码长度 / 邮箱格式，让 `InvalidParams` 变体用起来（现在 `{"password":"1"}` 也能注册成功）
- [ ] **登录接口**：`verify_password`（与 `hash_password` 反向）+ 发 token（JWT 或 session + fred）
  - ⚠️ 登录的错误消息**必须模糊**（「用户名或密码错误」），防用户名枚举（见踩坑 51）
- [ ] cache 层（fred，cache-aside：读→miss→查库→回填；写→写库→失效缓存）
- [ ] `GET /users` 列表（会碰到分页这个大话题）
- [ ] `GET /health` 接口（调接口时先确认服务活着）

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
8. **`cargo add` 代替手写依赖**：手写 `version = "latest"` / 乱用 `full` feature 必翻车；版本号是语义化版本，features 每个包各不相同
9. **Rust 模块两种写法等价**：`src/config.rs` 与 `src/config/mod.rs`，`mod config;` 都能找到
10. **JSON 顶层结构**：不允许裸 `{...}` 对象块，设置必须是 `"键": 值`
11. **`expect` 是 panic 不是打印**：`Ok` 取值继续，`Err` 打印消息后**进程退出**（exit 101）。只用在**启动阶段**；请求处理函数里绝不能用（一个 404 不能让整个服务器死掉）
12. **配置层禁用 `unwrap_or_default()`**：读不到会给空字符串，错误信息跟根因完全无关，排查成本爆炸。配置缺失就该 `expect` 快速失败
13. **`Database::connect` 是 async fn**：忘 `.await` 拿到的是 Future 不是连接，编译报 `unused_must_use`

### Git

14. **没有 commit 就没有分支**：分支是"指向提交的指针"，`git push` 报 `src refspec master does not match any` 说明还没 commit
15. **GitHub 认证用 token 不用密码**：密码认证已禁用，需 Personal Access Token（勾 `repo`）；Linux 终端粘贴用 `Ctrl+Shift+V`
16. **sea-orm-cli 的工作目录**：默认找 `./migration`，必须在项目根目录运行；`migrate up` 要 `-u` 传 URL（**CLI 不读 .env**）
17. **Cargo workspace**：`migration/` 是独立 crate，根 Cargo.toml 需 `[workspace] members = ["migration"]`，否则 rust-analyzer 提示失灵
18. **psql 分页器**：长输出显示 `--More--`，`q` 退出、空格翻页；可用 `-P pager=off` 关闭
19. **连接超时排查顺序**：容器在跑吗（`docker compose ps`）→ 端口通吗 → URL 对吗
20. **GitHub SSH 22 端口被墙**：改用 443 端口（`ssh.github.com:443`），写入 `~/.ssh/config`
21. **sea-orm-cli generate 只做一半**：
    - entity 生成：自动加 `mod`，但**不会**帮你在 `src/main.rs` 加 `mod model;`，不补则不参与编译
    - migrate generate：自动建文件，但**不会**往 `migration/src/lib.rs` 的 `migrations()` 列表注册，不注册则 `migrate up` 静默跳过
    - 兜底验证：`\dt` 查 PG + `select * from seaql_migrations`

### 架构 / 选型

22. **`valkey` crate 是陷阱**：官方库但停在 `0.0.0-alpha5`（2024-12）后再无更新，且是**同步驱动**（要配 `spawn_blocking`）。改用 **`fred` 10.1**（异步、原生支持 Valkey 8、920 万下载）
23. **`sea-orm` 别用 default features**：默认拖进 sqlite + async-std + stream + time。要 `--no-default-features` 显式挑：`runtime-tokio-rustls` / `sqlx-postgres` / `macros` / `with-chrono` / `with-rust_decimal` / `with-uuid`
24. **sea-orm-cli 编译极慢**：`cargo install` 必须带 `--no-default-features --features sqlx-postgres,runtime-tokio-rustls`，否则编全数据库驱动
25. **CLI 与库版本要对齐**：sea-orm 2.0.3 + sea-orm-cli 2.0.2 能跑，但报怪错时先 `cargo install sea-orm-cli --version <对齐> --force` 重装

### sea-orm 实战

26. **sea-query 的 `ColumnDef` 默认是 NOT NULL**：反直觉。`string("x")` 生成 `varchar NOT NULL`，**要可空必须显式 `.null()`**。踩坑路径：建表时忘写 `.null()` → PG 里成了 not null → 已有数据只能追加迁移
27. **时间列用 `timestamp_with_time_zone`**：PG 的 `timestamp` 不带时区，同一串字面值从北京写入和伦敦写入含义差 8 小时，数据本身无法分辨。`timestamptz` 内部统一按 UTC 存
28. **DEFAULT 只在 INSERT 生效**：`update_time default current_timestamp()` 不会在 UPDATE 时刷新。要么应用层手动 `Set(now)`，要么建 PG 触发器
29. **函数参数不能用裸 `str`**：str 是不定长类型（DST），函数参数编译期必须确定大小；`&str`（胖指针）才是正确写法
30. **枚举变体用双冒号路径**：`Column::Username` —— `Column.eq(...)` 报 `expected value, found enum`；同套路还有 `Option::Some`、`Order::Desc`
31. **ActiveModel 三态 + Set 不在 prelude**：`Set(Some(v))` / `Set(None)` / 不碰（into() 自带的 Unchanged，UPDATE 不含该列）；`Set` 要手动 `use sea_orm::Set;`
32. **Option 字段写库要包一层**：可空列在 Model 里是 `Option<T>`，设值必须 `Set(Some(...))`；想写 NULL 用 `Set(None)` —— 不设字段 ≠ 写 NULL
33. **复制粘贴是字段名 bug 的温床**：六个 update 方法复制后忘改字段名，全写成 password_hash，且 `cargo check` 照样绿 —— 模板代码写完逐字段自查
34. **ActiveValue 不能直接做算术**：`wallet.balance + amount` 报 `cannot add ActiveValue<Decimal>` —— 裸值先算好再 `into()` 再 `Set`（"先算后装"）
35. **glob import 撞名要靠别名**：两个模块的 `use xxx::*` 同时带进 Model/Entity/Column，报 `Model is ambiguous` —— 主实体 glob，次要实体用 `{Entity as Transactions, Model as TransactionModel, ...}` 点名加别名
36. **类型名必须大写开头**：`vec` 写进类型位置直接报 `cannot find type`
37. **金额一律 Decimal，没有字面量**：`Decimal::ZERO` / `Decimal::ONE`，从字符串 parse（`"19.99".parse::<Decimal>()`）；绝不用 f64 存金额
38. **`Users::find_by_xxx` 便捷方法需要 `mock` feature**：没开就得用原始写法 `Users::find().filter(Column::Username.eq(v)).one(db)`。学习阶段不装 mock，用原始写法
39. **migrate 模板的 `todo!()` 要删**：`migrate init` / `migrate generate` 生成的模板里带 `todo!()`，一执行就 panic
40. **回滚 `down()` 有数据时会失败**：`SET NOT NULL` 遇到 NULL 值直接报错 —— 这是**数据不允许回退**，不是迁移写错。表空时随便回退

### 事务与连接

41. **⚠️ 泛型挡不住"误传池"**：`find_wallet<C: ConnectionTrait>(db: &C, ...)` 在事务里传 `&db`（池）**照样编译通过**。真正的 bug 是"本该走事务的查询走了池"，会读到事务外的旧数据 → 检查失效 → 超卖
    - **防线 1（结构）**：事务体藏在方法内部，外部只传 `&db`（用来 begin），内部全部用 `&txn` —— 调用方没机会写错
    - **防线 2（命名）**：事务块内不出现 `db` 这个词，变量命名 `txn`，看到 `&db` 就是 smell
42. **`DatabaseConnection` 本身就是连接池**：`Pool<DB>(Arc<PoolInner<DB>>)`，`#[derive(Clone)]`。`db.clone()` 只是引用计数 +1，不新建连接、不复制数据。所以 AppState 到处传 db 零成本
43. **AppState 只有 `db` 一个字段时是空壳**：`DatabaseConnection` 本身已 `Clone`，axum 直接 `.with_state(db)` 就够。等 `cache`（fred client）/ `config` 加进来再抽 struct 才回本
44. **层依赖方向**：`handler`（外层）→ `service` → `store`（内层）→ `model`。反过来的信号是**某个内层模块 import 了外层的东西** —— 立刻警惕

### 错误处理

45. **错误类型按层归属（各层只说自己的语言）**

    | 错误类型 | 文件 | 变体 | 只管什么 |
    |---|---|---|---|
    | `StoreError` | `store/errors.rs` | `NotFound` / `Technical` | 存储（2 个够了，其余是业务概念） |
    | `ServiceError` | `service/errors.rs` | + `Business` / `InvalidParams` / `Conflict` | 业务规则 |
    | `ApiError` | `handler/errors.rs` | `Service(ServiceError)` | HTTP 状态码 |

    **为什么 store 只有 2 个变体**：`Conflict`（用户名重复）是 service 判定的 —— store 只负责「这行插进去没有」，它不知道什么叫「用户名」。实际写代码时发现 `Conflict` 全在 service 层构造，store 一次没用过，于是把业务变体上移。
46. **展平转换，不嵌套**：`ServiceError::Technical(DbErr)` 直接持有 sea-orm 的错误，**不**写 `ServiceError::Store(StoreError)`。嵌套会让 handler 的 match 变成三层 `ApiError::Service(ServiceError::Store(StoreError::Technical(e)))` —— 展平后只有两层。
47. **错误变体要带 payload**：`NotFound(String)` 而不是光秃秃的 `NotFound` —— 否则只能说「目标资源未找到」，说不出缺哪个。除 `Technical` 包 `DbErr` 外统一带 `String`
48. **`Display` 用 `[变体名]` 前缀 + payload 写完整句子**：`write!(f, "[Conflict] {msg}")`。前缀负责分类（日志可 grep），payload 负责细节，**不要重复分类词**（`[Conflict] 冲突：xxx` 是废话）。三个错误类型都遵守这条
49. **`From` 只包装、不拆解；`IntoResponse` 才 match**：`From<StoreError> for ServiceError` 一行 `Self::Store(value)` 即可，**不要在这里 match 每个变体** —— 变体→状态的映射写在 `IntoResponse` 里。两件事分两个地方做
50. **⚠️ 拼写错误只要前后一致就能编译**：变体名 `Bussiness`（多了个 s）+ Display 字符串 `"[Bussiness]"` 三处一致，`cargo check` 全绿、运行时也正常。编译器只管「这个名字存不存在」，不管拼得对不对 —— 只能靠 `grep '\[Business\]'` 搜不到日志时发现。**这是踩坑 23（复制粘贴字段名 bug）的姊妹篇**
51. **⚠️ 登录的错误消息必须模糊**：注册接口返「用户名 x 已被占用」「邮箱 y 已被注册」可以接受（用户自己填的，反馈清楚有用）；但**登录绝不能这样** —— 那等于告诉攻击者「这个用户名存在 / 这个邮箱注册过」，拿邮箱列表就能撞库。统一回「用户名或密码错误」
52. **⚠️ 先查后插有 TOCTOU 竞态**：两个并发请求用同一个 username，两边 `find_by_username` 都返 `None` 都通过查重，然后一个 INSERT 成功、另一个撞 unique 约束 → `DbErr::Exec` → 被 `From` 包成 `Technical` → **返 500 而不是 409**。低并发下够用，但这是**已知缺陷**不是 bug-free。两条修法：① 捕获 `DbErr::Exec` 判断 unique violation（要解析错误信息）② `INSERT ... ON CONFLICT DO NOTHING` + 检查影响行数（不解析错误文本，更干净）
53. **`Conflict`(409) vs `InvalidParams`(422) 的分界**：**422 = 「你发的东西本身有问题」**（格式、长度、类型）；**409 = 「你发的东西没问题，但当前状态不接受它」**（重复、撞约束、状态机不对）。将来「订单已支付不能取消」也是 `Conflict`

### handler / axum

54. **`?` 在尾部表达式位置要配 `Ok(...)`**：返回 `Result<T, E>` 的函数，尾部只有两种写法 —— ① 尾部表达式本身类型就是 `Result<T, E>`（错误类型**正好**是 E）② `Ok(expr?)`（靠 `?` 转换 + `Ok` 补壳）。`create_user(db, data).await` 类型是 `Result<Model, StoreError>` 而签名要 `Result<Model, ServiceError>` → 不匹配；`create_user(db, data).await?` 尾部类型是 `Model` → 也缺壳。**两个缺一不可**
55. **`match` 的左边会「顺手起变量」**：`Self::Store(StoreError::Conflict(msg))` 里的 `msg` 是模式的一部分 —— 它既在**测试形状**（我要 self 长这样）又在**绑定值**（把里面的 String 叫这个名字）。所以右边才能用 `msg`。加上 `let (status, message) = match ... { ... };` 是**两件独立的事**：match 选一个元组出来，let 再把元组拆成两个变量
56. **Rust 从外往里读方法链**：行尾的 `.into_response()` 作用在它**前面完整的表达式**上 —— `(status, Json(...)).into_response()` 是对**整个元组**调用，不是只对 `Json` 调用。看不懂时先往上找括号边界
57. **axum 的 `Json` 提取器自带错误码**：JSON 合法但字段不匹配 → **422**（`JsonDataError`）；JSON 语法错 → 400；缺 `Content-Type: application/json` → **415**。这三个不用自己写
58. **`Router` 方法顺序不能错**：`route()` → `layer()` → `with_state()`。`with_state` 必须最后（它终结 builder 链），`layer` 必须在 route 之后（否则拦不到路由）
59. **抽 router 层后 `main.rs` 应该零 HTTP 知识**：`TraceLayer` 这类中间件放在 `app()` 内部，`main.rs` 只剩「init 日志 → 读 Config → 连库 → 装配 → 绑端口 → serve」。判据：`main.rs` 里除 `router::app(db)` 那一行之外不该出现任何 axum / tower 类型
60. **空的占位层要标在文档里**：`cache/mod.rs` 目前 0 字节，`Config.valkey_url` 已读进来但无人使用，fred 依赖装了零引用。文档写明「空壳，待建」比假装它不存在好 —— 免得下次以为是已完成的活

## 约定

- 镜像名 / 服务名 / 容器名一律小写
- 目录 / 模块 / 文件 / 函数一律 `snake_case`；类型 `UpperCamelCase`
- **目录名单数**：`user/` 不是 `users/`（`register/` 不是 `registers/`）
- **依赖一律用 `cargo add`**，不手写 Cargo.toml 版本
- 能用 sea-query DSL 表达的，**不手写原生 SQL**（原生 SQL 换库即废、拼错编译期不报错）
- 历史迁移**不能改**，只能追加新迁移
- **API 出参一律用 DTO，绝不把 entity 直接当 JSON 契约**（`password_hash` 会泄露；字段一改就破接口）
- 本地开发凭据（`runeshop`/`runeshop`）仅限本地，上线前必须换成环境变量注入
- `.env` 不入 git（`.gitignore` 已加 `*.env`）
- 代码保持格式化后再保存（Zed 已开 `format_on_save`）

---

## 旧项目存档（2026-09-26 前，已废弃）

**废弃原因**：17 张表全建 + 3 个 store 铺开后，改一行要同时顾多个 store，"驾驭不了"。改为线性竖切重建。

**可复用的经验已全部并入上文**，以下仅作数据模型参考（未来若重回多表路线可查阅）：

- 表清单（17）：users / products / orders / order_items / spec_dims / spec_values / product_variants / variant_values / wallets / wallet_transactions / xp_records / member_levels / user_memberships / merchants / merchant_accounts / merchant_addresses / user_addresses
- 商品规格采用 SPU-SKU 模型（`product_variants` + `variant_values` 做 N:N 中间表）
- 命名规范已完整继承到新项目（见「store 方法命名」）
- `StoreError` 五变体设计已继承
- 已踩过的坑 1-29 中仍适用的部分已并入上文
