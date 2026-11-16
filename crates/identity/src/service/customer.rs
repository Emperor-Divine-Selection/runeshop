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

/// 用 token 换当前用户资料。§7 的 `GET /auth/me`。
///
/// 三种失败（没有 session / 端类型不对 / 人已被删）对外**完全一样**：
/// 都是 1002 跳登录页。区别只进日志。
pub async fn me<C: ConnectionTrait>(
    db: &C,
    session: &SessionStore,
    token: &str,
) -> Result<Model, ServiceError> {
    let not_logged_in = || ServiceError::Unauthorized("未登录或登录已失效".to_owned());

    let Some(ident) = session.load(token).await? else {
        return Err(not_logged_in());
    };

    // 第一刀只有顾客。商户/运营的 token 即使格式合法，也不能拿来看顾客资料（踩坑 94）。
    if ident.role != Role::Customer {
        tracing::warn!(reason = "wrong_role", "token 的端类型与 customers 不符");
        return Err(not_logged_in());
    }

    let Some(customer) = find_by_id(db, ident.user_id).await? else {
        // session 还在但人没了 —— 数据不一致，不是用户输错
        tracing::warn!(
            reason = "session_user_missing",
            "session 指向的顾客已不存在"
        );
        return Err(not_logged_in());
    };

    Ok(customer)
}

pub async fn logout(session: &SessionStore, token: &str) -> Result<(), ServiceError> {
    session.delete(token).await?;
    Ok(())
}
