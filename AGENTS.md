# runeshop

基于 Rust 的电商项目（**学习项目**，主要目的是**学透服务间通信**）。

> ## 📌 这份文档的定位
>
> 它是**规格书 + 踩坑清单**，不是「项目说明书」：描述**要建成什么样**，不是**现在长什么样**。
>
> **组织方式是一根竖线**：§2 的拓扑 → §4 认证 → §5 信封 → §9 中间件 → §10 八步 → §11 表归属，
> 一路从浏览器切到数据库。进度也按这根线推进（§10 定第一刀，§16 排后续刀），
> 不按「技术栈」「模块」横着铺。找一个决策点时，先问「它在这根线的哪一层」。
>
> **章节号与《定稿总表》1–15 一一对应**，所以总表里那句「以 `AGENTS.md` 第十一节为准」指的就是 §11。
> 总表与本文件冲突时，以总表 + 本文 §11 为准（本文是总表的展开，不是第二个真相）。

---

## 0. 当前状态（2026-11-13，先读这段）

**架构定稿；骨架已落地并跑通，第一刀的表还没建。**

2026-11-12 的定稿推翻了 11-11 晚那版骨架的四条口径：

| 旧口径（11-11 晚） | 定稿（11-12） |
|---|---|
| 四个服务含 `merchant` | **没有 `merchant` 服务**，第四个是 `order` |
| 库名 `identity` / `merchant` / `catalog` | `identity_db` / `catalog_db` / **`order_db`** |
| 内部 JWT（HS256，60s）网关签、各服务验 | **Session token 存 Valkey**，gateway 问 identity，下游只信 metadata |
| 信封 `code` = HTTP 状态码 | **`code: 0` = 成功，与 HTTP 无映射** |

2026-11-13 落地的骨架（§0.1 那张表是历史对照，现状见这里）：

- 虚拟 workspace（根 `Cargo.toml` 无 `[package]`）：`crates/common`、`crates/identity`、`migration-identity`
- `docker-compose.yml`：postgres 17 + valkey 8；**没有 initdb 挂载**，也没有 jaeger / garage
- `migration-identity` 的二进制已跑通：连 `runeshop_ops` → `CREATE DATABASE identity_db` → `Migrator::up`（迁移列表还是空的，所以只有 `seaql_migrations`）

**第一刀（`customers` 八步，见 §10）卡在第 1 步：`customers` 表还没建。**

### 0.1 骨架要改的地方

11-11 晚那版骨架（workspace + 六 crate + docker compose）如果还在别的副本上，动手前先按这张表对齐：

| 旧 | 新 |
|---|---|
| `crates/merchant/` | `crates/order/` |
| `crates/migration/`（一个 crate 跑三个库） | 根级 `migration-identity/`、`migration-catalog/`、`migration-order/`（三份，因为 `seaql_migrations` 每库一张、互不相通） |
| postgres 建 `identity` / `merchant` / `catalog` | 建 `identity_db` / `catalog_db` / `order_db` |
| 端口 3000 业务 / 3001–3003 服务 | **8080** 业务 + 50051–50053 gRPC + 8081–8083 只挂 `/healthz` |
| jaeger 容器 + OTel | **删掉**（§12 明确不装），用 `tracing` + `trace_id` 串 |
| `INTERNAL_JWT_SECRET` | `SESSION_TTL_SECONDS`（token 真身在 Valkey） |

⚠️ **建库不再走 initdb**（11-13 改口径）：三个库由 `migration-identity` 的 `ensure_database` 建，
所以「改库名」不再需要 `down -v`。但 entrypoint 那条规则本身仍成立 ——
卷非空时**整个初始化阶段跳过**（踩坑 117），现在只影响 `POSTGRES_DB=runeshop_ops` 这一个维护库。

### 0.2 时间线

| 日期 | 发生了什么 |
|---|---|
| 2026-11-10 | 重构：商户独立账号（脱离 `customers`）+ 表按 schema 拆 + 补正流程，后端全链路 curl 验证通过 |
| 2026-11-11 | 决定**推倒重来**，按分布式架构从零写 |
| 2026-11-11 晚 | 第 1 项骨架：workspace（六 crate）+ docker compose（三库 / valkey / garage / jaeger） |
| 2026-11-12 | **定稿**：三库改名 `*_db`、`merchant` 服务换成 `order`、Session+Valkey 取代内部 JWT、信封 `code:0`、库存挪进 order 库、第一刀改成 `customers` |
| 2026-11-13 | 骨架落地：虚拟 workspace（3 个 crate）+ postgres/valkey。**建库改由 migration 二进制负责**（放弃 initdb），`identity_db` 与 `seaql_migrations` 已建出。运营后台改名 `backoffice/` → `operator/` |

**为什么 11-11 要推倒**：目标是学分布式。渐进重构里「为什么先做单体」那一课已经上完，
而 schema 那三个坑证明 —— **边改边拆会同时付「重构税」和「新架构税」**。

**为什么 11-12 又改架构**：写代码前把「认证 / 信封 / 一致性边界」三件事一次定死，
比写完再改便宜一个数量级。这三条恰好是**穿透所有服务**的横向决策，后改会动到每一层。

### 0.3 旧代码在哪

2026-11-13 起 `master` 已被新骨架 force push 覆盖，`git clone` 拿到的是新代码。
旧实现（单体、9 张表、`model/store/service/handler` 分层）存档在 **tag `legacy-monolith-20261111`**（`5e6132e`）：

```bash
git fetch origin tag legacy-monolith-20261111
git switch --detach legacy-monolith-20261111
```

用 tag 而不是分支，是为了让 `git branch -a` 保持干净；真不要了就
`git push origin --delete legacy-monolith-20261111`。

- `.env` / `garage.toml` / `docker-compose.yml` 的旧副本原在 `/tmp/runeshop-backup-20261111/` —— **已被清**（`/tmp` 不持久）

### 0.4 五件别做的事

| ❌ 别做 | 为什么 |
|---|---|
| 别去「修」三个前端 | 它们连不上后端是**预期**的（旧后端删了、新的还没写）。等 §10 走通再动 |
| 别重新用「一个库三个 schema」 | 跨 schema `JOIN` 合法、枚举要跟表同 schema、列类型要全限定名、运行时还要 `search_path` —— 三条独立库把这些税全免了 |
| 别一次写完八步再编译 | proto + tonic-build 一层套一层，一次写完的错误互相掩盖。**每加一层 `cargo check` 一次**（踩坑 118） |
| 别把旧项目的 `store/service/handler` 结构照搬 | 旧结构是**一个进程内的分层**；现在是**每个服务一套**分层，跨服务一律走 gRPC |
| 别在没扫 §18 的情况下动 sea-orm / fred / axum | 那几段的错误都长得像「逻辑错误」，实际是库的坑（列默认 NOT NULL、`connect()` 不能 await、提取器顺序……） |
| 别把 token 放进 `localStorage` | 定稿是 httpOnly Cookie（§4.2）。localStorage 会被 XSS 偷，且 Nuxt SSR 读不到（踩坑 104） |

### 0.5 五条最容易犯的错

1. `?` 转错误 + `Ok()` 补壳**缺一不可**（踩坑 54）
2. 事务里只能传 `&txn`，传 `&db` 编译能过但**回滚失效**（踩坑 100）
3. 归一化（trim / lowercase）必须在**所有入口**统一做（踩坑 61/62/86）
4. `cargo check` 全绿 **≠ 逻辑对**，更 **≠ 能跑**（踩坑 82）
5. 前端给 Naive 组件设尺寸/背景，用 `style` / `content-style`；Tailwind 类会被组件 CSS-in-JS 盖掉

---

## 1. 部署形态：一个 PG 17 实例，三个 database

```
postgres:5432
├── identity_db   账号（customers / merchants / operators）+ session 之外的身份数据
├── catalog_db    商品
└── order_db      订单 + 库存
```

规则：

- 库由 **migration 二进制**建：`ensure_database` 连维护库 `runeshop_ops`（即 `POSTGRES_DB`），
  `SELECT 1 FROM pg_database` 后按需 `CREATE DATABASE` → **建库和建表是同一次部署步骤**（见 §1.2）
- 不走 `docker-entrypoint-initdb.d`：那条路只在数据目录为空时执行一次，加库 / 改库名必须 `down -v`，等于删数据
- **migration 既建库也建表** —— 旧口径「migration 只建表不建库」已于 11-13 作废
- **没有跨库 JOIN、没有跨库外键、一个事务只在一个 database 内**
- 隔离的是**数据**，不是故障：WAL、共享缓冲区、进程都是实例级共享

> 和「一个库三个 schema」的差别只有一条，但值钱：**PG 不能跨库查询**，边界是物理的；
> schema 的边界只能靠自觉（PG role 才拦得住）。

### 1.1 本地容器（docker compose）

| 服务 | 镜像 | 端口 | 说明 |
|---|---|---|---|
| postgres | `postgres:17-alpine` | 5432 | 用户/密码 `runeshop` / `runeshop`；`POSTGRES_DB=runeshop_ops` 是**维护库**，三个业务库由 migration 建 |
| valkey | `valkey/valkey:8-alpine` | 6379 | session 存储（以后做队列也用它），无密码（仅本地） |

```yaml
# docker-compose.yml 里决定「怎么连」的几行
environment:
  POSTGRES_USER: runeshop
  POSTGRES_PASSWORD: runeshop
  POSTGRES_DB: runeshop_ops   # 维护库，不是业务库
  PGDATABASE: runeshop_ops    # psql 不带 -d 时落这里
  PGUSER: runeshop            # psql 不带 -U 时不会去找 root（踩坑 123）
```

`runeshop_ops` 这名字是刻意的：镜像在 `POSTGRES_DB` 缺省时会建一个**与用户名同名**的库
（`file_env 'POSTGRES_DB' "$POSTGRES_USER"`），叫 `runeshop` 的话看起来像业务主库，实际是管理落脚点。

### 1.2 建库 + 建表 = 一次部署步骤

```
① postgres 启动                → entrypoint 建 POSTGRES_DB=runeshop_ops（仅空卷）
② migration 二进制（本地 cargo run / 容器里 migrate 服务）
     ensure_database           → CREATE DATABASE identity_db
     Migrator::up              → CREATE TABLE customers ...
③ identity / catalog / order 启动
```

- **幂等**：`SELECT 1 FROM pg_database` + `seaql_migrations` 双重保证，每次部署都能重跑
- **顺序**：部署时用 `depends_on: migrate: { condition: service_completed_successfully }` 把 ② ③ 隔开
- **禁止**应用启动时自动跑迁移：多副本会并发抢同一批 pending 迁移，sea-orm 不加锁
- **权限**：`CREATE DATABASE` 要 `CREATEDB`（本地 `runeshop` 是超级用户，生产需单独授）

> Garage（对象存储）与 Jaeger（trace UI）的容器已删 —— 见 §12 的「明确不装」和 §15 的未决点。

---

## 2. 服务拓扑与端口

| 服务 | 对外 | gRPC | HTTP | 谁连它 |
|---|---|---|---|---|
| gateway（BFF） | ✅ 公网 | — | **8080** 业务 + `/healthz` | 浏览器、Nuxt SSR |
| identity | ❌ 内网 | 50051 | 8081 只有 `/healthz` | gateway、catalog、order |
| catalog | ❌ 内网 | 50052 | 8082 只有 `/healthz` | gateway、order |
| order | ❌ 内网 | 50053 | 8083 只有 `/healthz` | gateway |

**关键约束**

1. 内部服务**不映射端口到宿主机**，公网不可达 —— 靠网络层切断，**不靠鉴权兜底**
2. 内部服务里的 axum **只挂 `/healthz`**，不挂业务路由。业务全走 gRPC
3. JSON / REST 语义**只存在于 gateway 这一层**

**开发期例外**：四个服务都在宿主机 `cargo run`（不写 Dockerfile，理由见 §12 末尾），
所以本地四组端口都在 localhost。「不对外」这条在**部署时**兑现：容器化只 publish 8080。

前端端口：`operator` `5173`、merchant `5174`、storefront `5175`。

---

## 3. BFF：Rust axum gateway（不是 Nuxt Nitro）

```
浏览器 ──Cookie(httpOnly)──> gateway(axum :8080) ──gRPC──> identity / catalog / order
                                     │
                                  Valkey 不碰
```

- gateway 是 **Rust axum**，不是把 BFF 塞进 Nuxt 的 Nitro
- gateway **不读 Valkey**：session key 的格式与 TTL 只归 identity 所有（改了不用通知第二个人）
- 后端服务**只认 token 字符串**，不认识 Cookie 这个概念 —— Cookie 是 gateway 的方言

**为什么不让前端直连多个服务**：前端一旦知道「后端有几个服务」，加服务 / 换协议 / 换路由
都会传到前端。BFF 是这条防线的唯一实现点，代价是 gateway 承担 REST DTO ↔ protobuf 翻译
（那是 BFF 的本职，不是脏活）。

---

## 4. 认证链路

### 4.1 三个词不在同一层

- **Session / Token** = 凭据本身是什么
- **Cookie** = 凭据放在哪、怎么传
- 本项目 = **Cookie + Token**：浏览器带 Cookie → gateway 取出 token 调 `ValidateSession` → identity 查 Valkey

### 4.2 Cookie 属性（写死）

```
Set-Cookie: session={token}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=86400
```

| 属性 | 作用 | 不许改的理由 |
|---|---|---|
| `HttpOnly` | JS 读不到 | 防 XSS 偷 token |
| `SameSite=Lax` | 跨站 POST 带不上 Cookie | **CSRF 的主防线** |
| `Path=/` | 全站共享 | 少了它只有 `/` 能带 |
| `Max-Age=86400` | 与 Valkey TTL 对齐 | 两处不一致 = 「cookie 还在但服务端已失效」 |

`Strict` 让从外部链接点进来时也丢登录态，太严，不用。

### 4.3 CSRF 三条

1. **`SameSite=Lax` 是主防线**（现代浏览器默认值）
2. **禁止用 GET 做状态变更** —— Lax 允许顶层 GET 导航带 Cookie，所以 `GET /delete?id=1` 是自毁
3. **Origin / Referer 校验只当兜底**：有 `Origin` 校验 `Origin`；没有就看 `Referer`；
   **两个都没有 → 拒绝**（「缺失就放行」等于没做）

只有当 `SameSite=None` 时才必须补 CSRF token —— **本项目不要退到 `None`**。

### 4.4 session 存储

```
SET session:{token} "{role}:{id}" EX 86400
```

- 只存 `role:id`（如 `customer:0199...`），**不存资料**
- 理由：session 是「你是谁」的**凭证缓存**，不是「你的资料」缓存
- `/auth/me` 每次查库拿资料
- 登出 = `DEL session:{token}` —— 天然支持**即时吊销**（改密码踢掉所有设备），JWT 做不到
- token 用 **32 字节 CSPRNG**（`rand` / `getrandom`），base64url 编码；**不要**用 UUID 当凭据

> **为什么放弃内部 JWT**：JWT 零网络调用很好，但代价是吊销有延迟、要处理 `exp`/`nbf` 时钟偏移、
> 密钥轮换要有预案。本项目每个受保护请求反正要多一跳 —— 干脆让 identity 当唯一真相，
> 拿「多一跳」换「没有过期窗口」。**5 秒本地缓存**留作以后的优化，现在不做。

---

## 5. 统一响应信封

```json
{
  "code": 0,
  "message": "ok",
  "data": {},
  "meta": {},
  "trace_id": "..."
}
```

**成功失败同一个壳。**

| 字段 | 规则 |
|---|---|
| `code` | `0` 成功；非 `0` 业务失败。数字业务码，**与 HTTP status 无映射**（禁止 `401 → code 401` 这种复读） |
| `message` | 给人看的文案，可改。前端**不得**依赖它做分支 |
| `data` | 成功放资源；**失败恒为 `null`**；列表为空返 `[]` |
| `meta` | **没有元数据时返 `{}`**，不返 `null`，不许省略字段 |
| `trace_id` | 跨服务定位全靠这一个串 |

- 不加 `success` 字段
- **不用 `204`** —— 信封带不了 body，登出返 `200 + code:0`
- 分页信息进 `meta`：`{total, page, page_size}`，`data` 只放资源
- 金额用**整数分**（`1999` = 19.99 元）
- 时间 **RFC3339 UTC**
- ID 用 **UUID v7**

### 5.1 为什么 `meta` 不能是 `null`

**同一个字段的类型必须恒定。**

- `data` 是**名词**：有没有东西给你，没有就是 `null`，这个信息有意义
- `meta` 是**形容词**：关于这次响应的附注，可以没内容，但不能不在场

三种错法：`meta: null`（前端处处判空）、省略 `meta` 字段（TS 只能写 `meta?:`，全项目 `?.`）、空列表返 `null`。

### 5.2 骨架

```rust
#[derive(Serialize, Default)]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    pub meta: Meta,        // 不是 Option<Meta>
    pub trace_id: String,
}
```

```ts
type Meta = { total?: number; page?: number; page_size?: number }
type Api<T> = { code: number; message: string; data: T | null; meta: Meta; trace_id: string }
```

放 `crates/common/src/response.rs` —— **只有 gateway 序列化它**，内部服务走 protobuf 不碰这个壳。

---

## 6. 业务码（auth 域）

判据：**每一个对外 code，都必须对应一个不同的客户端动作。没有动作差异的码，是噪音。**

```
0     成功
1001  凭证错误          → 留在登录页，清空密码框
1002  未登录 / 登录失效  → 跳登录页
1003  参数不合法        → 表单标红
1004  已被占用          → 表单标红并提示更换
5000  服务内部错误      → 通用错误提示
```

- `token 缺失` / `token 无效` / `token 过期` **合并为 1002**（前端动作相同）
- **对内要清楚，对外要克制**：`StoreError` / `ServiceError` 想分多细分多细（进日志），
  对外只暴露**能改变客户端行为**的（§14）
- HTTP status 独立说真话：`200` / `400` / `401` / `409` / `429`（本期留位）/ `500`

---

## 7. 认证四个接口

统一：JSON、UUID v7、RFC3339 UTC、信封见 §5。

### POST /auth/register

```json
{ "username": "alice", "email": "a@b.com", "password": "********" }
```

- 用户名 3–32 位字母数字下划线；密码 ≥8（`chars().count()`，踩坑 63）
- `201` → `{id, username, email}`
- `409 + code 1004` 重名 / `400 + code 1003` 不合法
- **注册不自动登录** —— 注册和登录各干一件事，出问题能分清是哪一步
- 归一化 `trim()` + `to_lowercase()`：**写和读都要做，且必须一模一样**（踩坑 62/86）

### POST /auth/login

```json
{ "token": "...", "expires_in": 86400, "role": "customer" }
```

- `401 + code 1001` 时**用户名不存在与密码错误返回完全相同的响应**（否则等于送攻击者一个用户枚举接口，踩坑 51）
- 同时 `Set-Cookie`（§4.2 四个属性一个不能少）

### GET /auth/me

- `200` → `{id, username, email, role}`
- 无 / 坏 / 过期 token → `401 + code 1002`
- **没登录是 401 不是 403**：401 = 不知道你是谁，403 = 知道但你没权限

### POST /auth/logout

- `200 + code:0`（**不是 204**）
- **幂等**：token 有效无效都成功
- 同时 `Set-Cookie: session=; Max-Age=0`

> 顺序纪律：**先算 token，再判状态，最后才写外部系统**。
> 反例见踩坑 78 —— 副作用写在判定之前，登录失败却在 Valkey 留了个 24h 的孤儿 token。

---

## 8. proto 契约

命名用 **`Request` / `Response` 成对**（不用 `Reply`：那是老教程的口语，Google API 设计规范是 `Response` 派）。

```proto
syntax = "proto3";
package identity.v1;

service IdentityService {
  rpc Register(RegisterRequest)               returns (RegisterResponse);
  rpc Login(LoginRequest)                     returns (LoginResponse);
  rpc ValidateSession(ValidateSessionRequest) returns (ValidateSessionResponse);
  rpc Logout(LogoutRequest)                   returns (LogoutResponse);
}

message ValidateSessionRequest { string token = 1; }

message ValidateSessionResponse {
  bool   valid   = 1;
  string user_id = 2;
  string role    = 3;
}
```

**proto 铁律：字段号才是身份，字段名不是。**

- 线上字节里传的是 `1: true` / `2: "0199..."`，没有 `user_id` 这几个字母
- 改名 `user_id` → `uid`：安全
- 改号 `user_id = 2` → `= 4`：**老客户端解析错位且不报错**
- 废弃字段号用 `reserved 3;` 焊死

⚠️ 生成的 `ValidateSessionResponse` 会和 `axum::response::Response` 撞名 → `use ... as SessionResp`。

`build.rs` 用 `protoc-bin-vendored` 提供 protoc，**不要求本机装 protoc**；开 `tonic-reflection`，
否则 `grpcurl list` 看不到服务，§10 第 6 步的验收会卡住。

---

## 9. gateway 中间件

```rust
use axum_extra::extract::cookie::CookieJar;

async fn auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = jar
        .get("session")
        .map(|c| c.value().to_owned())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let reply = state
        .identity
        .clone()                        // tonic client 内部是 Channel，clone 很便宜
        .validate_session(ValidateSessionRequest { token })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_inner();

    if !reply.valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    req.extensions_mut().insert(CurrentUser {
        id: reply.user_id,
        role: reply.role,
    });

    Ok(next.run(req).await)
}
```

- 下游（catalog / order）**不再验 token**，只信任 gateway 注入的 `x-user-id` / `x-user-role`（走 gRPC metadata）
- 每个受保护请求多一跳 gRPC —— 以后可以加 5 秒本地缓存，现在不做

⚠️ 上面为了讲清主干省略了信封映射：真实的 `Err` 分支要返 **§5 的壳 + §6 的 code**
（`1002` / `5000`），而不是裸 `StatusCode`。中间件里还要**补 `trace_id`** 到 extension 和响应头。

---

## 10. 竖切八步（第一刀：只切 `customers`）

先切**一张表**，跑通后把同一套实现用到 `merchants` / `operators`。
**三张账号表一起建是横向铺开，不是竖切。**

每一步都是「做什么 → 落点 → 验收判据」，判据必须能**在命令行看见**。

| # | 做什么 | 落点 | 验收判据 |
|---|---|---|---|
| 1 | 建 `customers` 表 | `migration-identity/src/m2026..._create_customers.rs`，并注册到该 crate 的 `migrations()` | `\d customers` 对得上；`select * from seaql_migrations` 有记录（踩坑 21） |
| 2 | 生成 entity | `identity/src/model/customer.rs`，**手动**补 `mod model;` / `mod customer;` | `cargo check -p identity` 输出里有 `Checking identity`（踩坑 118） |
| 3 | Store：`create` / `find_by_username` / `find_by_id` | `identity/src/store/customer.rs`，查询返 `Option` 不返 `NotFound` | 插一行再读出来，字段逐个对（踩坑 33） |
| 4 | Service：argon2 哈希、归一化、登录校验、签 token、写/删 Valkey session | `identity/src/service/customer.rs` + `common/password.rs` | 能打印 `role:id`；`valkey-cli GET session:{token}` 看得到 |
| 5 | proto + tonic server（50051），四个 rpc + reflection | `proto/identity.proto`、`identity/src/grpc/` | `grpcurl -plaintext localhost:50051 list` 列出 `IdentityService` |
| 6 | **`grpcurl` 四连** | 命令行 | 见下方脚本，**这一步不能跳** |
| 7 | gateway（8080）：Cookie 解析 → `ValidateSession` → 注入身份 | `gateway/src/middleware/auth.rs`（§9） | 无 Cookie 访问受保护路由 → `401 + code 1002` |
| 8 | **`curl` 经 gateway 四连** | 命令行 | `Set-Cookie` 四个属性齐全；登出后 `/auth/me` 返 `1002` |

**第 6 步为什么不能跳**：gateway 写完才发现 gRPC 有问题，就是**两个变量一起变**，排不出来。
先把 gRPC 这层钉死，gateway 的 bug 才只可能是 gateway 的 bug。

```bash
# 第 6 步：grpcurl 四连（直连 50051，不经 gateway）
grpcurl -plaintext -d '{"username":"alice","email":"a@b.com","password":"pass1234"}' \
        localhost:50051 identity.v1.IdentityService/Register
grpcurl -plaintext -d '{"username":"alice","password":"pass1234"}' \
        localhost:50051 identity.v1.IdentityService/Login          # 记下 token
grpcurl -plaintext -d "{\"token\":\"$T\"}" localhost:50051 identity.v1.IdentityService/ValidateSession  # valid:true
grpcurl -plaintext -d "{\"token\":\"$T\"}" localhost:50051 identity.v1.IdentityService/Logout
grpcurl -plaintext -d "{\"token\":\"$T\"}" localhost:50051 identity.v1.IdentityService/ValidateSession  # valid:false
```

```bash
# 第 8 步：经 gateway 四连
curl -i -X POST localhost:8080/auth/register -H 'content-type: application/json' \
     -d '{"username":"bob","email":"b@b.com","password":"pass1234"}'
curl -i -c /tmp/ck -X POST localhost:8080/auth/login -H 'content-type: application/json' \
     -d '{"username":"bob","password":"pass1234"}'        # 看 Set-Cookie
curl -i -b /tmp/ck localhost:8080/auth/me                  # 200 + code:0
curl -i -b /tmp/ck -X POST localhost:8080/auth/logout      # 200 + code:0
curl -i -b /tmp/ck localhost:8080/auth/me                  # 401 + code:1002
```

> **「跑通后复制到 merchants / operators」的正确理解**：复用**同一份实现**
> （同一套 store/service 泛型、同一个 argon2 封装、同一个 `ValidateSession`），
> **不是**把文件拷三份。三份逐字节拷贝就是踩坑 80 现场 —— 改密码规则要改三处，忘一处就是 bug，
> 而且编译器一句话都不会说。角色差异只允许体现为**一个参数 / 一张表名**。

---

## 11. 表归属

规则：**哪张表只被一个服务写，就放哪个库；被两个服务写，就是难点。**

| 库 | 表 |
|---|---|
| `identity_db` | `customers`、`merchants`、`operators` |
| `catalog_db` | `products`、`product_kinds`、`product_items`、`product_prices` |
| `order_db` | `orders`、`order_items`、**`product_inventory`** |

### 11.1 库存为什么放 order 库

横梁句：**「一致性边界优先于领域边界」**。

按领域，库存属于商品；按事务，库存必须跟订单在一起。用户原话：

> 就算通信失败了，库存表也不会出现减了，但没有创建订单这种事儿

收益：`扣库存 + 建订单` 变成**一次本地事务**，分布式事务从「需要解决」变成**「根本不存在」**。

代价三条：

1. `inventory.sku_id → catalog.products` **无外键**，靠应用保证 + 对账
2. 补货 / 上架也得走这个服务 → 它应该叫 **trade / order**，不是「订单服务」
3. catalog 展示库存只能**跨库读**

本质是把「写跨库」换成「读跨库」：**写必须强一致，读可以短暂陈旧，这笔非常划算。**

### 11.2 商品列表带库存（方案 3）

列表问 catalog、库存问 order，由 **gateway 聚合**。三条硬前提：

1. 聚合必须在 **gateway**，不能让前端直连两个服务
2. **必须批量接口** `GET /stock?skus=A,B,C`，order 侧不许循环单查
3. **必须三态降级**：有货 / 无货 / **未知**（不报错）

代价：两次请求不是原子快照，不一致**无法消除**，只能 UI 兜底（「刚被抢完」而不是「系统错误」）。

### 11.3 账号三张表，各管一端

| 表 | 是谁 | 密码 | 库 |
|---|---|---|---|
| `customers` | 顾客 | 自己的 | `identity_db` |
| `merchants` | 商户 | **自己的**（不挂在 customers 上） | `identity_db` |
| `operators` | 运营（后台） | 自己的 | `identity_db` |

表名按「**人是谁**」命名，不按「从哪进来」（旧叫 `backoffice`）也不按「多大权限」（`admin`）。

⚠️ 三张表在**同一个库**、**同一个服务**里写 —— 这是定稿的重要简化：
不再有 `merchant` 服务，账号相关的写全在 identity 本地事务里完成，**没有跨服务写**。
但 session 里的 `role` 必须区分三者（踩坑 94：token 不带端类型 = 跨端串用）。

> **未决**：商户入驻审核（`merchant_applications` + 补正状态机）在定稿总表里没有出现，
> 见 §15。在它定下来之前，`merchants` 行怎么产生**不要猜**。

---

## 12. 技术选型

| 项 | 版本 / 决定 |
|---|---|
| axum | 0.8.9 |
| tonic | 0.14.9 + prost 0.14.5 |
| tonic-build | 0.14.9 + `protoc-bin-vendored` 3.3.0（免装 protoc） |
| tonic-reflection | 0.14.9（配 `grpcurl`） |
| sea-orm | 2.0.3（`--no-default-features`，见踩坑 23） |
| PG | 17 |
| Valkey | 8（fred 10.1.0，`connect()` 不能 await，见踩坑 69） |
| tokio | 1.54.1 |
| 错误 | `thiserror` 2.0.21 |
| 重试 | `backon` 1.6.0 |
| 日志 | `tracing` 0.1.44 + `tracing-subscriber` 0.3.23 |
| 配置 | `dotenvy` 0.15.7 |
| 密码 | `argon2`（唯一实现在 `common`） |
| ID | `uuid`（**v7**） |
| Cookie | `axum-extra`（feature `cookie`） |

**明确不装**：Kafka、etcd / Consul、K8s、Jaeger、Envoy / Nginx、Redis Cluster。

**硬规则：每个组件必须能答「为什么选它 / 不加它会出什么事」。**
答不出来就不是选型，是收藏。

### 12.1 观测：不上 Jaeger 之后靠什么

- `tracing` 结构化日志，字段固定 `trace_id` / `service` / `grpc.method` / `latency_ms`
- `trace_id` 从 gateway 生成（或从入站 header 取），**写进每个响应的信封**（§5）
- 跨服务传递：gateway → gRPC metadata `x-trace-id` → 下游 `tracing::Span` 里同名字段
- 排查手法：`grep <trace_id>` 一次串起四个进程的日志

代价：没有瀑布图，看不出「哪一跳慢了多少」—— 只能靠每跳自己打的 `latency_ms` 相加推断。
真需要瀑布图时再上 OTel，**但那时 `trace_id` 传导已经写好了**，改的只是 exporter。

### 12.2 开发期不写 Dockerfile

基础设施走 compose，四个服务在宿主机 `cargo run`：

- 改一行要重编：容器里 `docker build` 是分钟级，宿主机增量编译是秒级
- 四个进程四个终端能同时看日志（学分布式最需要这个），容器日志混在一起难跟
- 真要做时：**一个根 `Dockerfile` + `--bin` 选二进制**，多阶段构建。
  **别**写四份 `crates/<svc>/Dockerfile` —— 那是踩坑 80 的跨端拷贝

### 12.3 `.env` 键名

```
POSTGRES_URL_MAINTENANCE=postgres://runeshop:runeshop@localhost:5432/runeshop_ops
POSTGRES_URL_IDENTITY=postgres://runeshop:runeshop@localhost:5432/identity_db
POSTGRES_URL_CATALOG=postgres://runeshop:runeshop@localhost:5432/catalog_db
POSTGRES_URL_ORDER=postgres://runeshop:runeshop@localhost:5432/order_db
VALKEY_URL=valkey://localhost:6379
SESSION_TTL_SECONDS=86400
```

- **代码里 `env::var("X")` 的 X 必须和这里一字不差**（踩坑 7）
- **`.env` 是必须存在的文件**，缺了 `cargo run` 直接 `Error: NotPresent`（不是警告，是退出）
  —— 容器里则靠 compose 的 `environment:` 注入，`dotenvy::dotenv().ok()` 静默跳过
- `POSTGRES_URL_MAINTENANCE` 指向 **`runeshop_ops`**，只给 `ensure_database` 用；业务服务不读它
- `sea-orm-cli` 只认 `DATABASE_URL` → 所有 CLI 命令**必须显式传 `-u`**（踩坑 16）
- 引导超管（`BOOTSTRAP_ADMIN_*`）等 `operators` 那一刀再定，现在不读这个键
- 对象存储键名（`GARAGE_*`）随 §15 的 Garage 未决点一起定，**先不加**

---

## 13. 目录结构

```
runeshop/
├── Cargo.toml                 # [workspace] members = ["crates/*", "migration-*"]
├── docker-compose.yml         # 只装基础设施（postgres + valkey），不挂 initdb
├── proto/                     # 唯一的契约来源
│   ├── identity.proto
│   ├── catalog.proto
│   └── order.proto
├── crates/
│   ├── common/                # lib：信封、错误、trace_id、password、config
│   ├── gateway/               # :8080  唯一说 HTTP 的进程（BFF + 协议翻译）
│   ├── identity/              # :50051 (:8081 only /healthz)
│   ├── catalog/               # :50052 (:8082 only /healthz)
│   └── order/                 # :50053 (:8083 only /healthz)
├── migration-identity/
├── migration-catalog/
└── migration-order/
```

三套 migration crate 的原因：**`seaql_migrations` 每个库各一张，互不相通**。

### 13.1 每个服务内部

```
crates/<service>/
├── src/
│   ├── main.rs             # 起 gRPC server + /healthz
│   ├── model/              # sea-orm entity（只含自己库的表）
│   ├── store/              # SQL
│   ├── service/            # 业务规则
│   └── grpc/               # proto 生成的 trait 实现
└── build.rs                # tonic-build（读 ../../proto）
```

`client.rs` 只有 gateway 需要（它调别人）。依赖方向单向向下，**不可倒置**：

```
grpc/（协议层） → service/（业务规则） → store/（SQL） → model/（entity）
```

crate 级：`common ← identity / catalog / order / gateway`；`gateway → 全部`；
**identity / catalog / order 之间零依赖**，需要协作一律走 gRPC。

---

## 14. 错误分层（对外收敛，对内细分）

```
StoreError     只表达「取数据失败 / 没找到」，不懂业务
    ↓ From
ServiceError   把技术错误翻译成业务语义
    ↓ From
ApiError       只负责映射 HTTP status + 业务 code，impl IntoResponse
```

- `StoreError` 只有 `NotFound` / `Technical` 两个变体（业务概念不归它）
- `From` **只包装不拆解**；变体 → status + code 的映射只写在 `IntoResponse` 里（踩坑 49）
- `Display` 用 `[变体名]` 前缀（日志可 grep），payload 写完整句子（踩坑 48）
- **对外只暴露 §6 那 5 个码**：同一个 `ServiceError::Conflict` 可能映射 1004，也可能映射 1003，
  由**调用现场**决定，不由变体决定

面试口径：**问「为什么要这一层」= 问「合了会出什么事」，答不出合并的代价就是没答。**

---

## 15. 待办 / 未决

| # | 事项 | 状态 | 倾向 |
|---|---|---|---|
| 1 | **商户入驻审核**（`merchant_applications` + 补正状态机 + 部分唯一索引 + CHECK） | 未决 | 并进 identity（同库同事务，`merchants` 和申请表都由 identity 写）—— 但**没拍** |
| 2 | 对象存储 / 头像（Garage） | 本期不做 | §12 选型表里已无 Garage；旧第 3 项的头像等这一刀定型后再议 |
| 3 | 异步（Redis Streams：`XADD` / `XREADGROUP` / `XACK`，幂等 / 重试 / 死信） | 本期不做 | 旧第 6 项。要看清 ack 语义，不上 apalis |
| 4 | refresh token | 未讨论 | 现在只有 24h session，无刷新 |
| 5 | 全站限流 `429` | 留位 | §6 先占号不给语义 |
| 6 | trace 可视化（OTel + Jaeger） | 本期不做 | §12.1 已把 `trace_id` 传导写好，将来只换 exporter |

---

## 16. 进度（竖切路线）

**判据永远是「能在命令行看见」**，不是「代码写完了」。

- [x] **0. 骨架**（11-13 按 §0.1 的形态重做）
      虚拟 workspace（`common` / `identity` / `migration-identity`）+ postgres/valkey + `ensure_database` 能建库
      验收：`Checking` 数 = `workspace_members` 数 = 3；`cargo run -p migration-identity` 后 `\l` 出现 `identity_db`
- [ ] **1. 第一刀：`customers` 八步**（当前，§10）← **下一个动作：写 `m2026..._create_customers.rs` 并注册进 `migrations()`**
      进度：迁移链路 ✅（`ensure_database` + 空 `Migrator` + `seaql_migrations` 已建）；步 1 建表 ⬜
      验收：§10 第 6 步 `grpcurl` 四连 + 第 8 步 `curl` 四连全过
- [ ] **2. 把第一刀的同一套实现用到 `merchants` / `operators`**
      验收：三张表都能 register / login / me / logout，`role` 各自正确（踩坑 94）
- [ ] **3. 商户入驻审核**（§15 第 1 条定了再做）
- [ ] **4. 商品写**：建商品（catalog），商户身份从 metadata 取
- [ ] **5. 商品读**：列表 / 详情 —— 第一次真的**不能 JOIN 店铺名**
- [ ] **6. 库存 + 读聚合**：`GET /stock?skus=...` 批量 + gateway 三态降级（§11.2）
- [ ] **7. 下单**：扣库存 + 建订单 = **一次本地事务**（§11.1 的兑现点）
- [ ] **8. 超时取消**：延迟队列 / 定时扫描
- [ ] **9. 支付回调**：幂等 + 重放 + 对账

---

## 17. 前端（三个仓库）

三个都是独立仓库，包管理一律 **bun**。

| 目录 | 是什么 | 技术栈 |
|---|---|---|
| `runeshop/` | **后端：gateway + 三个内部服务** | Rust + axum（gateway）/ tonic（gRPC）+ PG 17 + Valkey 8 |
| `operator/` | 运营后台 | Vue 3.5 + Vite + Naive UI + Tailwind v4 |
| `merchant/` | 商家中心 | Vue 3.5 + Vite + Naive UI + Tailwind v4 + Pinia |
| `storefront/` | 用户端商店 | Nuxt 4（SSR）+ Tailwind v4 + Pinia + Lucide |

| 项目 | 端口 | 调性 |
|---|---|---|
| `operator/` | 5173 | 企业风（深色侧栏 + 浅色内容） |
| `merchant/` | 5174 | 企业风 |
| `storefront/` | 5175 | **赛博朋克** |

命名口径：**对客产品按场景命名，内部工具按角色命名**。`storefront` 是「店」，`merchant` / `operator` 是「人」。
改 `operator/` 的理由是它和 `operators` 表、session 里的 `role: "operator"` 三处对齐；
`storefront` 不改是因为它是唯一 SSR 公网站点，叫 `customer` 会丢掉这层信息。

**共同约定（定稿后已改）**

| 项 | 做法 | 为什么 |
|---|---|---|
| 请求路径 | 前端打 `/api/...`，dev proxy 剥前缀转 `localhost:8080` | 上线换域名只改配置 |
| 登录态 | **httpOnly Cookie，前端不存 token** | XSS 偷不走；SSR 也能读（踩坑 104） |
| 响应信封 | `code !== 0` 就 throw，按 `code` 分支（§6） | `code` 不再等于 HTTP status |
| `meta` | 恒存在，可能是 `{}` | 分页读 `meta.total`，别判 `meta ?` |
| 401 处理 | 跳登录，但**在登录页不跳** | 登录失败也是 401，跳转会刷掉错误提示 |
| SSR 取数据 | `$fetch(url, { headers: useRequestHeaders(['cookie']) })` | 服务端没有浏览器自动带 Cookie（踩坑 106） |

- ⚠️ 三个前端现在**连不上任何后端**（旧的删了、新的还没写）—— 这是预期状态，别去「修」
- ⚠️ 定稿后前端不再有「路由守卫读 token」这一步，登录态靠 `/auth/me` 探一次；具体实现等 §16 第 1 刀跑通再动

**storefront 视觉五条硬约束**

1. **不加圆角** —— 圆角是「柔和友好」的语言
2. **霓虹只占 10% 面积** —— 青/洋红只用在品牌、hover、主按钮、角标
3. **按钮靠亮度变化，不靠发光**
4. **网格只在登录/注册页**
5. **元信息用等宽字**（`font-mono` + `tracking-widest`）

品牌 mark 用 `Gem` 图标 —— 别自己画 SVG。

---

## 18. 踩坑记录（重要教训）

> ⚠️ 这些条目录于**单体时期**（`model/store/service/handler` 分层），但绝大多数教训与架构无关
> （sea-orm / axum / fred / 错误处理 / 归一化 / 前端），重写后依然成立。
> 编号是稳定标识，正文里到处按编号引用，**不要重排**。
> 少数只针对旧结构的条目（如 `store/` 分组规则、`TokenKind` 不透明 token）已在新架构里失效，遇到时以 §11「表归属」/ §14「错误分层」为准。

**本版（2026-11-12 定稿）需要重读的条目**

| 条目 | 现在的读法 |
|---|---|
| 21 / 79（模块没进编译） | 仍然成立，而且迁移拆成三个 crate 后**更容易踩**：注册错 crate 的表会建到别的库（§13） |
| 95 / 96 / 97 / 103（入驻申请表的状态机与 CHECK） | 表暂时不存在（见 §15 未决）。但三条教训照样成立：一个枚举不能服务两个生命周期、事件时刻不留 INSERT 默认值、CHECK 挡「改了一半」 |
| 118（六个 `Checking <crate>`） | crate 清单变了（`common` + 四个服务 + 三个 migration）。判据改成：`cargo check --workspace` 输出里的 `Checking` 数与 `cargo metadata --no-deps` 的 `workspace_members` **一样多** |
| 105 / 106 / 104（cookie / SSR） | 全部仍然成立，而且现在**更关键** —— 定稿把认证整个押在 httpOnly Cookie 上（§4、§17） |
| 109（Nuxt 默认 3000 撞端口） | 已经不撞了（后端挪到 8080），但结论照样成立：**任何 dev server 都要显式配端口 + `strictPort`**，否则会静默 +1（踩坑 115） |
| 112（Garage / MinIO 镜像下架） | 对象存储本期不做（§15），但通用教训留着：**「AI 推荐某个自建服务」这句话本身会过期，动手前 `docker manifest inspect` 确认镜像还在** |


### 环境 / 配置

6. **`dotenv().ok()` 必须调用**：`.env` 只是磁盘文件，不调用就不会加载进环境变量，`env::var` 读到空 → `expect` panic
7. **`.env` 键名与代码一致**：`env::var("VALKEY_URL")` 与 `.env` 里的键必须一字不差
8. **`cargo add` 代替手写依赖**：手写 `version = "latest"` / 乱用 `full` feature 必翻车
9. **Rust 模块两种写法等价**：`src/config.rs` 与 `src/config/mod.rs`，`mod config;` 都能找到 —— 所以「文件搬家成目录」是零成本重构（2026-11-09 那次全靠这条）
11. **`expect` 是 panic 不是打印**：只用在**启动阶段**（读配置、连库）；请求处理函数里绝不能用
12. **配置层禁用 `unwrap_or_default()`**：读不到会给空字符串，错误信息跟根因无关；配置缺失就该 `expect` 快速失败
13. **`Database::connect` 是 async fn**：忘 `.await` 拿到的是 Future 不是连接
16. **sea-orm-cli 的工作目录**：默认找 `./migration`，必须在项目根运行；`migrate up` 要 `-u` 传 URL（**CLI 不读 .env**）
17. **Cargo workspace**：`migration/` 是独立 crate，根 Cargo.toml 需 `[workspace] members = ["migration"]`
21. **⚠️ sea-orm-cli 生成只做一半**：
    - `generate entity` 会自动加 `mod`，但**不会**在 `src/main.rs` 加 `mod model;`，不补则不参与编译
    - `migrate generate` 会建文件，但**不会**往 `migration/src/lib.rs` 的 `migrations()` 注册，不注册则 `migrate up` 静默跳过
    - 兜底验证：`\dt` 查 PG + `select * from seaql_migrations`

> 编号有空洞（1-5、10、14-15、18-20、36、67-68 已删）—— 那些是 Docker / Git 的通用常识，和本项目无关。
> 编号保留不复用，因为正文里按编号引用踩坑。

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
    | `ServiceError` | `service/error.rs` | + `Business` / `InvalidParams` / `Conflict` / `Unauthorized` / `Forbidden` | 业务规则 |
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

### 缓存 / fred 实战

69. **⚠️ fred 的 `connect()` 绝对不能 `await`**：`connect()` 返回 `ConnectHandle = JoinHandle<Result<(), Error>>`，fred 源码注释明说它「resolve 于连接**关闭**时」—— await 它会永久挂起。症状是**程序静默卡死**：没有 panic、没有日志、退出码 124，极易误诊。正确用法是**两步**：
    ```rust
    client.connect();                     // 启动后台连接任务（同步调用）
    client.wait_for_connect().await?;      // 这个才是「等连上」
    ```
    初次尝试过 `connect().await.expect()` 和链式写法，全部踩坑。**看到 `JoinHandle` 就要警觉**：它包的是「任务的结果」，不是「任务做完了」。
    > **fred 10 新增了 `init()`**：它是 `connect()` + `wait_for_connect()` 的简写，
    > `let conn_task = client.init().await?;` —— 注意 **`.await` 的是 `init()`，返回的 `ConnectHandle` 仍然不能 await**（要留到 `quit()` 之后再 await，或者干脆不管它）。
    > ✅ `valkey://` scheme 从 fred 10.0 起直接支持（`Config::from_url` 认 `redis|valkey`），不用改写成 `redis://`。
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
81. **`store` 和 `service` 的分组维度不同**：`store/user/mod.rs` 只有领域一层（表/聚合），`service/customer/auth/mod.rs` 是「端 + 领域」两层。差别是**有意的** —— 顾客和商户共用 `users` 表，store 里没法按端拆 —— 但会带来「两个层里 `user` 的位置不一样」的认知成本。
    - ⚠️ **2026-11-08 / 11-09 修订**：store 不再是「一律平铺」，也不再混用文件与目录 —— **每个领域一个目录**（`store/user/mod.rs`、`store/product/{write,read}.rs`），见「store 按领域分组」一节。
    - 附带代价：`verify_password` 是纯 argon2 计算、不碰 DB，严格说**不属于任何一张表**，却被塞在 `store/user/mod.rs` 里。**将来该上提到 `service/common/`**
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
    - **正解**：公共 DTO 放该端的 `mod.rs`（`handler/customer/mod.rs`），子模块用绝对路径 `use crate::handler::customer::UserResponse;`（`super::` 只上跳一层，到不了端根）
    - **哨兵**：`grep -rn 'struct UserResponse' src/` 出现两次 = 重复定义

### 三端账号表与 token 分端（2026-10-26）

93. **⚠️ 「历史迁移不能改」这条规矩有适用边界**：删掉已应用的 `m20261015_120000_alter_users_id_to_uuid.rs`（四步改列类型），改成直接改 `m20220101_000001_create_table_users.rs` 的 `pk_auto("id")` → `pk_uuid("id")`，另加一条 `add_uuid_defaults` 迁移补默认值。
    - **为什么敢改**：本地开发库、库里无真实数据、没有其他环境依赖这套迁移。留着一串「建表 → 改主键 → 改默认值」的历史迁移反而更难读
    - **规矩修正**：**有真实数据 / 有其他环境后不可改**；本地空库可以重写。改动必须和「重建开发库」这件事一起做（`docker compose down -v` + 重新 `migrate up`）
    - **哨兵**：改历史迁移后 `select * from seaql_migrations` 里的记录会和新文件对不上 → 只能重建库，别试图手工改迁移表
94. **⚠️ token 必须带端类型，否则跨端串用**：token value 只存 `user_id` 时，**顾客的 token 能直接打到 `/merchant/me`** —— 因为三端（顾客 / 商户）共用 `users` 表，提取器查得到人就算通过。
    - **修法**：value 改成 `端:user_id`，提取器按端比对 kind，不符直接 401
    - **反面**：这是「token 语义 = 身份，不带端类型」这条老结论的反面 —— 当时的理由是「商户状态会变，缓存角色会失真」，但**端类型不是角色**：端类型是**不变的**（顾客永远是顾客），商户的**状态**才需要每次查库
    - **推论**：token 里放「不变的」事实（哪一端），不放「会变的」事实（当前状态、权限）
95. **⚠️ 两张表各带一个 status = 两个真相来源**：`merchants.status` 和 `merchant_applications.status` 曾经**共用同一个 `MerchantStatus` 枚举**（pending/active/frozen/rejected），描述同一件事的两个视角。
    - **症状**：申请单上永远不会出现 `active/frozen`（"申请单被冻结"没有意义）；商户上永远用不到 `pending/rejected`（商户行是审核通过才创建的）→ 一半是死值
    - **✅ 已解决（2026-10-31）**：拆成两个枚举 —— `application_status`（pending/approved/rejected）管审核结论，`merchant_status`（active/frozen）管营业资格
    - **泛化**：同一件事拆成两张表时，**状态列不能各留一份**；更进一步，**一个枚举不能服务两个生命周期不同步的状态机**——改一个会污染另一个（加个 `withdrawn` 就同时出现在商户上）
    - **判据**：问「这些值在这个上下文里会真的出现吗？」。出现不了的就是死值，说明枚举借错了
96. **⚠️ 申请表和商户实体断链**：顾客 `POST /merchant/apply` 只写 `merchant_applications`；商户登录查的却是 `merchants` 表要求 `status == Active`。于是**审核通过了商户还是登不上**（`merchants` 行根本不存在 → `NotFound「你不是商户」`）。
    - **根因**：`store::merchant::create_merchant` 写好了但**从没被调用过** —— 编译器不报这个错（pub 函数不算 dead code），只有 `cargo check` 的 warning 提示
    - **✅ 已解决（2026-10-31）**：`approve_application` 在同一个事务里做两件事 —— 改申请单状态 + `create_merchant`（status = active）。实测审核通过后商户**立刻能登录**
    - **自查手法**：写完一条链路后，把涉及的**每张表的写入路径画一遍**，确认没有哪一步只读不写
97. **⚠️ 「事件发生时刻」字段不能有 INSERT 默认值**：`merchant_applications.reviewed_at` 建表时给了 `default current_timestamp()` → **未审核的行也有值**，看数据像审核过。
    - **判据**：`create_time`（行什么时候建）可以有默认值；`reviewed_at`（某件事什么时候发生）**只能由那件事发生时写入**
    - **✅ 已解决（2026-10-31）**：`reviewed_by` / `reviewed_at` 在建表迁移里就写成**可空、无默认值**（原 `m20261025_050002` 迁移已删除，内容并入建表）
    - **同款坑**：`update_time default current_timestamp()` 也不会在 UPDATE 时刷新（踩坑 28）
98. **⚠️ 换表时复制粘贴「查错列」**：后台注册从 `users` 切到 `backoffice_users` 独立表时，邮箱查重一度查成了 username 列 —— 编译全绿、逻辑全错（永远查不到已注册的邮箱 → 409 失效）。
    - **哨兵**：每个 `find_by_xxx` 的实现里，`.filter(Column::Yyy.eq(...))` 的列名必须和函数名里的 xxx **逐字对应**。换表时这些函数**名字相似但表不同** → 最容易看错
    - **姊妹篇**：踩坑 33（复制粘贴字段名 bug）、踩坑 82（`cargo check` 全绿 ≠ 逻辑对）

### 审核链路与分页（2026-10-31）

99. **⚠️ 裸 `DbErr` 进不了 service **：`ServiceError` 有 `Database(DbErr)` 变体，但**没有 `From<DbErr>`** —— 因为以前 `DbErr` 总是先被 store 包成 `StoreError` 再上来。第一次在 service 里直接调 sea-orm（`db.begin()` / `txn.commit()`）就报 `? couldn't convert the error to ServiceError`。
    - **修法**：补 `impl From<DbErr> for ServiceError { Self::Database(value) }`
    - **教训**：错误类型有变体 ≠ 有转换。**新引入一条"绕过 store 的调用"时，先想它的错误怎么上来**
100. **⚠️ 事务体里误传 `db` 编译能过、回滚失效**（踩坑 41 的现场版）：`approve_application` 里所有 store 调用必须传 `&txn`。写成 `&db` 照样编译通过，但每个写操作都自成事务 —— 中途失败时前后不一致。
    - **自查**：事务块内搜 `db` 这个词，**只应出现在 `db.begin()` 那一行**
    - **配套**：`return Err(...)` 不需要手动 `txn.rollback()` —— `txn` 被 drop 时自动回滚；写 `rollback` 反而在 `?` 之后不可达
101. **⚠️ 分页的三个必踩点**：
    - **排序必须稳定**：只 `order_by_desc(create_time)` 时，同一毫秒插入的行顺序不定 → 翻页重复/漏数据。加 `order_by_desc(Column::Id)` 做 tiebreaker（sea-orm 的 `Paginator` 文档也明写「没有 ORDER BY 翻页结果不确定」）
    - **`fetch_page` 是 0 基**：HTTP 的 `page=1` 要 `fetch_page(page - 1)`，用 `saturating_sub(1)` 防 `page=0` 下溢 panic
    - **`page_size` 要有上限**：`page_size.max(1)` 防 `LIMIT 0`（store 只管不炸），`clamp(1, 100)` 是业务规则（service 管）
102. **⚠️ `num_items()` 的正确性依赖关系方向**：`find_also_related` + `paginate` 时，`num_items()` 生成的 `COUNT(*)` 带 JOIN。**N:1**（申请单 → 一个用户）不会膨胀，count 正确；换成 **1:N**（用户 → 多条申请）就会重复计数。
    - **判据**：看 `find_also_related`（also = 一个，N:1）还是 `find_with_related`（with = 一堆，1:N）。后者 + `num_items` = 错的
103. **CHECK 约束能挡住「改了一半」的更新**：`chk_merchant_applications_review_consistency` 要求「非 pending 必须有 `reviewed_by` + `reviewed_at`」。
    - 实测 `update ... set status='approved'` 但没写审核人 → PG 直接报 `violates check constraint`
    - **价值**：把一个三天后才发现的脏数据 bug，变成当场失败的请求
    - **代价**：写审核逻辑时必须三列一起写（`update_merchant_application_status` 已经这么做）

### 用户端 / Nuxt / cookie 认证（2026-11-02）

104. **⚠️ SSR 拿不到 `localStorage`，认证方式被逼着改**：Nuxt 服务端渲染时没有 `window`，读不到 `localStorage` 里的 token。硬要留 `localStorage` 的话，首屏永远是「未登录」，水合后才闪成登录态。
    - **唯一解**：token 放 **httpOnly cookie**，浏览器自动带，服务端 `useCookie` / `useRequestFetch` 都能读到
    - **顺带的收益**：`HttpOnly` 让 JS 读不到 → **XSS 偷不走 token**，比 `localStorage` 安全
    - **顺带的义务**：要配 `SameSite=Lax` 挡 CSRF；上线加 `Secure`
    - **教训**：技术选型（SSR）会反过来改架构（认证）——这种连锁要提前想，越晚改越痛

105. **⚠️ cookie 按域名共享，不看端口**：`localhost:5175` 写下的 cookie，浏览器会一起带到 `localhost:5173` / `localhost:5174` / `localhost:3000` 的请求上。
    - **症状**：用户端登录过之后，商户端接口突然 401——因为提取器「cookie 优先」，取到的是顾客 token，kind 不符
    - **修法**：提取器按端分开来源 —— `CustomerUser` 读 cookie + Bearer，`MerchantUser` / `BackofficeUser` **只读 Bearer**
    - **更彻底的修法**（将来上 HTTPS 时）：三个端用三个 cookie 名 + `SameSite=Strict`
    - **判据**：只要「一个域名下跑多个端」，就要问「这个 cookie 会被谁带走」

106. **⚠️ 服务端的 `fetch` 不会自动带浏览器 cookie**：浏览器里 `fetch('/api/x')` 会自动带，但 SSR 时是 **Node 进程**在请求，没有任何 cookie 可言。
    - **修法**：`useRequestFetch()` —— 它会把当前请求的 headers（含 cookie）转发给下游
    - **反例**：`$fetch('/api/x')` 在 SSR 下拿到的永远是未登录状态，而且**不报错**，只是数据不对（踩坑 82 的同类）
    - **哨兵**：SSR 页面「刷新后是未登录、点一下路由才变登录」= 几乎一定是这个

107. **`/api` 前缀要在代理层剥掉**：前端打 `/api/customer/me`，后端路由是 `/customer/me`。
    - Vite：`proxy: { '/api': { target, rewrite: p => p.replace(/^\/api/, '') } }`
    - Nitro：`routeRules: { '/api/**': { proxy: 'http://localhost:3000/**' } }` —— **目标里的 `/**` 就是干这个的**
    - 直接用 `devProxy` 不会剥前缀，会变成 `:3000/api/customer/me` → 404

108. **⚠️ `app.vue` 里没有 `<NuxtLayout>`，`layouts/` 完全不生效**：`layouts/default.vue` 写得再对也没用，因为没人去套它。
    - 正确：`<NuxtLayout><NuxtPage /></NuxtLayout>`
    - **症状**：顶栏/页脚不出现，页面「看起来很空」，但控制台零报错
    - **同类**：`pages/` 目录不存在时，所有 URL 都 404 —— Nuxt 的文件即路由是**目录驱动**的，不建目录就没有任何路由

109. **Nuxt 默认端口是 `3000`，和 Axum 撞**：不配 `devServer.port` 的话，`bun run dev` 会报端口占用或者抢掉后端的端口。
    - 约定：后端 `3000`、运营后台 `5173`、商户端 `5174`、用户端 `5175`

110. **`modules` 是数组，不是对象**：写 `modules: { '@pinia/nuxt' }` 会报 `Object literal may only specify known properties, and ''@pinia/nuxt'' does not exist in type ...` —— 这个报错读起来像是「模组没装」，其实是「括号写错了」。
    - 正确：`modules: ['@pinia/nuxt']`

111. **⚠️ 千万别用 `pkill -f 'nuxt dev'`**：`pkill -f` 匹配的是**整条命令行**，而你执行的那条 shell 命令本身就包含 `nuxt dev` 这几个字 → **把自己杀了**（表现：`Killed by SIGTERM`，命令没有任何输出）。
    - 用 `pkill -x <进程名>`，或者记 PID 杀 PID
    - 更稳的起后台服务方式：`nohup bun run dev > /tmp/storefront.log 2>&1 &`

### 对象存储与三个前端（2026-11-03 ~ 11-06）

112. **⚠️ MinIO 的 Docker 镜像已经下架**：`minio/minio` 在 Docker Hub 返回 **404**，`bitnami/minio` 仓库还在但 **0 个 tag**，`quay.io/minio/minio` 要登录（401）。
    - **改用 Garage**（`dxflrs/garage:v2.5.0`）：S3 兼容、单节点、镜像只有 20MB
    - **代价**：没有网页控制台，建桶/发密钥全走命令行：
      ```bash
      garage layout assign -z dc1 -c 1G <node-id>
      garage layout apply --version 1
      garage key create runeshop-app
      garage bucket create avatars
      garage bucket allow --read --write --owner avatars --key runeshop-app
      garage bucket website --allow avatars
      ```
    - **website 模式是子域名分桶**：配了 `root_domain = ".localhost"` 之后，访问路径是
      `http://avatars.localhost:3902/<key>`，**不是** `http://localhost:3902/avatars/<key>`
    - **教训**：「AI 推荐某个流行的自建服务」这句建议本身会过期。动手前先 `docker manifest inspect <image>` 确认镜像还在

113. **⚠️ 全角字符会让编译器报一个完全不相干的错**：`}；`（全角分号）写进 Rust 文件，`cargo check` 的报错指向的是周围语法，不会告诉你「这是全角字符」。这次是一个全角分号被提交并推到了远端，本地修好才发现 master 上那份编译不过。
    - 中文输入法状态下写代码，标点很容易被自动转成全角
    - 高危字符：`;` `；`、`:` `：`、`(` `（`、`"` `"` `"`
    - **自查**：`grep -rnP '[\x{FF1B}\x{FF1A}\x{FF08}\x{FF09}\x{FF0C}]' src/`（全角分号/冒号/括号/逗号）
    - **哨兵**：报错位置看着完全正常 → 用 `cat -A` 或 `hexdump` 看那几个字节

114. **⚠️ `vue-tsc` 在组件 slot 内对 `reactive` + `v-model` 会误报 `.value`**：报错形如
    `Property 'value' does not exist on type '{ username: string; password: string; }'`，指向 `v-model="form.username"`。
    - **触发条件**（探针实测）：`reactive` 对象 + `v-model` + **写在组件 `<slot>` 里**。裸在页面里写不报，`ref` 写在 slot 里也不报
    - **这是工具缺陷，不是运行时问题**（Vue 运行时完全正常）
    - **修法**：表单变量改用 `ref`（模板自动解包，`v-model="form.username"` 一行不用改），脚本里补 `.value`
    - 试过给组件加 `defineSlots` —— **不管用**
    - **判据**：先 `bun run type-check` 在命令行复现；能复现才是真问题，复现不了是编辑器缓存（踩坑 F1）

115. **⚠️ Vite 端口被占时会静默 +1，不报错**：`server.port = 5174` 只是"首选"，被占用就自动 5175、5176……于是你会遇到「服务起来了，但页面是另一个项目」。
    - **症状**：curl 某个端口拿到的是别的项目的 `<title>`
    - **诊断**：`ss -ltnp | grep -E ':(517[0-9])'` + `curl -s localhost:PORT | grep -o '<title>[^<]*</title>'`
    - **根治**：想要「占用了就报错」就设 `server.strictPort = true`
    - 别同时留多个 dev server；`Ctrl+C` 之外的启动方式（`nohup ... &`）最容易忘

116. **`vue-tsc` 3.x 不兼容 TypeScript 7**：装出来是 `typescript@7.x` 时报
    `ERR_PACKAGE_PATH_NOT_EXPORTED: Package subpath './lib/tsc'`，因为 TS 7 移除了这个路径。
    - 锁到 `typescript@5.9`；`operator` / `merchant` 现在用的是 6.0.3，**升 7 会撞同一堵墙**
    - 另外 `tsconfig` 里的 `baseUrl` 在 TS 6+ 已废弃，用 `paths` 相对 tsconfig 解析即可


### 前端 / Naive UI（2026-10 起）

- **F1. `tsgo` 扩展会让 Vue 的类型解析失败**：Zed 装了 TypeScript 原生版（`tsgo`）后，它占了 `typescript-ls` 槽位，`vue-language-server` 的 tsserver 转发被跳过（日志：`forwarding skipped: language server not found`），表现为 **`Cannot find module './App.vue'`**，而 `vue-tsc --build` 却是绿的。卸掉 tsgo 即恢复
    - **判据**：编辑器红线 + `bun run type-check` 干净 = 编辑器语言服务问题，别改代码
    - **前提**：`.vue` 不需要 `shims-vue.d.ts`（那是 Volar 之前的老办法，加了反而把所有组件类型退化）
- **F2. 组件名拼错 → 整块白屏，且 `vue-tsc` 不报**：`<n-layout-side>`（正确是 `n-layout-sider`）时，`unplugin-vue-components` 生成了 `import { NLayoutSide } from 'naive-ui'` → 浏览器报 `does not provide an export named 'NLayoutSide'` → **整个 .vue 模块加载失败**
    - `components.d.ts` 顶部有 `// @ts-nocheck`，所以类型检查抓不到
    - **对策**：关键组件**显式 import**（有补全、能跳转、拼错立刻标红），别全靠自动导入
    - **另一个坑**：`ttrigger="click"` 少写一个 t 不报错（透传成 DOM 属性），`n-dropdown` 回退到默认 hover 触发
- **F3. 布局父路由的 path 不要带名字**：`path: '/AdminLayout'` 会让 URL 变成 `/AdminLayout/applications`，且 `redirect: '/applications'` 匹配不到任何路由（child 的真实路径是父路径拼出来的）。父级用 `path: '/'` + `children`
- **F4. `n-data-table` 后端分页必须加 `remote`**：否则组件会自己再切一遍已分页的数据（"每页只有 1 条"）。`pagination.itemCount` 填后端返回的 `total`
- **F5. Naive UI 没有 CSS 文件**：它是 CSS-in-JS，**不要**写 `import 'naive-ui/dist/index.css'`（Element/Ant 的习惯）
- **F6. 抽屉/菜单在非组件上下文用不了 `useMessage()`**：http.ts 这类模块要弹提示得用 `createDiscreteApi(['message'])`
- **F7. dev server 出现"所有项目文件 404、只有 `/@vite/client` 200"** = 进程坏了（root 解析错），**重启即可**，不是代码问题
- **F8. `n-select` / `n-input` 的宽度用 Tailwind 改不动**：组件默认 `width: 100%`（或由主题决定），`class="w-36"` 会被组件自己的样式盖掉 → 用 `style="width: 140px"`
- **F9. Naive 的 `n-layout-content` 背景同理**：`class="bg-[#f6f7f9]"` 无效，要用 `content-style="background: #f6f7f9; padding: 24px"`；`n-layout-sider` 的整列底色也要加在 **sider 根元素**上（加在内层 `div.h-full` 上只覆盖内容高度——父层是 `auto` 高度，`h-full` 退化）

### 重写期：workspace 与本地环境（2026-11-11 晚，第 1 项）

117. **⚠️ PGDATA 卷被复用 → `docker-entrypoint-initdb.d/` 静默跳过**
    新写的 `docker-compose.yml` 里服务叫 `postgres`、卷叫 `pgdata`，算出来的卷名还是 `runeshop_pgdata` —— 跟旧项目**同名**，于是 docker 直接复用那个旧卷。
    entrypoint 看到 PGDATA 非空 → **整个 init 阶段跳过** → 三个库一个都没建，但容器状态是 `Up (healthy)`。
    - **症状**：`docker compose ps` 全绿，`\l` 里只有 `runeshop`
    - **根治**：换环境前先 `docker rm -f <容器>` + `docker volume rm <卷>`（容器挂在卷上时卷删不掉，顺序不能反）；或者直接 `docker compose down -v`
    - **诊断**：`docker compose logs postgres | grep -i initdb` —— 初始化阶段的输出全在里面，跑没跑一眼就看出来
    - **判据**：「容器起来了」和「初始化跑了」是**两件事**，必须分别验证（同踩坑 82）

118. **⚠️ 从 `cargo new` 到 workspace 的三个"编译过但不对"**
    一次性列出来，这三个都不会给出指向根因的报错：

    | 现象 | 根因 | 判据 |
    |---|---|---|
    | `failed to load manifest for workspace member .../crate/*` | `members = ["crate/*"]` 少个 `s`。**glob 匹配不到时 cargo 当字面路径去读** | 路径里出现 `*` 就是拼错了 |
    | `no targets specified in the manifest` | 根 `Cargo.toml` 里 `[package]` 没删。有 `[package]` 就等于声明"根目录也是个 crate"，cargo 去找 `src/main.rs` | 虚拟 manifest 只有 `[workspace]`，没有 `[package]` |
    | `version.workspace was set to true but workspace.package.version was not defined` | 成员用了 `.workspace = true`，但根里只有 `[package]`、没有 `[workspace.package]` | `[package]` → `[workspace.package]` 是**一张表的改名**，必须一次做完 |

    - **另外两件同源的事**：
      ① 六个 crate 全建成 `--lib` 时 `cargo check` 全绿、零 warning，但 `cargo run -p identity` 报 `a bin target must be available` —— lib 合法但**起不了服务**（踩坑 82 的现场版）
      ② `cargo new` 偶尔在 crate 里**自己 `git init`**（`crates/common/.git`），于是父仓库把它当 submodule，`git add` 报「没有检出一个提交」→ `rm -rf` 掉，再 `find . -name .git -not -path './.git'` 确认只剩仓库根一个
    - **验收判据**：不是看到 `Finished`，而是看到 **`Checking` 数量 == `workspace_members` 数量**（当前 3）；`cargo metadata --no-deps` 里成员数对得上

### 重写期：建库与迁移链路（2026-11-13）

119. **⚠️ sea-orm 2.0 的 `query_one` / `execute` 收 `&S`，且 `S` 必须是 `StatementBuilder`**
    原生 SQL（`Statement::from_sql_and_values`）**不是** `StatementBuilder`，所以 `query_one(&stmt)` 也不对。
    两个错会连着报：
    - 不写 `&`：`expected &{unknown}, found Statement`（泛型 `S` 推不出来，编译器**不会**提示缺引用）
    - 补上 `&`：`Statement: StatementBuilder is not satisfied`

    **正解**：原生 SQL 用 `query_one_raw` / `query_all_raw` / `execute_raw`（**按值**收 `Statement`）；
    只有 sea-query 构建器（`Query::select()` 那条链）才用不带 `_raw` 的、传 `&stmt`。
    坑源：1.x 教程写的是 `query_one(&self, stmt: Statement)`，照抄必踩（同踩坑 89：老结论要按版本重验）。

120. **⚠️ `sea-orm-cli migrate init` 默认 `-d ./migration`**
    在 `migration-identity/` 里跑它 → 生成 `migration-identity/migration/`，多套一层。
    最阴的地方：内层 crate 不被 `members = ["crates/*", "migration-*"]` 匹配 →
    **不是 workspace 成员 → 完全不参与编译**，`cargo check` 一声不响（踩坑 79 的变体）。
    - 判据：`cargo metadata --no-deps | jq '.workspace_members | length'` + `Checking` 数
    - 修法：要么 `-d <目标目录>`，要么只用 `cargo new` + 手写，**别两条命令都跑**

121. **⚠️ `migrate init` 生成的 features 模板全被注释 → 没有驱动**
    ```toml
    features = [ # "runtime-tokio-rustls",  # "sqlx-postgres" ]
    ```
    这是个空数组。`cargo check` 全绿（编译期不需要驱动），`cargo run` 一连库就挂。
    必须显式写 `["runtime-tokio-rustls", "sqlx-postgres"]` —— 踩坑 82 的又一个现场。

122. **⚠️ 缺 `.env` 不是警告，是退出**
    `env::var` 返回 `VarError::NotPresent`，`?` 让 main 打印 `Error: NotPresent` 并退出。
    `dotenvy::dotenv().ok()` 只是「有就读，没有就跳过」，**不会创建文件**。
    - 判据：看退出码和最后一行输出，别看到 `Running target/debug/...` 就以为跑成了
    - 容器里没有 `.env` 是正常的，靠 compose 的 `environment:` 注入

123. **`docker compose exec postgres psql` 默认拿操作系统用户当 DB 角色**
    容器里是 root → `FATAL: role "root" does not exist`。
    两条修法：命令加 `-U runeshop`，或 compose 里设 `PGUSER: runeshop`。
    同理 `PGDATABASE`：改名 `POSTGRES_DB` 时必须一起改，否则裸 `psql` 会去找那个不存在的同名库。

124. **`POSTGRES_DB` 缺省 = `POSTGRES_USER`**
    entrypoint 里是 `file_env 'POSTGRES_DB' "$POSTGRES_USER"` —— 不写这行也会建一个**与用户名同名**的库，删不掉。
    所以只能改名字不能消灭它：叫 `runeshop_ops` 是为了让人一眼看出「这不是业务库」。
    （`postgres` 是集群内置维护库，`ensure_database` 连它也行；用 `runeshop_ops` 是为了让裸 `psql` 有个落点。）

125. **Zed 的 dockerfile 扩展会烧光 GitHub 未认证配额**
    `docker-language-server` 每次 LSP 启动都调一次 `latest_github_release`（**不缓存、失败就重试**），
    60 次/小时很快见底，然后满屏 `403 API rate limit exceeded`。它同时负责 compose 的补全和 schema 校验，
    所以「没有补全」和「那条 403」是同一件事。
    - **根治**：扩展第一步是 `worktree.which("docker-language-server")` —— 二进制放 PATH 就跳过全部下载逻辑
    - 网络不通时的旁路：`api.github.com` / `objects.githubusercontent.com` 通、`github.com:443` 不通的情况，
      直接用 `github.com/.../releases/download/...` 的地址下资产即可（不需要走 API）
    - 替代方案：compose 文件的权威校验是 `docker compose config`，补全只查 schema，`config` 还会查卷和变量

---

## 19. 代码纪律（写代码前先看）

> 目标：半年后回来还能一眼看懂。**宁可多写一行直白代码，也不写一行需要猜的「聪明」代码。**

**命名**
- 名字回答「是什么 / 干什么」，不回答「怎么实现」；函数动词开头，布尔 `is_` / `has_` / `can_` 开头
- **能脱离目录独立读懂**：`apply` / `update` 看不出对象 → `apply_merchant` / `update_order_status`
- 集合复数（`users`）、单个单数（`user`）；宁可长，不要 `data` / `info` / `temp`；不造缩写（`db` / `id` / `url` 可以，`usr` / `val` 不行）

**卫兵写法**：参数不对 / 找不到 / 没权限 → 立刻 `return Err`，主干逻辑留在最外层缩进，嵌套最多两层。
用 `let Some(x) = ... else { return ... }`，别写「金字塔」。

**函数只做一件事**
- 判据：一句话说不清「它做什么」（需要「并且 / 然后」）→ 拆
- **超过 40 行先想拆，超过 60 行基本必须拆**
- 写库 / 写缓存 / 发消息**不要**和参数校验混在一起
- 层与层不越界：HTTP / Cookie 概念留 gateway，业务规则留 service，SQL 留 store，protobuf 只在 grpc/

**错误处理**
- **请求路径禁止 `unwrap()` / `expect()` / `panic!()`**（启动阶段可以用）
- 优先 `?`；补上下文用 `map_err`，别丢原始错误；不写 `let _ = result;` 吞错误
- 变体带 payload（`NotFound(String)`）；`From` 只包装不拆解
- 错误消息要能定位问题，但登录失败这类必须模糊（防枚举）
- **对内细分、对外收敛**：日志里想多细都行，响应里只给 §6 那 5 个码（§14）

**直白优先**
- 链式超过两步拆成有名字的中间变量；能用 `for` 就别硬凹 `fold`
- 魔法数字提常量；注释解释「为什么」，不解释「做了什么」

**收尾自查**
- `cargo check` 无新增 error / warning（warning 当 TODO，别用 `#[allow]` 藏）
- `grep -rn "unwrap()\|expect(" src/`，请求路径上的逐个确认
- 每个 `if` 代入「正常值 / 异常值」各验一遍（防布尔写反）
- 有副作用的代码（写库 / 写缓存 / 发消息 / Set-Cookie）是否在所有失败分支**之后**？（踩坑 78）
- 新增迁移是否注册到**对应的那个** `migration-*/src/lib.rs` 的 `migrations()`？（踩坑 21）
- 新增的 `mod` 是否都声明了？（踩坑 79 的空 `mod.rs` 会让整个子目录静默蒸发）
- proto 改字段是否只改名没改号？废弃字段号是否 `reserved`？

---

## 20. 约定

**通用**
- 镜像名 / 服务名 / 容器名一律小写
- 目录 / 模块 / 文件 / 函数一律 `snake_case`；类型 `UpperCamelCase`
- **目录名单数**：`user/` 不是 `users/`（`register/` 不是 `registers/`）
- **依赖一律用 `cargo add`**，不手写 Cargo.toml 版本
- 能用 sea-query DSL 表达的，**不手写原生 SQL**（原生 SQL 换库即废、拼错编译期不报错）
- 代码保持格式化后再保存（Zed 已开 `format_on_save`）

**边界（定稿后的四条硬线）**
- **一张表只被一个服务写**。被两个服务写 = 设计还没做完，回 §11 重新定归属
- **一个事务只在一个 database 内**。想跨库写 = 先问「这张表是不是放错库了」
- **只有 gateway 说 HTTP / JSON**。内部服务只挂 `/healthz`，业务全走 gRPC
- **禁止用 GET 做状态变更**（§4.3）

**迁移**
- 历史迁移**不能改**，只能追加 —— **例外**：本地开发库（无真实数据、无其他环境）可以重写，
  但必须同步 `docker compose down -v` 重建开发库（踩坑 93）
- **重建开发库的新办法**（11-13 起）：不必删卷。`DROP DATABASE identity_db` 后重跑
  `cargo run -p migration-identity` 即可 —— 建库和建表都在同一个二进制里（§1.2）
- `migration-identity` / `-catalog` / `-order` **各管各库**，条目录错 crate = 表建到别的库（`seaql_migrations` 不跨库）

**契约**
- **API 出参一律用 DTO，绝不把 entity 直接当 JSON 契约**（`password_hash` 会泄露；字段一改就破接口）
- proto 用 `Request` / `Response` 成对；**字段号才是身份**，改名安全、改号静默错位（§8）
- 生成类型与现有类型撞名，用 `as` 别名，**不要**为了避让去改 proto 名字
- 列表接口一律**后端分页**（`LIMIT/OFFSET` + 稳定排序 + tiebreaker，踩坑 101），
  分页信息进 `meta: {total, page, page_size}`，前端不做内存切片
- 写操作返回写后的那一行（`Model`），保持 `create_*` / `update_*` 一致

**数据表示**
- 金额一律 `Decimal` 或**整数分**，绝不用 `f64`（踩坑 37）
- 时间存 `timestamptz`，出参 **RFC3339 UTC**（踩坑 27）
- ID 一律 **UUID v7**
- 枚举 → API 字符串的映射放 `crates/common/src/status.rs`，不挂在某个服务里（跨服务共用）

**凭据 / 配置**
- 本地开发凭据（`runeshop`/`runeshop`）仅限本地，上线前必须换成环境变量注入
- `.env` 不入 git（`.gitignore` 已加 `*.env`）
- `SESSION_TTL_SECONDS` 与 Cookie 的 `Max-Age` **改一处必须改另一处**（§4.2）
