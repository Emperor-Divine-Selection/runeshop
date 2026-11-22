use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::model::merchants::{Column, Model};
use crate::model::prelude::Merchants;
use crate::store::error::StoreError;

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
