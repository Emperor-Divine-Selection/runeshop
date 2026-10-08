use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use sea_orm::ActiveValue::Set;
use sea_orm::DbErr;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::model::prelude::Users;
use crate::model::users::{ActiveModel, Column, Model};
use crate::store::errors::StoreError;

/// 创建用户的入参；`password` 是明文，由 store 内部哈希成 `password_hash`
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
}

/// 创建用户；不查重（查重是 service 层的业务规则）
pub async fn create_user<C: ConnectionTrait>(db: &C, data: NewUser) -> Result<Model, StoreError> {
    let hash = Argon2::default()
        .hash_password(data.password.as_bytes())
        .map_err(|e| StoreError::Technical(DbErr::Custom(e.to_string())))?
        .to_string();

    Ok(ActiveModel {
        username: Set(data.username),
        password_hash: Set(hash),
        email: Set(data.email),
        avatar: Set(data.avatar),
        bio: Set(data.bio),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

/// 按用户名查用户，查不到返回 Ok(None)
pub async fn find_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Users::find()
        .filter(Column::Username.eq(username))
        .one(db)
        .await?)
}

/// 按邮箱查用户，查不到返回 Ok(None)
pub async fn find_by_email<C: ConnectionTrait>(
    db: &C,
    email: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Users::find()
        .filter(Column::Email.eq(email))
        .one(db)
        .await?)
}

/// 验证密码
pub async fn verify_password(stored_hash: &str, plain: &str) -> bool {
    Argon2::default()
        .verify_password(plain.as_bytes(), stored_hash)
        .is_ok()
}
