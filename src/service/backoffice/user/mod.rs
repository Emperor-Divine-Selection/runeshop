use sea_orm::{ConnectionTrait, sqlx::types::Uuid};

use crate::{
    cache::token::{delete, generate, save, TokenKind},
    model::backoffice_users::Model,
    service::common::validation::validate,
    store::backoffice_user::{
        NewBackofficeUser, create_backoffice_user, find_backoffice_user_by_email,
        find_backoffice_user_by_id, find_backoffice_user_by_username, verify_password,
    },
};

use crate::service::error::ServiceError;

pub async fn authenticate_backoffice<C: ConnectionTrait>(
    db: &C,
    user_id: Uuid,
) -> Result<Model, ServiceError> {
    find_backoffice_user_by_id(db, user_id)
        .await?
        .ok_or_else(|| ServiceError::Forbidden("不是后台账号".into()))
}

pub async fn backoffice_register<C: ConnectionTrait>(
    db: &C,
    mut data: NewBackofficeUser,
) -> Result<Model, ServiceError> {
    data.username = data.username.trim().to_lowercase();
    data.email = data.email.trim().to_lowercase();
    data.password = data.password.trim().to_string();

    validate(&data.username, &data.email, &data.password)?;

    if find_backoffice_user_by_username(db, &data.username)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict(format!(
            "用户名{}已被占用",
            data.username
        )));
    }
    if find_backoffice_user_by_email(db, &data.email)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict(format!(
            "邮箱{}已被注册",
            data.email
        )));
    }

    Ok(create_backoffice_user(db, data).await?)
}

// login 和 logout 保持不变
pub async fn backoffice_login<C: ConnectionTrait>(
    db: &C,
    cache: &fred::clients::Client,
    username: &str,
    password: &str,
) -> Result<String, ServiceError> {
    let username = username.trim().to_lowercase();

    let Some(user) = find_backoffice_user_by_username(db, &username).await? else {
        return Err(ServiceError::Unauthorized("用户名或密码错误".into()));
    };
    if !verify_password(&user.password_hash, password.trim()).await {
        return Err(ServiceError::Unauthorized("用户名或密码错误".into()));
    };

    let token = generate();
    save(cache, &token, TokenKind::Backoffice, user.id, None).await?;
    Ok(token)
}

pub async fn backoffice_logout(
    cache: &fred::clients::Client,
    token: &str,
) -> Result<(), ServiceError> {
    Ok(delete(cache, token).await?)
}
