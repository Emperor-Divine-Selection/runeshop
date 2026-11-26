use sea_orm::ActiveValue::Set;
use sea_orm::sea_query::{Expr, ExprTrait, Func};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::model::prelude::Shops;
use crate::model::shops::{ActiveModel, Column, Model};
use crate::store::error::StoreError;

#[derive(Debug)]
pub struct NewShop {
    pub id: Uuid,
    pub name: String,
}

pub async fn create<C: ConnectionTrait>(db: &C, data: NewShop) -> Result<Model, StoreError> {
    Ok(ActiveModel {
        id: Set(data.id),
        name: Set(data.name),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<Option<Model>, StoreError> {
    Ok(Shops::find_by_id(id).one(db).await?)
}

/// 店名唯一索引是 `lower(name)`，所以查重也必须大小写不敏感。
pub async fn find_by_name<C: ConnectionTrait>(
    db: &C,
    name: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Shops::find()
        .filter(Func::lower(Expr::col(Column::Name)).eq(name.to_lowercase()))
        .one(db)
        .await?)
}
