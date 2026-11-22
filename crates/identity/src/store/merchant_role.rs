use sea_orm::{ConnectionTrait, EntityTrait};
use uuid::Uuid;

use crate::model::merchant_roles::Model;
use crate::model::prelude::MerchantRoles;
use crate::store::error::StoreError;

pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<Option<Model>, StoreError> {
    Ok(MerchantRoles::find_by_id(id).one(db).await?)
}
