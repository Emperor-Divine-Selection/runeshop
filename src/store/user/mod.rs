use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::model::prelude::Users;
use crate::model::users::{Column, Model};
use crate::store::errors::StoreError;

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
