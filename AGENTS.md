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
    └── store/              # 存库层
        ├── mod.rs          # 只放 pub mod 声明，一行一个，不写逻辑
        ├── errors.rs       # StoreError（属 store 层）
        └── user/
            └── mod.rs      # users 表的方法
```

**后续规划**（用到才建，不预先造空目录）：

```
src/server/     # HTTP 层：handler + Request/Response DTO + ApiError
src/cache/      # 缓存层：fred 封装
```

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
- **泛型挡不住"误传池"**（见踩坑 37），真正的防线是结构

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
- [x] `src/store/errors.rs`：`StoreError` 五变体 + `Display`（`[变体名]` 前缀）+ `From<DbErr>`
- [x] `.gitignore` 补 `*.env`（防密码入 git 历史）

### 🚧 进行中 / 下一步（线性竖切：一次一条链路）

- [ ] **store 第一刀**：`find_by_username` / `find_by_email`（泛型 `C: ConnectionTrait` 签名）
- [ ] **store 第二刀**：`create_user`（ActiveModel + `Set`，avatar/bio 走 `Set(None)` 验证可空写入）
- [ ] **server 层**：`src/server/` + `ApiError`（`StoreError` 变体 → HTTP 状态码映射）
- [ ] **第一个真实接口**：`POST /users` 注册（校验 → argon2 哈希 → 先查后插判 409 → 201 + 脱敏 DTO）
  - 决策已定：先查后插（不解析 DB 错误文本）/ DTO 脱敏（不返 password_hash）/ 201 Created
- [ ] `/health` 接口 + `TraceLayer`（`tower-http` 装了还没用）
- [ ] cache 层（fred，cache-aside：读→miss→查库→回填；写→写库→失效缓存）

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
44. **层依赖方向**：`server`（外层）依赖 `store`（内层）。反过来的信号是**某个内层模块 import 了 AppState** —— 立刻警惕

### 错误处理

45. **错误类型按层归属**：`StoreError` 属 store 层（`src/store/errors.rs`），`ApiError` 属 web 层（`src/server/`）。`ApiError` 包装 `StoreError`（`From` 自动转），方向是 web → store，本来就对
46. **错误变体要带 payload**：`NotFound(String)` 而不是光秃秃的 `NotFound` —— 否则只能说"目标资源未找到"，说不出缺哪个。前五个变体（除 `Technical` 包 `DbErr`）统一带 `String`
47. **`Display` 用 `[变体名]` 前缀 + payload 写完整句子**：`write!(f, "[Conflict] {msg}")`。前缀负责分类（日志可 grep），payload 负责细节，**不要重复分类词**（`[Conflict] 冲突：xxx` 是废话）
48. **`From<DbErr>` 让 `?` 自动包装**：store 方法体内的 DB 错误路径零改动。但 sea-orm 的 `one()` 返 `Result<_, DbErr>`，仍需 `.map_err(StoreError::from)` 才能交给 `?`

## 约定

- 镜像名 / 服务名 / 容器名一律小写
- 目录 / 模块 / 文件 / 函数一律 `snake_case`；类型 `UpperCamelCase`
- 依赖一律用 `cargo add`，不手写 Cargo.toml 版本
- 能用 sea-query DSL 表达的，**不手写原生 SQL**（原生 SQL 换库即废、拼错编译期不报错）
- 历史迁移**不能改**，只能追加新迁移
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
