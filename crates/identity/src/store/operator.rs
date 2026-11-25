use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::model::operators::{ActiveModel, Column, Model};
use crate::model::prelude::Operators;
use crate::store::error::StoreError;

#[derive(Debug)]
pub struct NewOperator {
    pub id: Uuid,
    pub role_id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: String,
}

pub async fn create<C: ConnectionTrait>(db: &C, data: NewOperator) -> Result<Model, StoreError> {
    Ok(ActiveModel {
        id: Set(data.id),
        role_id: Set(data.role_id),
        username: Set(data.username),
        password_hash: Set(Some(data.password_hash)),
        email: Set(data.email),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<Option<Model>, StoreError> {
    Ok(Operators::find_by_id(id).one(db).await?)
}

pub async fn find_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Operators::find()
        .filter(Column::Username.eq(username))
        .one(db)
        .await?)
}

/// 引导超管用：只要有一行就说明已经初始化过。
pub async fn find_any<C: ConnectionTrait>(db: &C) -> Result<Option<Model>, StoreError> {
    Ok(Operators::find().one(db).await?)
}
