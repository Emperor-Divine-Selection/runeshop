use regex::Regex;
use sea_orm::ConnectionTrait;
use std::sync::LazyLock;

use crate::{
    cache::token::{generate, save},
    model::users::Model,
    store::user::{NewUser, create_user, find_by_email, find_by_username, verify_password},
};

use crate::service::errors::ServiceError;

/// 用户登陆
pub async fn login<C: ConnectionTrait>(
    db: &C,
    cache: &fred::clients::Client,
    username: &str,
    password: &str,
) -> Result<String, ServiceError> {
    let username = username.trim().to_lowercase();

    let Some(user) = find_by_username(db, &username).await? else {
        return Err(ServiceError::Business("用户名或密码错误".into()));
    };
    if !verify_password(&user.password_hash, password).await {
        return Err(ServiceError::Business("用户名或密码错误".into()));
    };

    let token = generate();
    save(cache, &token, user.id, None).await?;
    Ok(token)
}

/// 注册一个用户；
/// 入参不合法返 `InvalidParams`(422)，用户名或邮箱已被占用返 `Conflict`(409)
pub async fn register<C: ConnectionTrait>(
    db: &C,
    mut data: NewUser,
) -> Result<Model, ServiceError> {
    // 1. 归一化：去掉首尾空白，后续校验/查重/入库都用这个值
    data.username = data.username.trim().to_string();
    data.email = data.email.trim().to_string();

    // 2. 校验参数（不碰数据库，比查重便宜）
    validate(&data)?;

    // 3. 查重（要查两次库，放在校验之后）
    if find_by_username(db, &data.username).await?.is_some() {
        return Err(ServiceError::Conflict(format!(
            "用户名{}已被占用",
            data.username
        )));
    }
    if find_by_email(db, &data.email).await?.is_some() {
        return Err(ServiceError::Conflict(format!(
            "邮箱{}已被注册",
            data.email
        )));
    }

    // 4. 入库
    Ok(create_user(db, data).await?)
}

/// 密码最短长度
const MIN_PASSWORD_LEN: usize = 8;

/// 校验注册入参；任一不合法返 `InvalidParams`
fn validate(data: &NewUser) -> Result<(), ServiceError> {
    if data.username.is_empty() {
        return Err(ServiceError::InvalidParams("用户名不能为空".into()));
    }
    if !is_valid_email(&data.email) {
        return Err(ServiceError::InvalidParams("邮箱格式不正确".into()));
    }
    // chars().count() 是字符数不是字节数：4 个汉字 = 4 而非 8
    if data.password.chars().count() < MIN_PASSWORD_LEN {
        return Err(ServiceError::InvalidParams(format!(
            "密码至少 {} 位",
            MIN_PASSWORD_LEN
        )));
    }
    Ok(())
}

/// 邮箱格式校验：顶级域名至少 2 个字母。
/// 只校验格式，**不验证邮箱是否真实存在**——将来要真验证得加 `email_verified` 字段 + 发验证码。
fn is_valid_email(email: &str) -> bool {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[a-zA-Z]{2,}$").unwrap());
    RE.is_match(email)
}
