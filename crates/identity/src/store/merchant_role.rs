use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::model::merchant_roles::{Column, Model};
use crate::model::prelude::MerchantRoles;
use crate::store::error::StoreError;

pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<Option<Model>, StoreError> {
    Ok(MerchantRoles::find_by_id(id).one(db).await?)
}

pub async fn find_by_permission<C: ConnectionTrait>(
    db: &C,
    permission: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(MerchantRoles::find()
        .filter(Column::Permission.eq(permission))
        .one(db)
        .await?)
}
