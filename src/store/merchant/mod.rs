use sea_orm::ActiveValue::Set;
use sea_orm::sqlx::types::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::model::merchant::merchants::Entity as Merchants;
use crate::model::merchant::merchants::{ActiveModel, Column, Model};
use crate::model::merchant::sea_orm_active_enums::MerchantStatus;
use crate::store::error::StoreError;

/// 创建商户的入参；密码已经是哈希值（申请单里存的就是哈希）
pub struct NewMerchant {
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub shop_name: String,
}

/// 创建商户（status = active）；应在申请审核通过时调用，不查重（查重是 service 层的业务规则）
pub async fn create_merchant<C: ConnectionTrait>(
    db: &C,
    data: NewMerchant,
) -> Result<Model, StoreError> {
    Ok(ActiveModel {
        username: Set(data.username),
        password_hash: Set(data.password_hash),
        email: Set(data.email),
        shop_name: Set(data.shop_name),
        status: Set(MerchantStatus::Active),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

/// 按 id 查商户；查不到返回 Ok(None)
pub async fn find_merchant_by_id<C: ConnectionTrait>(
    db: &C,
    id: Uuid,
) -> Result<Option<Model>, StoreError> {
    Ok(Merchants::find_by_id(id).one(db).await?)
}

/// 按登录名查商户；查不到返回 Ok(None)
pub async fn find_merchant_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Merchants::find()
        .filter(Column::Username.eq(username))
        .one(db)
        .await?)
}

/// 按邮箱查商户；查不到返回 Ok(None)
pub async fn find_merchant_by_email<C: ConnectionTrait>(
    db: &C,
    email: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(Merchants::find()
        .filter(Column::Email.eq(email))
        .one(db)
        .await?)
}
