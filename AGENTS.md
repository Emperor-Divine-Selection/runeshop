# runeshop 重构任务书（临时文档）

> ⚠️ **这份文档不是项目总览。** 项目总览在工作区根目录 `/home/steven/Projects/AGENTS.md`（四个仓库共用）。
> 这里只记**当前正在进行的重构**：目标、决策、进度、下一步。重构完成后请删掉本文件，并把结论合并回根文档。

---

## 一、我们在干什么

把现在的单体后端，**分阶段改造成分布式架构**。

**目的不是"解决臃肿"，是学习分布式。** 项目本身只有 3300 行、9 张表、最大文件 137 行，远没到需要拆的规模。
拆的唯一理由是：作者想借这个项目把分布式真正学会，而不是只看书。

所以评价标准不是"架构是否最优"，而是：

1. 每一步能学到什么真实的分布式问题
2. 项目不会因为复杂度失控而死掉

---

## 二、铁律（违反任何一条都会让项目死）

1. **每个阶段结束时，三个前端必须还能正常跑。** 不允许 big-bang 重写。
2. **一次只引入一个变量。** 不要同时改"网络"和"数据"；不要同时改"结构"和"业务"。
3. **任何一步卡住超过 3 天 → 回滚到上一个绿的状态**，重新想。
4. **最难的事先做，趁系统还最简单。** 改数据模型 > 拆 schema > 拆 crate > 服务化。

---

## 三、已确定的目标形态

```
浏览器 ──HTTP/JSON──> api (:3000) ──gRPC──> identity (:3001)
                            └──────gRPC──> merchant (:3002)
                            └──────gRPC──> catalog  (:3003)
```

| 项 | 决定 |
|---|---|
| 数据库 | **单个 PG 实例，多个 schema**（不是多个实例） |
| schema | `identity` / `merchant` / `catalog` |
| 服务间通信 | **gRPC**（但阶段 1 先用 HTTP，阶段 1.5 再换 gRPC） |
| 异步 | Redis Streams（不用 apalis，要看裸的 ack 语义） |
| 网关 | `api` 兼任：唯一说 HTTP 的进程，内部全说 gRPC |

**为什么单库多 schema**：本地开发友好（不用起 3 个 PG）。但它**不提供数据归属强制** —— 跨 schema JOIN 照样能写。
真正断开靠「每个 schema 一个 PG role，服务用自己的 role 连」（阶段 3 做）。

### 领域划分（重要，别按"登录/用户/商品"拆）

| schema | 表 | 说明 |
|---|---|---|
| `identity` | `customers`、`operators` | 顾客账号 + 运营账号 |
| `merchant` | `merchants`、`merchant_applications` | **商户账号 + 入驻审核**，自包含 |
| `catalog` | `products` + 4 张子表 | 商品 |

**枚举类型跟着表放在各自 schema**（`merchant.merchant_status` / `merchant.application_status` / `catalog.product_status`）。

踩过的两个坑，都别再踩：

1. **PG 建表时不会用 search_path 去别的 schema 找类型**
   ```sql
   CREATE SCHEMA _t; CREATE TYPE _t.st AS ENUM ('a','b');
   CREATE TABLE _t.t1 (s st);   -- ERROR: type "st" does not exist
   ```
   所以 `enumeration("status", "merchant_status", ...)` 用不了，列类型必须写全限定名：
   ```rust
   ColumnDef::new(Alias::new("status"))
       .custom("merchant.merchant_status")   // ColumnType::Custom 是原样输出的
   ```
2. **`sea-orm-cli generate entity -s <schema>` 会用表的 schema 解析枚举名**
   枚举放 `public` 时生成直接失败（`type "merchant_status" does not exist`）。
   所以枚举**必须**和表同 schema。
3. **运行时也要 search_path**
   sea-orm 拼的 SQL 里是 `::$1::application_status`（不带 schema），
   所以 `create_schemas` 迁移里用 `DO $$ ... ALTER DATABASE ... SET search_path ...` 把
   `identity, merchant, catalog, public` 设到库级别。

   （将来每个服务用自己的 PG role 时，改成 `ALTER ROLE <svc> SET search_path = <自己的 schema>, public`）

**⚠️ 不要单独拆"登录服务"。** 登录和用户共用 `users` 表，拆成两个服务 = 两个进程写同一张表，这是反模式。
真正可独立的是**会话服务**（只碰 Valkey，不碰 PG）—— 但那会给出站请求加一跳，留到阶段 5 当取舍练习。

---

## 四、路线图

| 阶段 | 做什么 | 进程 | 数据库 | 新出现的难题 | 状态 |
|---|---|---|---|---|---|
| **1** | 商户账号独立 + 入驻搬端 + 补正 | 1 | 1 | 状态机、token 分端 | ✅ 后端完成，前端待改 |
| 2 | 拆 schema（identity / merchant / catalog） | 1 | 1（3 schema） | 逻辑隔离、迁移分家 | ✅ 已完成 |
| 3 | 拆 crate（common / identity / merchant / catalog / api） | 1 | 1 | 边界（编译器守） | ⬜ |
| 4 | catalog 独立进程（HTTP） | 2 | 1 | 网络失败、超时、trace id | ⬜ |
| 5 | 同一接口换 gRPC | 2 | 1 | proto 契约、codegen、Status 映射 | ⬜ |
| 6 | worker + Redis Streams（头像处理异步化） | 3 | 1 | 异步、幂等、重复投递 | ⬜ |
| 7 | 每个 schema 一个 PG role | 3 | 1 | 真正的数据归属（跨域查询报错） | ⬜ |
| 8 | 审核通过变 Saga | 3 | 1 | 补偿、对账 | ⬜ |
| 9 | 会话服务独立（auth 拆出去） | 4 | 1 | 该不该拆的取舍、JWT | ⬜ |

阶段 2、3 是**行为零变化**的机械活。阶段 1 是唯一有行为变化的，所以排最前。

---

## 五、⚠️ 与根 AGENTS.md 冲突的地方（以本文为准）

根文档里有几条结论**已经被本次重构反转**。下一个 AI 看到矛盾时，按下面这张表走：

| 根文档的说法 | 现在的决定 | 为什么反转 |
|---|---|---|
| 「商户不另起表：商户本质还是用户」 | **商户独立账号表**，有自己的 username / password_hash / email | 商户不需要以顾客身份买东西；独立后登录从查 2 张表变 1 张 |
| `merchants.user_id` 唯一外键指向 `users` | **删除**，商户不再引用顾客 | 同上 |
| 「商户端登录查 2 张表（users + merchants）」 | 只查 `merchants` | 同上 |
| `application_status` = pending / approved / rejected（一次性终态） | **加 `supplement_required`**，可回退到 pending | 新增「要求补正」流程 |
| `merchant_applications.user_id` 唯一约束 | **删除**，改成 `username` 的部分唯一索引 | 申请不再要求顾客登录 |
| 顾客端 `/apply` 五态页面 | **砍掉**，入驻流程搬到商户端 | 商户是商户，用户是用户 |
| 审核链路「apply 由顾客发起」 | apply 变成**公开接口**，不需要登录 | 同上 |
| 表名 `backoffice_users` | **改名 `operators`** | `backoffice` 说的是入口，`admin` 说的是权限，表名该说「人是谁」；和 `users` / `merchants` 对仗 |
| 表名 `users` | **改名 `customers`** | 三张账号表统一按「人是谁」命名：`customers` / `merchants` / `operators` |

根文档这些章节等阶段 1 完成后要一并更新，别让两份文档长期打架。

---

## 六、阶段 1 的详细设计

### 6.1 目标

- 商户从 `users` 里独立出来，有自己的账号密码
- 入驻申请从顾客端搬到商户端，变成**公开接口**（不需要登录）
- 新增「要求补正」状态，申请人可以改资料后重新提交
- 顺带还掉技术债：`verify_password` 的三份副本合成一份

### 6.2 数据模型

```sql
merchants
  id            uuid PK default gen_random_uuid()
  username      varchar UNIQUE NOT NULL      -- 商户端登录名
  password_hash varchar NOT NULL
  email         varchar UNIQUE NOT NULL
  shop_name     varchar NOT NULL
  status        merchant_status NOT NULL      -- active | frozen
  create_time / update_time  timestamptz

merchant_applications
  id                uuid PK
  username          varchar NOT NULL          -- 申请人自填，通过后原样搬进 merchants
  password_hash     varchar NOT NULL          -- 申请时就哈希好
  email             varchar NOT NULL
  shop_name         varchar NOT NULL UNIQUE
  status            application_status NOT NULL  -- pending | approved | rejected | supplement_required
  reviewed_by       uuid NULL                 -- FK → operators（保留）
  reviewed_at       timestamptz NULL
  review_remark     text NULL                 -- 由 m20261031 迁移加，保持不动

  -- 同一个用户名同时只能有一条「在流程中」的申请
  CREATE UNIQUE INDEX uq_active_app_username
    ON merchant_applications (username)
    WHERE status IN ('pending', 'supplement_required');
```

**CHECK 约束必须改。** 原来是：

```sql
(status = 'pending' AND reviewed_at IS NULL AND reviewed_by IS NULL)
OR
(status <> 'pending' AND reviewed_at IS NOT NULL AND reviewed_by IS NOT NULL)
```

要求 pending 的 `reviewed_*` 必须为 NULL。但补正后回到 pending 要**保留**「被打回过」的痕迹，所以改成：

```sql
status = 'pending'
OR (reviewed_by IS NOT NULL AND reviewed_at IS NOT NULL)
```

即：只有 pending 不要求审核人，其余三个状态必须有。

### 6.3 状态机

```
提交 ──→ pending ──通过──→ approved（终态）
           │
           ├──驳回──→ rejected（终态）
           │
           └──要求补正──→ supplement_required
                            │
                            └──申请人改完重新提交──→ pending
```

`supplement_required` 时 `reviewed_*` 有值（记录谁要求的），回 pending 后**不清空**。

### 6.4 认证

新增 `TokenKind::Applicant`（和现有 Customer / Merchant / Backoffice 同一套机制）。

**申请人和商户共用一个登录接口** `POST /merchant/auth/login`：

```
先查 merchants 表
  ├─ 找到 → 验密码 → 状态 active 发 Merchant token / frozen 拒绝
  └─ 没找到 → 查 merchant_applications
        ├─ pending             → 403「审核中」
        ├─ supplement_required → 验密码 → 发 Applicant token
        ├─ rejected            → 403「已被驳回：{理由}」
        └─ 都没有              → 401「用户名或密码错误」
```

⚠️ **「用户名不存在」和「密码错误」必须回同一句话**（防枚举，见根文档踩坑 51）。

### 6.5 接口

| 方法 | 路径 | 认证 | 作用 |
|---|---|---|---|
| POST | `/merchant/apply` | 公开 | 提交申请 |
| POST | `/merchant/auth/login` | 公开 | 商户**或**申请人登录，返回 `{ token, role }` |
| GET | `/merchant/application/me` | Applicant | 看状态 + 审核意见 |
| PATCH | `/merchant/application/me` | Applicant | 补正资料（仅当 `supplement_required`） |
| POST | `/backoffice/applications/{id}/supplement` | Backoffice | 要求补正（理由必填） |

**删除**：`POST /customer/application`、`GET /customer/application`。

### 6.6 补正能改什么

| 字段 | 能改 | 理由 |
|---|---|---|
| `shop_name` | ✅ | 主要目的 |
| `email` | ✅ | 联系方式 |
| `password` | ✅ | 顺便允许 |
| `username` | ❌ | 身份标识，通过后要原样搬进 `merchants` |

### 6.7 密码哈希

`verify_password` 现在有三份（`store/user`、`store/backoffice_user`，再加 `store/merchant` 就是三份）。
抽到 `src/service/common/password.rs`：

```rust
pub async fn hash_password(plain: &str) -> Result<String, ServiceError>;
pub async fn verify_password(stored_hash: &str, plain: &str) -> bool;
```

三处都改成调它。这一步**单独做完先编译一次**，别和别的改动混。

### 6.8 执行顺序

```
1. 改 2 个迁移文件（merchants / merchant_applications）
2. docker compose down -v && up -d && migrate up
3. 验证表结构 + 枚举 + 索引 + 约束（故意插一条非法数据看是否报错）
4. sea-orm-cli generate entity 重新生成 model
5. 抽 service/common/password.rs → cargo check 一次
6. 改 store 层
7. 改 service 层（删 customer/application，新建 merchant/application）
8. 改 cache（TokenKind::Applicant）/ handler / router
9. 改前端（storefront 删 /apply；merchant 加 /apply 和 /apply/status；backoffice 加补正按钮）
10. 逐条验收
```

**注意：第 1~4 步做完之前 `cargo check` 必然是红的，这是预期的，不是出错。**

---

## 七、验收清单（阶段 1 做完必须逐条点）

```
□ 商户端提交申请 → 立刻用同一账号登录 → 看到「审核中」
□ 后台点「要求补正」填理由 → 申请人登录 → 看到理由 + 能改资料
□ 改完提交 → 状态回「待审核」
□ 后台通过 → 申请人重新登录 → 进商户中心，看到自己的店铺
□ 后台驳回 → 申请人登录 → 看到驳回理由
□ 被拒后可以用同一个用户名重新申请
□ 顾客端没有 /apply 了，所有旧链接不 404
□ 三端登录互不串（顾客 token 打商户端 → 401）
□ 头像上传仍然正常（确认没被误伤）
□ 建商品仍然正常（确认没被误伤）
```

---

## 八、环境速查

```bash
# 基础设施
cd runeshop && docker compose up -d

# 重建数据库（本地无真实数据，允许重写迁移，见根文档踩坑 93）
docker compose down -v && docker compose up -d

# 跑迁移（sea-orm-cli 只认 DATABASE_URL，本项目用 POSTGRES_URL，必须显式 -u）
sea-orm-cli migrate up -u "postgres://runeshop:runeshop@localhost:5432/runeshop"

# 生成 entity
sea-orm-cli generate entity -u "postgres://runeshop:runeshop@localhost:5432/runeshop" --output-dir src/model

# 起后端
cargo run          # :3000

# 三个前端
cd ../backoffice && bun run dev   # :5173 运营后台
cd ../merchant   && bun run dev   # :5174 商家中心
cd ../storefront && bun run dev   # :5175 用户端
```

**端口**：后端 3000 / 后台 5173 / 商家 5174 / 用户端 5175

---

## 九、写代码时的硬约束（摘自根文档，仍然有效）

1. `?` 转错误 + `Ok()` 补壳**缺一不可**
2. 事务里只能传 `&txn`，传 `&db` 编译能过但**回滚失效**
3. 归一化（trim / lowercase）必须在**所有入口**统一做
4. 前端给 Naive 组件设尺寸/背景，用 `style` / `content-style`
5. `cargo check` / `bun run type-check` 全绿 **≠ 逻辑对**
6. 请求路径禁止 `unwrap()` / `expect()` / `panic!()`
7. 有副作用的代码必须在**所有失败分支之后**（别留孤儿 token）
8. 新增迁移必须注册到 `migration/src/lib.rs` 的 `migrations()`

---

## 十、当前进度

**阶段 1 + 阶段 2 的后端部分已完成并端到端验证。前端还没改。**

### 已完成

**① 数据模型**
- 三张账号表统一命名：`customers` / `merchants` / `operators`（`backoffice_users` → `operators`，`users` → `customers`）
- `merchants` 独立账号：删 `user_id`，加 `username` / `password_hash` / `email`
- `merchant_applications`：删 `user_id`，加申请人的 `username` / `password_hash` / `email`
- `application_status` 加 `supplement_required`，CHECK 改成「只有 pending 不要求审核人」
- 部分唯一索引 `uq_active_app_username`（`WHERE status IN ('pending','supplement_required')`）
- 密码哈希三份合一份 → `store/password.rs`

**② 拆 schema**
- `identity{customers, operators}` / `merchant{merchants, merchant_applications}` / `catalog{products + 4}`
- 枚举进各自 schema，库级 `search_path`（见第三节三个坑）
- **删掉 2 条跨 schema 外键**：`products.merchant_id`、`merchant_applications.reviewed_by`

**③ 后端代码**
- `src/model/{identity,merchant,catalog}/` 按 schema 分目录，实体带 `schema_name`
- `TokenKind::Applicant` + `ApplicantUser` 提取器
- 登录合并：商户 / 申请人共用 `POST /merchant/auth/login`，返回 `{ token, role }`
- 新增 `POST /merchant/apply`（公开）、`GET|PATCH /merchant/application/me`（Applicant）
- 新增 `POST /backoffice/applications/{id}/supplement`
- 删除 `/customer/application`（service + handler 全删）

### 验证结果（curl 实测，全部通过）

```
提交申请                        201 + status=pending
重复提交                        409
申请人登录（pending）           400「申请正在审核中」
密码错                          401「用户名或密码错误」（和用户名不存在同一句）
运营要求补正                    200 status=supplement_required
申请人登录 → applicant token    200 role=applicant
查申请状态                      200 带 review_remark
补正资料                        200 回 pending，reviewed_* 保留
pending 下再改                  409「当前状态不允许修改」
审核通过                        200 status=approved，同时建商户
商户用同一套账号登录            200 role=merchant
GET /merchant/me                200 看到店铺
顾客注册 / 登录                 正常
商户建商品                      201
```

`cargo check` 零 error 零 warning。

### 下一步：前端

| 仓库 | 要做什么 |
|---|---|
| **storefront** | 删 `pages/apply.vue` 和所有指向 `/apply` 的链接、API 调用 |
| **merchant** | 新增 `/apply`（公开表单：用户名 + 密码 + 邮箱 + 店铺名）；新增 `/apply-status`（看状态 + 补正）；登录按 `role` 跳转 |
| **backoffice** | 申请列表加「要求补正」按钮 + 弹窗；「申请人」列改用 `username` / `email`；状态筛选加 `supplement_required` |

### 再下一步

阶段 3：拆 crate（common / identity / merchant / catalog / api）。

### ⚠️ 重新生成实体时注意

```bash
# 三个 schema 分开生成，再手工合并到 src/model/<schema>/
sea-orm-cli generate entity -u "$U" -s identity --output-dir /tmp/gen/identity
```

- 生成物里的 `prelude.rs` 已在仓库中**删除**（没人用，只会产生 unused warning）；
  重新生成后要么删掉、要么在 `mod.rs` 里去掉 `pub mod prelude;`
- `src/model/mod.rs` 是**手写**的，别让生成覆盖
- 生成后确认每个实体都有 `#[sea_orm(schema_name = "...")]`
