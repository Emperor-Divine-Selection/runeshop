use sea_orm::ConnectionTrait;
use sea_orm::sqlx::types::Uuid;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::model::products::{self, Entity};
use crate::store::error::StoreError;

pub async fn find_product_by_merchant_and_name<C: ConnectionTrait>(
    db: &C,
    merchant_id: &Uuid,
    name: &str,
) -> Result<Option<products::Model>, StoreError> {
    Ok(Entity::find()
        .filter(products::Column::MerchantId.eq(*merchant_id))
        .filter(products::Column::Name.eq(name))
        .one(db)
        .await?)
}
