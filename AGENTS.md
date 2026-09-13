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
├── migration/              # 独立 crate：数据库迁移（17 张表 + 性能索引）
└── src/
    ├── main.rs             # 入口：连接 DB + 验证查询（Axum 服务待加）
    ├── config/
    │   └── mod.rs          # Config 结构体 + new() 加载
    ├── model/              # entity 层：17 张表全部生成完毕（sea-orm-cli 自动）
    │   ├── mod.rs          # entity 模块声明
    │   ├── prelude.rs      # 别名（Entity as Users 等）
    │   └── users.rs        # users 实体（含 Relation / Related / ActiveModelBehavior）
    └── store/
        ├── mod.rs          # store 模块声明（user_store + wallet_store）
        ├── user_store.rs  # UserStore：users 表 CRUD
        └── wallet_store.rs # WalletStore：wallets + wallet_transactions（含事务方法 charge/deposit）
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
- [x] git 推送到 GitHub：SSH 密钥 + 443 端口方案（`Emperor-Divine-Selection/runeshop`）
- [x] 商品规格体系：spec_dims / spec_values / product_variants / variant_values（SPU-SKU 模型，N:N 中间表）
- [x] 钱包体系：wallets（一人一钱包）+ wallet_transactions（流水）
- [x] 经验体系：users.xp 列 + xp_records（经验流水）
- [x] 会员体系：member_levels（字典表）+ user_memberships（关系表）
- [x] 商户体系：merchants + merchant_accounts（敏感信息独立表）
- [x] **数据层完工：15 张表全部 migrate + 测试数据写入**
  - 表：users/products/orders/order_items/spec_dims/spec_values/product_variants/variant_values/wallets/wallet_transactions/xp_records/member_levels/user_memberships/merchants/merchant_accounts
- [x] 性能索引：add_performance_indexes（8 个 btree 索引，外键列）
- [x] 用户地址：user_addresses（1:N，is_default 应用层保证，不建部分索引）
- [x] **数据层封顶：16 张业务表全部 migrate 完成**（另有 merchant_addresses，共 17 张）
- [x] **entity 全量重生成：17 张表 model 完成**（users/products/orders/order_items/spec_dims/spec_values/product_variants/variant_values/wallets/wallet_transactions/xp_records/member_levels/user_memberships/merchants/merchant_accounts/merchant_addresses/user_addresses）
- [x] **store 层开工：UserStore 完成 users 表 CRUD**
  - 增：add_user（ActiveModel + Set + ..Default::default()，数据库默认值接管 created_at 等）
  - 删：delete_by_id / delete_by_username / delete_by_email（delete_many().filter().exec()）
  - 改：update_username_by_id / update_password_hash_by_id / update_email_by_id / update_avatar_url_by_id / update_bio_by_id（查→into()→Set→update 四步流程）
  - 查：find_by_id / find_by_name / find_by_email（one() 返 Option，查不到≠错误）
- [x] **UserStore 领域拆分成型**（src/store/user_store/ 目录模块）
  - mod.rs：UserStore struct + users 表方法（增删改查 9 个，含 add_user 的 Conflict 挂账注释）
  - addresses.rs：list_addresses / add_address / delete_address / update_address（AddressUpdate 参数对象，patch 模式：Some 才 Set）/ set_default_address（事务：先清场 update_many + Condition::all，再上台；含越权校验）
  - memberships.rs：已建待填（list_memberships / grant_membership）
  - 重构实录：impl 可跨文件但兄弟模块看不到私有字段（db 字段经历 私有→pub(crate)→回 mod.rs 恢复私有）；mod.rs 用 pub use 保持对外路径不变；最终形态 = struct 在 mod.rs、按功能域拆文件（社区主流：不为对称性拆，大到难受才拆）
- [x] 踩坑实录入册：见踩坑记录 19-23
- [x] **WalletStore 完工**（wallets + wallet_transactions，7 个方法）
  - create_for_user（一人一钱包，balance 初始 Decimal::ZERO；user_id 有 unique 约束，重复开钱包会 insert 报错——upsert 场景留待后续）
  - find_by_id / find_by_user_id（双实体 import 用别名：Entity as Transactions / Model as TransactionModel / Column as TransactionsColumn）
  - list_transactions（第一个 .all() 批量查，返 Vec<TransactionModel>，空结果=Ok(vec![])）
  - **charge（全项目第一个事务方法）**：begin() → 事务内查钱包(&txn 铁律) → 余额检查(不足则 early return 自动回滚) → 扣余额 → 插负数流水 → commit()
  - **deposit**：charge 的镜像（无余额检查、流水记正数、加法）
  - 事务设计决定：amount 参数约定传正数，内部扣款取负写流水；tx_type 暂用 &str，将来换枚举
  - 两个事务方法已带参数守卫（amount <= 0 → InvalidInput）
- [x] **StoreError 业务错误类型**（src/store/error.rs）
  - 变体：NotFound / Business / InvalidInput / Conflict / Technical(DbErr)——技术错误与业务错误分家，将来 server 层按变体译成不同 HTTP 状态码，不用解析错误文本
  - From<DbErr> 让 ? 自动把 DbErr 包成 Technical，store 方法体内的错误路径零改动
  - Display 实现人类可读输出；**UserStore / WalletStore 全部方法已切到 Result<_, StoreError>**（查询方法保持 Option 表达“查不到≠错误”）
  - 挂账：add_user 的 unique 冲突识别（→ Conflict）留待做注册接口时处理

- [x] **补外键：user_memberships.level_id → member_levels.id**（迁移 m20260907_154853_add_fk_user_memberships_level，RESTRICT 保护字典表；原因：原建表迁移漏建了 level_id 外键，历史迁移不能改，只能追加新迁移）

### 🚧 进行中 / 下一步（自底向上：先把后端层盖完，再做 server）

- [ ] **store 层铺齐**：UserStore 补 user_addresses/user_memberships/xp_records；剩余 ProductStore（products/spec_dims/spec_values/product_variants/variant_values，5 张表最大）/ OrderStore（orders/order_items）/ MerchantStore（merchants/merchant_accounts/merchant_addresses）
  - 注意：字典表（member_levels/spec_dims/spec_values）读多写少，只配查就够，不配增删改
  - 事务方法候选（领域方法，非单表 CRUD）：add_xp+流水（UserStore）、订单下单多表写入（OrderStore）
- [ ] **复杂方法留白区**（铺齐 CRUD 后再评估）：列表/分页/count、事务方法（钱包扣款+流水、add_xp+流水是首批）、跨表 join 查询——做到哪层需要再写，不凭空预写
- [ ] cache 层（Valkey 缓存封装，cache-aside：读→缓存 miss→查库→回填；写→写库→失效缓存）
- [ ] valkey 连接验证（`PING` → `PONG`）
- [ ] `main.rs` 完整化（**放最后**）：Axum + 共享状态（AppState 装 DatabaseConnection + 各 store + cache）+ `/health` + 第一个真实接口
- [x] ~~待查证：user_memberships 的 level_id 外键是否在迁移里漏建~~（已实锤并修复，见上方“补外键”条目；后续会重新生成 entity 同步 Relation）

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
17. **GitHub SSH 22 端口被墙**：改用 443 端口（`ssh.github.com:443`），写入 `~/.ssh/config`（Host github.com → HostName ssh.github.com / Port 443 / User git）
18. **sea-orm-cli generate 只做一半**：自动加 `mod` 声明，但 `migrations()` 列表要手动注册，否则迁移被静默忽略（`migrate up` 成功但表没建）；生成后检查 `lib.rs`，用 `\dt` 兌底验证

### Store 层 / sea-orm 实战

19. **函数参数不能用裸 `str`**：str 是不定长类型（DST），函数参数编译期必须确定大小；`&str`（胖指针：起始地址+长度）才是“只读借用”的正确写法，调用方传 String/字面量都不被消耗
20. **枚举变体用双冒号路径**：`Column` 是枚举类型，具体列是 `Column::Username`——`Column.eq(...)` 报 `expected value, found enum`；同套路还有 `Option::Some`、`Order::Desc`
21. **ActiveModel 三态 + Set 不在 prelude**：Set(Some(v)) / Set(None) / 不碰（into() 自带的 Unchanged，UPDATE 不含该列）；`Set` 要手动 `use sea_orm::Set;`（prelude 只收高频通用项，缺 import 报 `cannot find in scope` 就补 use）
22. **Option 字段写库要包一层**：数据库可空列（avatar_url/bio/xp）在 Model 里是 `Option<T>`，Set 时必须 `Set(Some(...))`；想写 NULL 用 `Set(None)`——不设字段 ≠ 写 NULL
23. **复制粘贴是字段名 bug 的温床**：六个 update_xxx_by_id 复制后忘改字段名，全写成了 password_hash，且 cargo check 照样绿（类型恰好相同）——模板代码写完逐字段自查，能用参数化/收拢就不复制
24. **ActiveValue 不能直接做算术**：into() 后的 `wallet.balance + amount` 报 `cannot add ActiveValue<Decimal>`——裸值先算好再 into() 再 Set（"先算后装"），还能顺便避开 unwrap 拆包
25. **glob import 撞名要靠别名**：两个模块的 `use xxx::*` 同时带进 Model/Entity/Column，报 `Model is ambiguous`——主实体 glob、次要实体用 `{self, Entity as Transactions, Model as TransactionModel, Column as XxxColumn}` 点名加别名
26. **类型名必须大写开头**：Vec/Option/Result/自定义类型大写，小写开头是变量/函数，`vec` 写进类型位置直接报 `cannot find type`
27. **金额一律 Decimal，没有字面量**：`Decimal::ZERO` / `Decimal::ONE`，一般金额从字符串 parse（`"19.99".parse::<Decimal>()`）；绝不用 f64 存金额（精度误差）
28. **事务里的查询必须展开重写**：不能调 store 自带的 find_xxx（内部写死 &self.db），事务内所有 SQL 显式用 &txn；"先查后改"的领域方法将来可抽公共连接参数（&txn 或 &self.db）消除重复
29. **构造器命名跟标准库**：Rust 无构造函数，惯例 `Type::new()`（String::new/Vec::new），init 在 Rust 语境里是"对已有实例初始化"，不是构造

## 约定

- 镜像名 / 服务名 / 容器名一律小写
- 依赖一律用 `cargo add`，不手写 Cargo.toml 版本
- 本地开发凭据（`runeshop`/`runeshop`）仅限本地，上线前必须更换为环境变量注入
- `.env` 不入 git（`.gitignore` 已加 `*.env`）
- 代码保持格式化后再保存（Zed 已开启自动格式化）
