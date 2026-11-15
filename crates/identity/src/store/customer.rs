use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::model::customers::{ActiveModel, Column, Entity, Model};
use crate::store::error::StoreError;

#[derive(Debug)]
pub struct NewCustomer {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: String,
}

pub async fn create<C: ConnectionTrait>(db: &C, data: NewCustomer) -> Result<Model, StoreError> {
    Ok(ActiveModel {
        id: Set(data.id),
        username: Set(data.username),
        password_hash: Set(data.password_hash),
        email: Set(data.email),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<Option<Model>, StoreError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Entity::find()
        .filter(Column::Username.eq(username))
        .one(db)
        .await?)
}

pub async fn find_by_email<C: ConnectionTrait>(
    db: &C,
    email: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Entity::find()
        .filter(Column::Email.eq(email))
        .one(db)
        .await?)
}
