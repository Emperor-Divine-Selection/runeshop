use sea_orm::ActiveValue::Set;
use sea_orm::sea_query::OnConflictIdentifier::Column;
use sea_orm::sqlx::types::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::model::customers::{self, ActiveModel, Model};
use crate::model::prelude::Customers;

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

pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: &Uuid,
) -> Result<Option<Model>, StoreError> {
    Ok(Customers::find_by_id(*id).one(db).await?)
}

pub async fn find_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Customers::find()
        .filter(customers::Column::Username.eq(username))
        .one(db)
        .await?)
}
