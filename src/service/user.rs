use sea_orm::ConnectionTrait;

use crate::{
    model::users::Model,
    store::user::{NewUser, create_user, find_by_email, find_by_username},
};

use super::errors::ServiceError;

/// 注册一个用户；用户名或邮箱已被占用时返回 Conflict
pub async fn register<C: ConnectionTrait>(db: &C, data: NewUser) -> Result<Model, ServiceError> {
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
    Ok(create_user(db, data).await?)
}
