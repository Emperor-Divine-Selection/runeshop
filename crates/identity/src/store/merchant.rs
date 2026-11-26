use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::model::merchants::{ActiveModel, Column, Model};
use crate::model::prelude::Merchants;
use crate::store::error::StoreError;

#[derive(Debug)]
pub struct NewMerchant {
    pub id: Uuid,
    pub shop_id: Uuid,
    pub role_id: Uuid,
    pub username: String,
    /// None = 未激活（等邀请链接设密码）
    pub password_hash: Option<String>,
    pub email: String,
}

pub async fn create<C: ConnectionTrait>(db: &C, data: NewMerchant) -> Result<Model, StoreError> {
    Ok(ActiveModel {
        id: Set(data.id),
        shop_id: Set(data.shop_id),
        role_id: Set(data.role_id),
        username: Set(data.username),
        password_hash: Set(data.password_hash),
        email: Set(data.email),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<Option<Model>, StoreError> {
    Ok(Merchants::find_by_id(id).one(db).await?)
}

/// `merchants.username` 是全局唯一，所以 `.one()` 是安全的。
pub async fn find_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Merchants::find()
        .filter(Column::Username.eq(username))
        .one(db)
        .await?)
}

pub async fn find_by_email<C: ConnectionTrait>(
    db: &C,
    email: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Merchants::find()
        .filter(Column::Email.eq(email))
        .one(db)
        .await?)
}
