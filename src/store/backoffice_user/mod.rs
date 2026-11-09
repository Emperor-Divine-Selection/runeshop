use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use sea_orm::ActiveValue::Set;
use sea_orm::DbErr;
use sea_orm::sqlx::types::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::model::backoffice_users::{ActiveModel, Column, Model};
use crate::model::prelude::BackofficeUsers;
use crate::store::error::StoreError;

pub struct NewBackofficeUser {
    pub username: String,
    pub password: String,
    pub email: String,
}

pub async fn create_backoffice_user<C: ConnectionTrait>(
    db: &C,
    data: NewBackofficeUser,
) -> Result<Model, StoreError> {
    let hash = Argon2::default()
        .hash_password(data.password.as_bytes())
        .map_err(|e| StoreError::Technical(DbErr::Custom(e.to_string())))?
        .to_string();

    Ok(ActiveModel {
        username: Set(data.username),
        password_hash: Set(hash),
        email: Set(data.email),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn find_backoffice_user_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(BackofficeUsers::find()
        .filter(Column::Username.eq(username))
        .one(db)
        .await?)
}

pub async fn find_backoffice_user_by_id<C: ConnectionTrait>(
    db: &C,
    id: Uuid,
) -> Result<Option<Model>, StoreError> {
    Ok(BackofficeUsers::find()
        .filter(Column::Id.eq(id))
        .one(db)
        .await?)
}

pub async fn find_backoffice_user_by_email<C: ConnectionTrait>(
    db: &C,
    email: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(BackofficeUsers::find()
        .filter(Column::Email.eq(email))
        .one(db)
        .await?)
}

pub async fn verify_password(stored_hash: &str, plain: &str) -> bool {
    Argon2::default()
        .verify_password(plain.as_bytes(), stored_hash)
        .is_ok()
}
