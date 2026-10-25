use sea_orm::{ConnectionTrait, sqlx::types::Uuid};

use crate::{
    model::merchant_applications::Model,
    service::error::ServiceError,
    store::merchant::find_merchant_by_user_id,
    store::merchant_application::{
        NewMerchantApplication, create_merchant_application,
        find_merchant_application_by_shop_name, find_merchant_application_by_user_id,
    },
};

const MAX_SHOP_NAME_LEN: usize = 50;

/// 提交商户申请；已申请过 / 已是商户 / 店铺名重复返 `Conflict`
pub async fn apply_merchant<C: ConnectionTrait>(
    db: &C,
    user_id: Uuid,
    shop_name: &str,
) -> Result<Model, ServiceError> {
    let shop_name = shop_name.trim();

    if shop_name.is_empty() {
        return Err(ServiceError::InvalidParams("店铺名称不能为空".into()));
    }
    if shop_name.chars().count() > MAX_SHOP_NAME_LEN {
        return Err(ServiceError::InvalidParams(format!(
            "店铺名称不能超过 {MAX_SHOP_NAME_LEN} 个字"
        )));
    }

    if find_merchant_by_user_id(db, user_id).await?.is_some() {
        return Err(ServiceError::Conflict("你已经是商户".into()));
    }
    if find_merchant_application_by_user_id(db, user_id)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict("你已提交过商户申请".into()));
    }
    if find_merchant_application_by_shop_name(db, shop_name)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict(format!(
            "店铺名称{shop_name}已被占用"
        )));
    }

    Ok(create_merchant_application(
        db,
        NewMerchantApplication {
            user_id,
            shop_name: shop_name.to_string(),
        },
    )
    .await?)
}
