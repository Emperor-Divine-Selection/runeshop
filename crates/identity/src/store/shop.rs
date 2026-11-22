use sea_orm::{ConnectionTrait, EntityTrait};
use uuid::Uuid;

use crate::model::prelude::Shops;
use crate::model::shops::Model;
use crate::store::error::StoreError;

pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<Option<Model>, StoreError> {
    Ok(Shops::find_by_id(id).one(db).await?)
}
