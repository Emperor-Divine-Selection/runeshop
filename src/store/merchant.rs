use sea_orm::ActiveValue::Set;
use sea_orm::sqlx::types::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::model::merchants::{ActiveModel, Column, Model};
use crate::model::prelude::Merchants;
use crate::model::sea_orm_active_enums::MerchantStatus;
use crate::store::error::StoreError;

pub struct NewMerchant {
    pub user_id: Uuid,
    pub shop_name: String,
}

/// 为指定用户创建商户申请（status = pending）；不查重（查重是 service 层的业务规则）
pub async fn create_merchant<C: ConnectionTrait>(
    db: &C,
    data: NewMerchant,
) -> Result<Model, StoreError> {
    Ok(ActiveModel {
        user_id: Set(data.user_id),
        shop_name: Set(data.shop_name),
        status: Set(MerchantStatus::Pending),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

/// 按 user_id 查商户；查不到返回 Ok(None)
pub async fn find_merchant_by_user_id<C: ConnectionTrait>(
    db: &C,
    user_id: Uuid,
) -> Result<Option<Model>, StoreError> {
    Ok(Merchants::find()
        .filter(Column::UserId.eq(user_id))
        .one(db)
        .await?)
}
