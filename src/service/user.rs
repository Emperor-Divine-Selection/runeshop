use sea_orm::ConnectionTrait;

use crate::{
    model::users::Model,
    store::{
        errors::StoreError,
        user::{NewUser, create_user, find_by_email, find_by_username},
    },
};

/// 注册一个用户；用户名或邮箱已被占用时返回 Conflict
pub async fn register<C: ConnectionTrait>(db: &C, data: NewUser) -> Result<Model, StoreError> {
    if find_by_username(db, &data.username).await?.is_some() {
        return Err(StoreError::Conflict(format!(
            "用户名{}已被占用",
            data.username
        )));
    }
    if find_by_email(db, &data.email).await?.is_some() {
        return Err(StoreError::Conflict(format!("邮箱{}已被注册", data.email)));
    }
    create_user(db, data).await
}
