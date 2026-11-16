use common::password::{hash_password, verify_password};
use common::role::Role;
use sea_orm::ConnectionTrait;
use uuid::Uuid;

use crate::model::customers::Model;
use crate::service::credential::{session::SessionStore, token::generate_token};
use crate::service::error::ServiceError;
use crate::service::normalize::{normalize_email, normalize_username};
use crate::service::validate::{validate_email, validate_password, validate_username};
use crate::store::customer::{NewCustomer, create, find_by_email, find_by_id, find_by_username};

#[derive(Debug)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// 注册。**不自动登录** —— 不建 session，调用方拿到的只有新行（§7）。
pub async fn register<C: ConnectionTrait>(
    db: &C,
    input: RegisterInput,
) -> Result<Model, ServiceError> {
    // 1. 先归一化：后面的校验和查重都必须基于归一化后的值
    let username = normalize_username(&input.username);
    let email = normalize_email(&input.email);

    // 2. 再校验：参数不合法就不该产生数据库往返
    validate_username(&username)?;
    validate_email(&email)?;
    validate_password(&input.password)?;

    // 3. 查重（便宜）。用归一化后的值查，否则 "ALICE" 查不到库里的 "alice"
    if find_by_username(db, &username).await?.is_some() {
        return Err(ServiceError::Conflict("用户名已被占用".to_owned()));
    }

    if find_by_email(db, &email).await?.is_some() {
        return Err(ServiceError::Conflict("邮箱已被注册".to_owned()));
    }

    // 4. 最后才哈希（贵，一次约 50ms）。放前面等于让人用重复请求白嫖 CPU
    let password_hash = hash_password(&input.password)?;

    // 5. 建号。id 由应用生成（UUID v7），数据库没有默认值
    let new_customer = NewCustomer {
        id: Uuid::now_v7(),
        username,
        email,
        password_hash,
    };

    Ok(create(db, new_customer).await?)
}

#[derive(Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

/// 登录。成功返回 `(token, 那一行)`。
///
/// **不返回 DTO** —— proto 的 `LoginResponse` 是步 5 的事，service 只产数据。
pub async fn login<C: ConnectionTrait>(
    db: &C,
    session: &SessionStore,
    input: LoginInput,
) -> Result<(String, Model), ServiceError> {
    let username = normalize_username(&input.username);

    // 用户名不存在与密码错必须返回【同一条】错误（踩坑 51），
    // 否则等于送攻击者一个「这个用户名存在吗」的查询接口。
    let Some(customer) = find_by_username(db, &username).await? else {
        return Err(ServiceError::InvalidCredentials(
            "用户名或密码错误".to_owned(),
        ));
    };

    if !verify_password(&customer.password_hash, &input.password)? {
        return Err(ServiceError::InvalidCredentials(
            "用户名或密码错误".to_owned(),
        ));
    }

    // 顺序纪律（踩坑 78）：先算 token、判完所有条件，**最后**才写外部系统。
    // 反过来写就是「登录失败但 Valkey 里留了一个 24h 的孤儿 token」。
    let token = generate_token()?;
    session.save(&token, Role::Customer, &customer.id).await?;

    Ok((token, customer))
}

/// 按 id 取顾客。给 `GetCustomer` RPC 和 `me` 共用。
/// 取当前用户资料。§7 的 `GET /auth/me` 走的就是它 —— gateway 已经用
/// `ValidateSession` 拿到 user_id 了，所以这里只认 id，不再解析 token。
pub async fn me<C: ConnectionTrait>(db: &C, user_id: Uuid) -> Result<Model, ServiceError> {
    find_by_id(db, user_id).await?.ok_or_else(|| {
        tracing::warn!(reason = "customer_missing", "用户不存在");
        ServiceError::Unauthorized("用户不存在".to_owned())
    })
}

pub async fn logout(session: &SessionStore, token: &str) -> Result<(), ServiceError> {
    session.delete(token).await?;
    Ok(())
}
