use sea_orm::ActiveValue::Set;
use sea_orm::sqlx::types::Uuid;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter,
};

use crate::model::merchant_applications::{ActiveModel, Column, Model};
use crate::model::prelude::MerchantApplications;
use crate::model::sea_orm_active_enums::MerchantStatus;
use crate::store::error::StoreError;

pub struct NewMerchantApplication {
    pub user_id: Uuid,
    pub shop_name: String,
}

pub async fn create_merchant_application<C: ConnectionTrait>(
    db: &C,
    data: NewMerchantApplication,
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

pub async fn find_merchant_application_by_user_id<C: ConnectionTrait>(
    db: &C,
    user_id: Uuid,
) -> Result<Option<Model>, StoreError> {
    Ok(MerchantApplications::find()
        .filter(Column::UserId.eq(user_id))
        .one(db)
        .await?)
}

pub async fn find_merchant_application_by_shop_name<C: ConnectionTrait>(
    db: &C,
    shop_name: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(MerchantApplications::find()
        .filter(Column::ShopName.eq(shop_name))
        .one(db)
        .await?)
}

pub async fn find_merchant_application_by_id<C: ConnectionTrait>(
    db: &C,
    id: Uuid,
) -> Result<Option<Model>, StoreError> {
    Ok(MerchantApplications::find_by_id(id).one(db).await?)
}

pub async fn find_pending_merchant_applications<C: ConnectionTrait>(
    db: &C,
) -> Result<Vec<Model>, StoreError> {
    Ok(MerchantApplications::find()
        .filter(Column::Status.eq(MerchantStatus::Pending))
        .all(db)
        .await?)
}

pub async fn update_merchant_application_status<C: ConnectionTrait>(
    db: &C,
    id: Uuid,
    status: MerchantStatus,
    reviewed_by: Uuid,
) -> Result<Model, StoreError> {
    let mut app: ActiveModel = MerchantApplications::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| StoreError::NotFound("商户申请不存在".into()))?
        .into_active_model();

    app.status = Set(status);
    app.reviewed_by = Set(Some(reviewed_by));
    app.reviewed_at = Set(Some(chrono::Utc::now().into()));

    app.update(db).await.map_err(StoreError::from)
}
