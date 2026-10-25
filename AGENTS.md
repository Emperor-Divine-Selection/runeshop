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
├── migration/              # 独立 crate：数据库迁移（sea-orm 2.0.0，与根 2.0.3 有版本差）
│   └── src/
│       ├── lib.rs          # Migrator + migrations() 列表（每个新迁移都要手动注册！）
│       ├── main.rs         # CLI 入口（migrate up/down 的执行体）
│       ├── m20220101_000001_create_table_users.rs            # 建 users（id 是 pk_auto integer）
│       ├── m20260926_055846_alter_users_avatar_bio_nullable.rs  # avatar/bio 改可空
│       ├── m20261015_120000_alter_users_id_to_uuid.rs        # users.id integer → uuid
│       └── m20261015_160417_create_table_merchants.rs       # 建 merchants + merchant_status 枚举
└── src/
    ├── main.rs             # 入口：读 Config → 连库 → 连 Valkey → 起 Axum
    ├── config/
    │   └── mod.rs          # Config 结构体 + new()（内部调 dotenvy::dotenv()）
    ├── model/              # entity 层（sea-orm-cli 自动生成，不要手改）
    │   ├── mod.rs          # ⚠️ 手写模块里唯一用 mod.rs 的例外见命名章节
    │   ├── prelude.rs      # 别名（Entity as Users / Merchants）
    │   ├── sea_orm_active_enums.rs  # MerchantStatus（PG enum，自动生成）
    │   ├── users.rs        # users 实体（id: Uuid）
    │   └── merchants.rs    # merchants 实体（user_id 唯一 + FK cascade）
    ├── store/              # 存库层：只管「数据怎么存」——按表平铺，一个表一个文件
    │   ├── mod.rs          # 只放 pub mod 声明
    │   ├── error.rs        # StoreError（2 变体：NotFound / Technical）
    │   ├── user.rs         # users 表的方法（create_user / find_by_* / verify_password）
    │   └── merchant.rs     # merchants 表的方法（create_merchant / find_merchant_by_user_id）
    ├── service/            # 业务层：按「端」分组，common/ 放三端共用
    │   ├── mod.rs          # 只放 pub mod 声明
    │   ├── error.rs        # ServiceError（7 变体，业务错误在这里产生）
    │   ├── common/mod.rs   # pub mod auth / logout
    │   ├── common/auth.rs  # authenticate：查 token → user_id，无效返 Unauthorized
    │   ├── common/logout.rs # 删 token，三端共用
    │   ├── customer/mod.rs # pub mod user
    │   ├── customer/user/mod.rs   # register + login + profile + validate + is_valid_email
    │   ├── merchant/mod.rs # pub mod user
    │   ├── merchant/user/mod.rs   # merchant_login（查 merchants 判状态）
    │   ├── backoffice/mod.rs      # ⚠️ 空文件
    │   └── backoffice/user/mod.rs # ⚠️ WIP：没被 pub 声明 → 不参与编译（97 行拷贝）
    ├── handler/            # HTTP 层：与 service/ 同构，按「端」分组
    │   ├── mod.rs          # 只放 pub mod 声明
    │   ├── error.rs        # ApiError（把 ServiceError 翻成 HTTP 状态码）
    │   ├── response.rs     # ApiResponse<T> 统一响应信封
    │   ├── extractor.rs    # AuthUser 提取器 + bearer_token()（解析 Authorization 头）
    │   ├── common/mod.rs   # pub mod logout
    │   ├── common/logout.rs       # 三端共用登出
    │   ├── customer/mod.rs # pub mod user
    │   ├── customer/user/mod.rs   # pub mod login / me / register + 公共 DTO UserResponse
    │   ├── customer/user/{login,me,register}.rs
    │   ├── merchant/mod.rs # pub mod user
    │   ├── merchant/user/mod.rs   # pub mod login
    │   ├── merchant/user/login.rs # 商户登录（已挂 /merchant/auth/login）
    │   └── backoffice/mod.rs      # ⚠️ 空文件
    ├── router/             # 路由层：只管「URL → handler」
    │   ├── mod.rs          # app(state)：merge 各端 + TraceLayer + CorsLayer + with_state
    │   ├── common.rs       # /auth/logout
    │   ├── customer.rs     # /customer/register、/customer/auth/login、/customer/me
    │   ├── merchant.rs     # /merchant/auth/login
    │   └── backoffice.rs   # ⚠️ 空文件（尚未接入）
    ├── state/
    │   └── mod.rs          # AppState { db, cache }
    └── cache/              # 缓存层：fred 封装
        ├── mod.rs          # 只放 pub mod 声明
        ├── error.rs        # CacheError（单变体 Technical）
        ├── kv.rs           # 通用 KV：set_ex / get / del（不认识 token）
        └── token.rs        # token 业务：generate / save / get / delete
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
| **层的直属文件** | 扁平放，与 `mod.rs` 同级 | `service/error.rs`、`store/error.rs`、`handler/response.rs` |
| **按实体/操作分组的子集合** | 建子目录 | `store/user/`、`handler/user/`、`service/user/` |
| **层本身是容器** | 直接建目录 | `router/`、`cache/`、`state/` |

- **模块布局用 `foo/mod.rs`（2026-10-19 试过全换 `foo.rs` + `foo/`，已回退）**
  - `src/router.rs` 与 `src/router/mod.rs` 对 `mod router;` 完全等价（踩坑 9）—— 两种形态随时可互换，调用方零改动
  - 2018 风格（`foo.rs` + `foo/`）确实是官方推荐，也能消灭编辑器里一排同名 `mod.rs` 标签；但代价是目录树里 `handler.rs` 和 `handler/`、`user.rs` 和 `user/` **同名成对**，肉眼看起来更乱。本项目选择留在 `mod.rs` 风格
  - **唯一保留 `src/model/mod.rs` 的理由不同**：它是 sea-orm-codegen 生成的，重新跑 `sea-orm-cli generate entity` 会写回 `mod.rs`，手改会被覆盖
  - **单文件模块也可以用 `foo.rs`**（与 `foo/mod.rs` 等价，见踩坑 9），但本项目为统一保留 `foo/mod.rs`
- **错误文件名单数**：`error.rs` 不是 `errors.rs`（对照 hyper / reqwest / axum-core 的惯例）；模块里放的是「这一层的错误类型」，概念是单数
- 目录名单数：`user/` 不是 `users/`，`register/` 不是 `registers/`
- **按实体分的层**（store / service / handler）用 `user/` 承载一个实体的所有操作；**handler 层**曾经试过按操作分（`handler/register/` + 散放 `login.rs`），最后改回按实体分 —— 一个实体的所有接口放一起更好找
- **⚠️ 别让目录名 stutter**（见踩坑 76）：`service/user_service/` 里 `service` 说两遍，父目录已经说明层次
- **单复数不一致是「生成物 vs 手写」的边界**：`model/users.rs` / `model/merchants.rs` 是 sea-orm 按表名生成的**复数**；`store/user.rs` / `service/.../user/` 是手写模块，用**单数**。这不是漂移，是生成工具的约束 —— 别为了「统一」去改 `model/` 的文件名

### ⚠️ `store/` 是唯一的例外：按表平铺，不建子目录

store 层目前是 `store/user.rs` + `store/merchant.rs` 平铺，**不是** `store/user/mod.rs`。

**为什么和上面「按实体建子目录」不一致**（这是有意的例外，不是漂移）：

- store 层的单位是**表**，不是「实体」——`users` 和 `merchants` 就是两张表，将来 `products` / `orders` 是更多平铺文件
- 一个 `user/` 目录里塞 `create_user` / `find_by_username` / `verify_password`，`verify_password` 严格说**不属于 users 表**（它是纯 argon2 计算，不碰 DB），放进去反而别扭
- ⚠️ **但这带来一个真实的痛点**：`verify_password` 这个三端共用的逻辑现在藏在 `store/user.rs` 里，而它既是 store 层的、又和 store 的语义（只管数据怎么存）不搭。**将来大概率要上提到 `service/common/`**（见踩坑 78）

### ★ service/ 与 handler/ 现在有两个分组维度：端 + 实体

```
service/customer/user/mod.rs     ← 端（customer/merchant/backoffice）+ 实体（user）
```

- **端这一维**：三端（顾客 / 商户 / 后台）业务规则不同，同一个操作要做三份
- **实体这一维**：端内还有多个实体，将来会有 `service/customer/order/mod.rs`、`service/merchant/product/mod.rs`
- `common/` 是不属于任何一端的共用逻辑（当前只有 `logout`）

**⚠️ 这里有个尚未想清楚的问题**（**待决**，别急着照着写）：

`service/backoffice/user/mod.rs` 目前是 `service/customer/user/mod.rs` 的**逐字节拷贝**（97 行，`register` + `login` + `logout` + `validate` + `is_valid_email` 全部重复），而且**没被 `service/backoffice/mod.rs` 声明，根本不参与编译**。

问题是：**三端登录真的需要三份代码吗？** 三个选项，都还没定：

| 方案 | 说明 | 代价 |
|---|---|---|
| A. 每端一份 | 现状（拷贝三遍） | 改密码规则要改三处，忘一处就是 bug（踩坑 33 的跨端版本） |
| B. 端只做命名空间，逻辑下沉 common | `common/user/` 放归一化+validate+验密码，各端只留自己特有的（如商户多判 `merchants.status`） | 端变得很薄，「一眼看完这个端干什么」要跨文件跳 |
| C. 登录统一成一个入口 | `/auth/login` 内部按账户类型分派 | URL 语义变复杂；且后台账号是否和顾客共用 `users` 表还没定 |

**在这个问题定下来之前，`backoffice/` 那份拷贝是「脚手架」还是「要删的债」，不要默认任何一个。**

### 各层的划分依据

| 层 | 分法 | 理由 |
|---|---|---|
| `store/` | **按表平铺**（`user.rs` = users 表所有操作） | 唯一按表分的层，见上面「例外」说明 |
| `service/` `handler/` | **按端 → 按实体**（`customer/user/` = 顾客端的 user 业务） | 端隔离 + 端内实体隔离；待决问题见上 |
| `handler/` | 同上，按端分组 | 一个端的所有接口放一起更好找；按操作分反而散（试过 `handler/register/` + 散放 `login.rs`，改回了） |
| `router/` | **按端拆**（`mod.rs` 只做 merge + layer + with_state） | 路由是「URL → handler」的映射，三端已经够拆了（`.merge()` 拼） |

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
find_by_username(&state.db, "x").await   // 走池
find_by_username(&txn, "x").await        // 走事务，同一份代码
```

**当前形态是自由函数（`store::user::find_by_username`），不是 `UserStore` 关联类型** —— 因为 store 不持有任何状态（没有 `&self`），套一层 struct 只是多打几个字。文件路径本身承担了「命名空间」的作用。

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
| 目录、模块、文件 | `snake_case` | `store/user.rs` | ~~`store/UserStore/`~~ |
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
  - `StoreError`（`src/store/error.rs`）2 变体：`NotFound` / `Technical` —— 只管存储
  - `ServiceError`（`src/service/error.rs`）7 变体 —— 业务错误在这里产生
  - `ApiError`（`src/handler/error.rs`）`Service(ServiceError)` + `IntoResponse` —— 翻成 HTTP
  - 转换链：`DbErr → StoreError → ServiceError → ApiError → HTTP`，每层一个 `From` impl
  - **为什么不嵌套**（`ServiceError::Store(StoreError)`）：嵌套会让 handler 的 match 变成三层 `ApiError::Service(ServiceError::Store(StoreError::Technical(e)))`，展平后只有两层
- [x] **service 层**：`register`（username/email 查重 → 409；再编排 store 插库）
- [x] **handler 层**：注册接口的 DTO + handler（初版在 `src/handler/register/mod.rs`，三端重组后迁到 `src/handler/customer/user/register.rs`）
  - **脱敏**：`impl From<Model> for UserResponse` 里**故意不搬 `password_hash`**
  - DTO 与 entity 分离：字段一改不会破 API 契约
- [x] **`main.rs` 完整化**：`Router` + `TraceLayer` + `.with_state(db)` + `axum::serve`
  - 顺序不能错：`route` → `layer` → `with_state`
  - `DatabaseConnection` 本身已 `Clone`，**不需要 `AppState` 结构体**（见踩坑 43）
  - `tracing_subscriber::fmt::init()` 必须在挂 `TraceLayer` 之前，否则 TraceLayer 是哑巴
- [x] **端到端实测通过**：`POST /register` → 201 Created
  - `password_hash = $argon2id$v=19$m=19456,...`（argon2id，非明文）
  - `avatar` / `bio` 返回 `null`（`Set(None)` → NULL 生效）
  - `create_time` 由数据库默认值填充（`..Default::default()` 生效）
- [x] **抽出 `router/` 层**：`src/router/mod.rs` 的 `app(db) -> Router`
  - 动机：`main.rs` 原本混着「进程生命周期」和「HTTP 装配」两件事
  - `TraceLayer` 放在 `app()` 里 → 返回的是**套好中间件的完整 Router**，`main.rs` 零 HTTP 知识
  - `main.rs` 从 37 行降到 29 行，净减 4 行 import（`axum::Router` / `routing::post` / `TraceLayer` / `register_handler` 全部搬走）
  - 剩下的全是进程生命周期：init 日志 → 读 Config → 连库 → 装配 → 绑端口 → serve
  - 生长路径：三端出现后 `router/` 按端拆（`customer.rs` / `merchant.rs` / `common.rs`），用 `.merge()` 拼，`main.rs` 零改动
- [x] **补全注册规则**（`InvalidParams` 终于用上了）
  - 抽出 `validate()` + `MIN_PASSWORD_LEN` 常量；顺序：**归一化 → 校验（不碰 DB）→ 查重（查两次库）→ 入库**
  - 规则：用户名非空 / 邮箱正则（顶级域名 ≥2 字母）/ 密码 `chars().count() >= 8`
  - **归一化**：`trim()` + `to_lowercase()` 作用于 `username` 和 `email`，**检查/查重/入库三处都用归一化后的值**
  - 邮箱正则用 `static RE: LazyLock<Regex>` 包一层，全局只编译一次
  - 实测：422（空用户名/邮箱格式错/`a@b.c`/密码 7 位/4 个汉字）、201（带首尾空格+大小写混写）、409（用户名或邮箱重复）全部通过
  - ⚠️ 归一化只对新写入生效 —— 库里已有 `alice` + `ALICE` 脏数据，见踩坑 61
- [x] **cache 层**（fred，通用 KV 打底 + 业务封装）
  - `cache/kv.rs`：`set_ex` / `get` / `del` 三个通用函数，**不认识「token」** —— 收 `client: &Client` 参数（连接由 main 注入，和 store 层方案乙一致）
  - `cache/token.rs`：`generate()` / `save()` / `get()` / `delete()`，拼 `token:` 前缀 + 默认 24h TTL
  - `cache/error.rs`：`CacheError::Technical(fred Error)` —— 单变体（valkey 只可能成功/技术失败，「查不到」走 `Option`）
  - 实测：save→TTL 86367（默认24h）/ 指定 60s 生效 / get 不存在返 `None` / generate 64 字符
- [x] **登录接口** `POST /customer/auth/login`
  - `store::verify_password(stored_hash, plain)`：argon2 **反向验证**，不是重新哈希再比对（盐随机，字符串永不相等）
  - ⚠️ 参数顺序是 **`(哈希, 明文)`** —— 和 `create_user` 的 `NewUser.password`（明文）正好相反，记混了编译能过、运行必错（见踩坑 83）
  - `service::login`：归一化 → 查用户 → 验密码 → `generate()` → `save()` → 返 token
  - ⚠️ **用户不存在和密码错返同一句话**「用户名或密码错误」，防用户名枚举（见踩坑 51）
  - `service::login` 只返 `String`（token）—— 用户名让前端自己查，登录不是查用户
  - `LoginResponse` 只含 `token`，不从 `Model` 搬字段
- [x] **统一响应信封** `ApiResponse<T>`
  - 三种场景（成功/登录/失败）统一 `{code, msg, data}`，前端一套解析逻辑
  - `ok(code, data)` / `err(code, msg)` —— `ApiResponse<()>` 单独 impl `err`，失败侧不需要泛型
  - `IntoResponse` 里 `StatusCode::from_u16(code)` 反推 HTTP 状态码 —— **body 的 code 和 HTTP 头保持一致**
  - ⚠️ 不能因为包了信封就把 HTTP 状态码一律返 200 —— 监控/网关/重试只看 HTTP 头
- [x] **`AppState` 诞生**（踩坑 43 说的「第二个字段」触发时刻）
  - 放在 `src/state/mod.rs` 而非 `main.rs` —— 否则 handler 要 `use crate::AppState` 反向依赖 main（见踩坑 44）
- [x] **handler 按实体重组**：`handler/register/` + 散放的 `login.rs` → `handler/user/{register,login}.rs`
  - 一个实体的所有接口放一起；`login` 的 URL 是 `/auth/login`（动作）但代码归 user
- [x] **登出接口** `POST /auth/logout`：读 `Authorization: Bearer <token>` 头 → `cache::token::delete`（头名后来从 `token` 改过来了，见上面 401 语义统一那节）
- [x] **主键 integer → uuid**（`m20261015_120000_alter_users_id_to_uuid`）
  - sea-query 没有「改列类型」的 DSL，只能**四步绕**：加新列（`uuid().default("gen_random_uuid()")`）→ drop 旧列 → rename → `modify_column().primary_key()`
  - `users.id` 和 `merchants.id` / `merchants.user_id` 全部 uuid
  - ⚠️ `down()` 是**破坏性**的（回滚会丢新生成的 uuid），和「有数据时回滚失败」（踩坑 40）是两类问题
  - **entity 要重新生成**：`#[sea_orm(primary_key, auto_increment = false)]` + 字段类型 `Uuid`
- [x] **`merchants` 表**（`m20261015_160417_create_table_merchants`）+ `merchant_status` PG 枚举
  - 枚举用 `create_type(...)` 建 PG 原生 enum，列用 `.enumeration("status", "merchant_status", [...]).default("pending")`
  - `user_id` 加 unique（一人一商户）+ FK 指向 `users(id)`，`on_delete/on_update = Cascade`
  - entity 自动生成 `sea_orm_active_enums.rs` → `MerchantStatus`（四个态：Pending/Active/Frozen/Rejected）
  - ⚠️ `down()` 顺序有讲究：**先 drop table 再 drop enum type**，反了会因依赖关系失败
- [x] **`merchant_login`（商户登录骨架）**：查 users → 验密码 → 查 merchants → 判 `status`
  - 状态判定用 `match merchant.status` 穷举四个变体，`Active` 才发 token
  - ✅ 已修：`generate()` + `save()` 原来写在 `match status` **之前**（非 Active 商户会留下孤儿 token），现在只在 `Active` 分支里发 token（见踩坑 78）
  - ✅ 已挂路由：`POST /merchant/auth/login`
- [x] **router 按端拆分（2026-10-20）**
  - `router/mod.rs` 只做 `merge` + `layer` + `with_state`；各端 `routes() -> Router<AppState>`
  - **URL 约定**：端特有接口带端前缀（`/customer/auth/login`、`/merchant/auth/login`），三端共享的不带（`/auth/logout`）
  - `Router<S>` 的 `S` 是「还缺哪种 state」的编译期标记：端文件返回 `Router<AppState>`，`with_state` 一次性补成 `Router<()>`
  - ⚠️ 尾斜杠是另一个路径：`/customer/register/` ≠ `/customer/register`
- [x] **错误文件改名（2026-10-19）**：`errors.rs` → `error.rs`（4 处）
  - 主流 crate（hyper / reqwest / axum-core）都是单数；模块里放的是「这一层的错误类型」
- [x] **模块布局试过 `foo.rs` + `foo/`，已回退到 `mod.rs`**
  - 19 个文件纯 rename、0 行内容变更，编译完全通过
  - 回退原因：目录树里 `handler.rs` 和 `handler/`、`user.rs` 和 `user/` 同名成对，观感比一排 `mod.rs` 更乱
  - 结论：这是**审美/习惯选择**，两种都合法；本项目选 `mod.rs`。`src/model/mod.rs` 无论如何都保留（生成物）

- [x] **登录态校验**：`AuthUser` 提取器（`FromRequestParts`：读 `Authorization` 头 → `cache::token::get` → 拿 `user_id`）
  - 头名从 `token` 改成 **`Authorization: Bearer <token>`**（趁没有客户端，改 1 处 vs 以后改 N 处）
  - `bearer_token()` 只做「解析头」；`logout` 复用它但**不校验**（过期 token 也要能登出 → 幂等）
  - **token 语义 = 身份，不带端类型**：商户状态可变，token 里存角色 = 缓存了会失真的权限。角色由端点查库判定（将来的 `MerchantUser` 提取器 = `AuthUser` + 查 `merchants.status`）
  - ✅ `GET /customer/me` 已挂上 —— 提取器的第一个使用场景，也是前端「刷新后保持登录」的唯一依据
- [x] **401 语义统一**：`ServiceError::Unauthorized` 新增，映射 401
  - 没带 Authorization 头 / 头格式不对 / token 过期 → 401
  - 登录失败（密码错、用户不存在）→ **从 400 改成 401**（文案仍模糊「用户名或密码错误」，防枚举）
  - ⚠️ 前端要靠「401 → 清 token + 跳登录页」兜底会话失效，状态码混用前端只能比对文案
- [x] **CORS**：`CorsLayer` 装在 `app()` 里（`allow_origin/allow_methods/allow_headers` 全 `Any`）—— 前端跨域必装，否则浏览器直接拦
  - ⚠️ 开发期用 `Any`，上线要收紧成具体域名
- [x] **`UserResponse` DTO 搬家**：`handler/customer/user/register.rs` → `handler/customer/user/mod.rs`（注册和 /me 共用）
  - ⚠️ 搬之前两份同名 DTO 并存 → **是两个不同类型**，改字段要改两处

### 🚧 进行中 / 下一步（线性竖切：一次一条链路）

**⚠️ 先收尾 WIP（当前 3 个 dead_code warning 都来自「功能未接线」）**

- [x] **删 `handler/customer/user/logout.rs` 和 `service/customer/user::logout`** —— logout 已经提到 `common/`，这两份是死代码，router 用的是 `handler::common::logout`
- [ ] **删 `service/backoffice/user/mod.rs`（97 行逐字节拷贝）** 或补上 `pub mod user;` 声明让它参与编译 —— 先定「三端是否共用 login/register 逻辑」（见上面 A/B/C 三个方案）
- [x] **修 `merchant_login` 的孤儿 token bug**：把 `generate()` + `save()` 挪到 `match status` 的 `Active` 分支里
- [x] **修 `ceate_merchant` 拼写** → `create_merchant`
- [x] **清 `service/merchant/user/mod.rs` 的未用 import**

**剩余 warning 对应的功能（不是死代码，别用 `#[allow]` 掩盖）**

| warning | 等哪个功能 |
|---|---|
| `create_merchant` / `NewMerchant` never used | 商户注册链路 |
| `store::error::StoreError::NotFound` never constructed | ⚠️ 见踩坑 90 —— 按 `find_by_xxx` 返 `Option` 的约定，store 层**根本不会**返回 NotFound，这一条大概会一直死着 |

**然后往下走**

- [ ] **商户注册链路** `POST /merchant/register`：用上 `create_merchant` / `NewMerchant`（`status` 默认 pending），让「注册 → 申请商户 → 商户登录」闭环
- [ ] **`GET /users/{id}`** —— 走通「`Option` → 404」路径，顺便验证 axum 路径参数
  - ⚠️ **是 `Path<Uuid>` 不是 `Path<i32>`**（主键已迁 uuid，见上面）
- [ ] **角色判定提取器** `MerchantUser`：`AuthUser` + 查 `merchants` 表判 `status == Active`（token 不带端类型的代价就在这补回来）
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
38. **`Users::find_by_xxx` 便捷方法需要 `mock` feature**：没开就得用原始写法 `Users::find().filter(Column::Username.eq(v)).one(db)`。学习阶段不装 mock，用原始写法 —— ⚠️ **这条在 sea-orm 2.0.3 已不成立**（`find_by_id` 没有 mock gate），见踩坑 89
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
    | `StoreError` | `store/error.rs` | `NotFound` / `Technical` | 存储（2 个够了，其余是业务概念） |
    | `ServiceError` | `service/error.rs` | + `Business` / `InvalidParams` / `Conflict` / `Unauthorized` | 业务规则 |
    | `ApiError` | `handler/error.rs` | `Service(ServiceError)` | HTTP 状态码 |

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
60. **空的占位层要标在文档里**：项目早期的 `cache/mod.rs` 曾是 0 字节，`Config.valkey_url` 已读进来但无人使用，fred 依赖装了零引用。文档写明「空壳，待建」比假装它不存在好 —— 免得下次以为是已完成的活

### 归一化与校验（register 的实战）

61. **⚠️ 加归一化规则不改历史数据 → 库里留下脏数据**：给 `username` / `email` 加 `to_lowercase()` 后，**规则只对新写入生效**。旧数据原样躺着 —— 实测库里同时存在 `alice` 和 `ALICE`，而查重只认小写，于是新注册 `ALICE` 会撞上已有的 `alice` 但库里的历史 `ALICE` 永远占着坑
    - 修法（本地/测试库）：`DELETE FROM users WHERE username != lower(username)`
    - 修法（真库）：追加迁移 `UPDATE users SET username = lower(username)` —— **但可能撞 unique 约束**，得先`SELECT` 探明哪些会撞、合并还是改名
    - 教训：归一化规则的变更和 schema 变更一样，**必须配一条迁移**，不能只改代码
62. **⚠️ 归一化必须在所有入口统一做**：注册时做了 `trim()` + `to_lowercase()`，**将来登录查重必须做一模一样的处理**，否则用户输 `Alice` 登不上（库里存的是 `alice`）。凡是「写的时候归一化」，所有读路径都得跟上
63. **`chars().count()` 数字符不是字节**：`"密码密码"` 是 4 个字符（8 字节），`len()` 会算成 8 从而误放行，`chars().count()` 算 4 正确拒绝。反过来汉字在「至少 8 位」的规则下天然占便宜（8 个汉字 ≈ 英文 16+ 字符强度）—— 这是取舍不是 bug
64. **⚠️ 布尔判断写反编译器抓不到**：`if is_valid_email(x) { return Err("格式不正确") }` —— 逻辑正好颠倒，合法邮箱被拒、垃圾邮箱放行，而 `bool` 类型合法所以编译全绿。**同类：踩坑 50（拼写一致就能编译）**。写完 `if` 条件后**逐条代入正常值和异常值各验一遍**
65. **正则永远不要在循环里编译**：`Regex::new` 要几微秒到几毫秒。官方推荐用 `static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"...").unwrap())` —— `static` 定义在函数内部也行，全局只初始化一次
66. **⚠️ 用户输入绝不能当正则**：`Regex::new(用户输入)` 会因非法语法 panic，那是 DoS 攻击面。`.unwrap()` 只在「模式是手写常量」时安全
67. **Rust 的 `regex` 不支持反向引用和环视**（`\1`、`(?=...)`）：这是**有意的取舍** —— 该库用有限自动机保证线性时间，不可能出现「灾难性回溯」，而 JS/Java 的正则能写出 `(a+)+$` 这种指数级 DoS。代价是某些模式要换个思路写
68. **邮箱正则只能验格式，验不了存在性**：`notreal@gmail.com` 格式合法但邮箱不存在。正则挡住的只是 `abc`、`a@b`、`@x.com` 这类明显笔误。**要真验证必须发验证码 + 加 `email_verified` 字段**（现在项目里没做，是有意为之）

### 缓存 / fred 实战

69. **⚠️ fred 的 `connect()` 绝对不能 `await`**：`connect()` 返回 `ConnectHandle = JoinHandle<Result<(), Error>>`，fred 源码注释明说它「resolve 于连接**关闭**时」—— await 它会永久挂起。症状是**程序静默卡死**：没有 panic、没有日志、退出码 124，极易误诊。正确用法是**两步**：
    ```rust
    client.connect();                     // 启动后台连接任务（同步调用）
    client.wait_for_connect().await?;      // 这个才是「等连上」
    ```
    初次尝试过 `connect().await.expect()` 和链式写法，全部踩坑。**看到 `JoinHandle` 就要警觉**：它包的是「任务的结果」，不是「任务做完了」。
70. **`ClientLike` trait 必须导入**：`connect()` / `wait_for_connect()` 是 trait 方法，不是 `Client` 的 inherent 方法。`use fred::interfaces::ClientLike;` —— 而且**每个用到命令的模块都要自己导一次**（trait 作用域不跨模块）。
71. **`get` 必须显式标注目标类型**：`let v: Option<i32> = client.get(key).await?;` —— fred 靠 `R: FromValue` 决定怎么把字节解析成你的类型，标注了才推得出来。少了标注报 `type annotations needed`。
72. **不需要的返回值用 `let _: () = ...` 消掉**：`set` / `del` 的返回类型 `R` 只出现在返回值里，不标注编译器推不出来。标 `()` 表示「我不关心回了什么，只关心有没有报错」。
73. **`format!` 里写错括号会静默出错**：`format!("[KEY_PREFIX] {token}")` —— 方括号是**字面量**，不会插值！结果是 `"[KEY_PREFIX] abc123"`，而常量 `KEY_PREFIX` 变成未使用（编译器会警告 `constant is never used`，这是发现它的信号）。**哨兵**：写完 `format!` 后，凡是不该出现在输出里的常量名都要检查。
74. **⚠️ 日志分类（Display 前缀）和变体名是两件事**：`ServiceError` 的 `Database` / `Cache` 两个变体在 `Display` 里**都打 `[Technical]` 前缀** —— 因为「这是技术故障」才是日志要回答的问题，来源靠**模式匹配**判断，不靠**字符串**。想统一日志，就统一 Display 前缀；**别为了统一字符串去搞 `Box<dyn Error>`**（那是过度设计，曾在这个问题上绕了弯路）。
75. **响应信封里 HTTP 状态码和 body 的 `code` 必须一致**：包了 `ApiResponse` 也**不能一律返 200**。HTTP 状态码给机器看（nginx / 监控 / HTTP 库的重试逻辑），body 的 code 给前端逻辑看。实现方式：`IntoResponse` 里 `StatusCode::from_u16(self.code)` 反推，**同一个字段驱动两处**，不可能不一致。
76. **目录名别 stutter（service/user_service 实录）**：`src/service/user_service/{user,login}.rs` 里 `service` 说了两遍。父目录 `service/` 已经说明层次，子级只该说「哪个实体」。改名为 `service/user/mod.rs` 后调用路径从 `crate::service::user_store::` 变成 `crate::service::user::`，**少一层纯为命名的目录**。
77. **URL 不认识你的代码结构**：`.route("handler/register", ...)` 把目录名写进了 URL。URL 是 API 契约，文件放哪不该影响它 —— 函数在 `handler/user/register.rs`，URL 叫 `/register` 完全没问题。另外**路由必须以 `/` 开头**，少了就是静默 404。

### 三端重构期的坑（2026-10-15 起）

78. **⚠️ 有副作用的代码写在「判定」之前 = 孤儿数据**：`merchant_login` 里 `generate()` + `save(cache, &token, ...)` 写在 `match merchant.status` **之前**。于是 Pending/Frozen/Rejected 的商户**登录失败**（返 5xx/4xx），但 Valkey 里留了个 24h 后才过期的 token —— 谁也不知道它存在、谁也用不掉
    - **通用教训**：任何「写外部系统」的动作（DB insert / cache set / 发消息 / 发邮件）都必须**放在所有判定通过之后**。先算出 `token`，判完状态再 `save()`
    - **自查手法**：在函数里搜 `save(` / `insert(` / `send(`，逐个问「这一行的前面，所有失败分支都走完了吗？」
    - **姊妹篇**：踩坑 52 的 TOCTOU —— 同样是「判定」和「副作用」的先后顺序问题
79. **⚠️ 空 `mod.rs` 让整个子目录「静默蒸发」**：`service/backoffice/mod.rs` 是 **0 字节**，没有 `pub mod user;` —— 于是 `service/backoffice/user/mod.rs` 里那 97 行代码**根本不参与编译**。`cargo check` 全绿、`grep` 能搜到内容、`git` 里文件好好躺着，但**一行都没跑**
    - 根因和踩坑 21 同源：**「文件存在」不等于「代码在编译」**。Rust 的模块是**从 `mod.rs` 出发向下递归**的，父目录不声明 = 子文件不存在
    - **哨兵**：`git status` 显示某个文件「存在」，但 `cargo check` 从没报过它的任何 warning（死代码也会 warning）→ 它不在编译范围里
    - **自查手法**：`cargo check 2>&1 | grep <文件名关键词>` —— 零输出就说明它没进编译
    - 对照组：`service/backoffice/mod.rs` 0 字节 → 子目录蒸发；`handler/backoffice/mod.rs` 1 字节（一个换行）→ 同样蒸发。**父 `mod.rs` 空 = 子目录白建**
80. **⚠️ 三端拷贝是踩坑 33 的跨端版本**：`service/backoffice/user/mod.rs` 是 `service/customer/user/mod.rs` 的**逐字节拷贝**（`diff` 结果完全一致，97 行）。`register` / `login` / `validate` / `is_valid_email` / `MIN_PASSWORD_LEN` 全部重复
    - 三端查的是**同一张 `users` 表**，登录逻辑本该只有一份
    - 将来改密码规则要同时改三处，**忘一处就是 bug**，而且编译器完全不会提示（每个副本都自洽）
    - **判断标准**：拷贝之前先问「这两份的**差异**是什么？」。差异 = 零 → 不该拷贝；差异 = 一条额外规则 → 应该抽公共部分，端里只留那一条
81. **`store` 按表平铺 ≠ `service` 按实体分目录**：`store/user.rs` 是平铺文件，但 `service/customer/user/mod.rs` 是两层目录。两个层用了不同的分法 —— 这是**有意的**（store 的单位是表，service 的单位是「端 + 实体」），但会带来「两个层里 `user` 的位置不一样」的认知成本。见「层内组织的命名规律」章节
    - 附带代价：`verify_password` 是纯 argon2 计算、不碰 DB，严格说**不属于任何一张表**，却被塞在 `store/user.rs` 里。**将来该上提到 `service/common/`**
82. **⚠️ `cargo check` 不报错 ≠ 逻辑对**：本次 18 个 warning 里，**类型/借用层面的错误一个都没有**，但藏着 1 个真 bug（孤儿 token）、1 个拼写错误（`ceate_merchant`）、1 个没进编译的子目录（踩坑 79）。编译器只管**类型和借用**，语义错误、命名错误、装配错误一概不管
    - **对策**：每个 WIP 阶段收尾都跑一遍 `cargo check 2>&1 | grep warning`，把 warning 清单当 TODO 清单逐条消
83. **`store` 方法参数顺序要成对记**：`verify_password(stored_hash, plain)` —— **哈希在前**。而 `create_user(db, data)` 里 `data.password` 是**明文**（store 内部自己哈希）。两个函数的「密码参数」位置正好相反，记混了编译能过、运行必错
    - 记忆锚点：凡是「已经存好的值」当查询条件（`find_by_username`），第一个参数就是它；凡是「要写进去的值」（`NewUser.password`），就是明文
84. **⚠️ 空壳层比没有更危险**：`service/backoffice/`（97 行拷贝，不参与编译）、`handler/backoffice/mod.rs`（1 字节）、`store/merchant.rs`（曾经有 3 个未用 import + 1 个拼写错误的方法，且**从未被调用**）—— 这些「看起来已经做了」的代码，grep 得到、编辑器里能看到，但**不参与编译或从未执行**
    - 文档写明「⚠️ WIP：不参与编译」比假装它不存在好 —— 免得下次以为是已完成的活（对照踩坑 60）
85. **⚠️ 根 crate 的 sea-orm features 靠「workspace feature 统一」白拿**：根 `Cargo.toml` 只声明了 `["sqlx-postgres", "runtime-tokio-rustls"]`，而 `with-chrono` / `with-uuid` / `macros` 这些**只在 `migration/Cargo.toml` 里声明**。主 crate 能编译，是因为 cargo 把同一 workspace 内所有成员的 feature 求了并集
    - **危险之处**：哪天上游改了这个「求并集」的行为（或把 migration 拆出 workspace），主 crate 立刻一堆 `cannot find type Uuid` / `DateTimeWithTimeZone`
    - **正解**：根 crate 的 features 必须**自己写全**，不能依赖别的成员顺带打开
    - **姊妹篇**：踩坑 23 —— 那次是「别用 default features（会拖进 sqlite + async-std）」，这次是「别用别人的 features」
86. **⚠️ 归一化在 register 和 login 之间不对称**：`login` 做了 `trim().to_lowercase()`，`register` **只做了 `trim()`**（`service/customer/user/mod.rs:46-47`）。后果：注册 `ALICE` 存成 `ALICE`，登录 `Alice` 转成 `alice` 去查 —— **查不到**
    - 这正是踩坑 62 说的「归一化必须在所有入口统一做」，文档里写了规则，代码里没落实
    - **哨兵**：改任何一处归一化规则，`grep -rn "to_lowercase" src/` 看**每一处**的调用方是不是都跟着对齐了
    - 附带：库里已经有 `alice` / `ALICE` 混存的脏数据（踩坑 61），修代码前先想好要不要配迁移

### axum 提取器 / uuid 类型（2026-10-21 认证闭环）

87. **⚠️ `&Uuid` 不能 `into()` 成 `Uuid`**：`uuid` crate **没有** `impl From<&Uuid> for Uuid`（对比 std 给 `String`/`Vec<T>` 都提供了 `From<&T>`），所以 `user_id.into()`（`user_id: &Uuid`，目标 `Uuid`）**编译不过**。`Uuid` 是 `Copy` → 用 `*user_id`
    - **连带**：`Column::Id.eq(*user_id)` 里 `UuidColumn::eq` 的约束是 `V: Into<Uuid>`（sea-orm 2.x 的**类型化列**，`src/entity/column/types/with_uuid.rs`）
88. **`clone()` 在引用上的返回值是"推"出来的**：`user_id: &Uuid` 时 `user_id.clone()` 返回 **`Uuid` 而不是 `&Uuid`** —— 方法解析先试 `Uuid::clone`（接收者 `&Uuid` 刚好匹配），`<&Uuid>::clone` 需要接收者 `&&Uuid`（多一次自动引用，优先级更低）
    - 能编译，但类型不是写出来的 → 读代码时容易看错。**`Uuid` 是 `Copy`，直接 `*user_id`**（clippy 的 `clone_on_copy` 也会盯上 clone 写法）
89. **⚠️ 踩坑 38 在 sea-orm 2.0.3 已经不成立**：查 `sea-orm-2.0.3/src/entity/base_entity.rs`，`EntityTrait::find_by_id` **没有** `mock` gate（`mock` feature 只 gate `DatabaseConnection::from_mock` 那类），所以 `Users::find_by_id(uuid)` 在本项目**可用**。踩坑 38 是 1.x 时期的结论 —— **版本升级后要重新验证老结论**
    - 顺带：`find_by_id` 自带主键条件，再 `.filter(Column::Id.eq(...))` 是重复的
90. **⚠️ store 层的 `NotFound` 变体大概是死代码**：`find_by_xxx` 按约定返 `Option`（不是 `Result`），store 层**根本不会**返回 NotFound —— 「查不到」由 service 层翻成 `ServiceError::NotFound` → 404。所以 AGENTS.md 里「`GET /users/{id}` 会用上 `StoreError::NotFound`」的说法是错的（除非给 store 加一个 `get_by_id` 返 `Result`）
91. **提取器（extractor）是 axum 的核心抽象**：`State` / `Json` / `Path` / `Query` 全是提取器 = 「从请求里取一份数据交给 handler」
    - 自己写一个 = 实现 `FromRequestParts`（trait 在 **axum-core** 里，axum 只是再导出；trait 方法声明为 `-> impl Future + Send`，**实现时可以用 `async fn`**）
    - **为什么用 `FromRequestParts` 不用 `FromRequest`**：前者不消费 body，能排在 `Json` 前面；后者只能当最后一个参数
    - `type Rejection` = 「提取失败时返回什么」，本项目选 `ApiError`（实现了 `IntoResponse`），于是 401 走统一信封
    - `Router<S>` 的 `S` 是**「还缺哪种 state」的编译期标记**：端文件返 `Router<AppState>`，`with_state` 一次性补成 `Router<()>`；**只有 `Router<()>` 实现了 `Service`**，所以没装配完的路由传不进 `axum::serve`
92. **⚠️ 同一个 DTO 在两个模块各定义一份 = 两个不同的 Rust 类型**：`UserResponse` 曾在 `register.rs` 和 `user/mod.rs` 各有一份 —— 编译能过，但改字段要改两处，且 `register_handler` / `me_handler` 返回的类型不是同一个
    - **正解**：公共 DTO 放该实体的 `mod.rs`（`handler/customer/user/mod.rs`），子模块 `use super::UserResponse;`
    - **哨兵**：`grep -rn 'struct UserResponse' src/` 出现两次 = 重复定义

## 代码纪律（写代码前先看）

> 目标：半年后回来还能一眼看懂。**宁可多写一行直白代码，也不写一行需要猜的「聪明」代码。**

### 1. 命名说人话

- 名字回答「是什么 / 干什么」，不回答「怎么实现」
- 函数用动词开头：`create_user` / `find_by_id` / `validate`；布尔用 `is_` / `has_` / `can_` 开头
- **函数名要能脱离目录独立读懂**：`apply` / `login` / `update` 单独看不知道操作对象 → 带上实体：`apply_merchant` / `merchant_login` / `update_order_status`。目录是补充，不是替代
- 集合用复数：`users` / `merchants`；单个用单数：`user` / `merchant`
- 不用自造缩写。`db` / `cache` / `id` / `url` 可以，`usr` / `mgr` / `val` 不行
- 宁可名字长一点，也不要 `data` / `info` / `temp` 这种看不出内容的词

```rust
// 差：看不出 data 是什么
fn handle(data: Data) -> Result<()>

// 好：一眼知道是注册请求、返回用户
fn register_user(request: RegisterRequest) -> Result<User, ServiceError>
```

### 2. 卫兵写法：先挡异常，再写主干

- 参数不对、找不到、没权限 → **立刻 return / Err**，主干逻辑不要往 if 里面塞
- 主干代码保持在最外层缩进；函数里嵌套最多两层
- 用 `let Some(x) = ... else { return ... }`、`if !ok { return ... }`，不要写成「金字塔」

```rust
// 差：主干被三层 if 埋住
if let Some(user) = find_user(db, id).await? {
    if user.status == Status::Active {
        if verify_password(&user.password_hash, password) {
            return Ok(user);
        }
    }
}
Err(ServiceError::Unauthorized("用户名或密码错误".into()))

// 好：异常先挡掉，主干一眼看完
let Some(user) = find_user(db, id).await? else {
    return Err(ServiceError::Unauthorized("用户名或密码错误".into()));
};
if user.status != Status::Active {
    return Err(ServiceError::Unauthorized("用户名或密码错误".into()));
}
if !verify_password(&user.password_hash, password) {
    return Err(ServiceError::Unauthorized("用户名或密码错误".into()));
}
Ok(user)
```

### 3. 函数只做一件事，不写超大函数

- 判据：能用一句话说清「它做什么」。需要「并且 / 然后」才能说清 → 拆
- 经验阈值：**超过 40 行先想拆，超过 60 行基本必须拆**
- service 按步骤拆成小函数：`归一化 → 校验 → 查重 → 入库`，每一步都能单独看懂
- 一次只做一件事的副作用：写库 / 写缓存 / 发消息**不要**和参数校验混在一起
- 层与层不越界：HTTP 概念（状态码、Header）留在 handler；业务规则留在 service；SQL 留在 store

### 4. 错误处理：早失败、不吞、不裸 unwrap

- **请求路径禁止 `unwrap()` / `expect()` / `panic!()`**。一个 404 不能让整个服务死掉
- `expect()` 只允许出现在启动阶段（读配置、连数据库）
- 优先 `?`；需要补上下文时 `map_err(|e| ...)`，不要丢掉原始错误
- 错误变体带 payload（`NotFound(String)`），每层只说自己层的错误；`From` 只包装不拆解
- 不写 `let _ = result;` 吞错误。确实要忽略，必须写清楚为什么
- 用户输入不能直接 `unwrap`；外部数据解析失败一律当「无效」处理，不要 panic
- 错误消息要能定位问题，但登录失败这类要模糊（防用户名枚举）

### 5. 直白优先

- 链式调用超过两步就拆成有名字的中间变量，别让人在脑子里跑一遍
- 不炫技：能用 `for` 看懂就别硬凹 `fold`；能写两行就别写一行超长表达式
- 魔法数字提常量：`const MIN_PASSWORD_LEN: usize = 8;`
- 注释解释「为什么」，不复述「做了什么」；代码本身应该能说明「做了什么」

### 6. 收尾自查（每次收尾都过一遍）

- `cargo check` 没有新增 error / warning（warning 当 TODO，不要用 `#[allow]` 藏）
- `grep -rn "unwrap()\|expect(" src/`，请求路径上的逐个确认
- 每个 `if` 条件代入「正常值 / 异常值」各验一遍（防布尔写反）
- 有副作用的代码（写库 / 写缓存 / 发消息）是否在所有失败分支之后？
- 新增迁移是否注册到 `migration/src/lib.rs`？
- 新增的 service / handler / router 三处是否都接线了？（别留「文件存在但不参与编译」）

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
