use sea_orm::ActiveValue::Set;
use sea_orm::sqlx::types::Uuid;
use sea_orm::{PaginatorTrait, QueryOrder};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter,
};

use crate::model::backoffice_users::Model as BackofficeUserModel;
use crate::model::merchant_applications::{ActiveModel, Column, Model};
use crate::model::prelude::{BackofficeUsers, MerchantApplications, Users};
use crate::model::sea_orm_active_enums::ApplicationStatus;
use crate::model::users::Model as UserModel;
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
        status: Set(ApplicationStatus::Pending),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub struct ApplicationFilter {
    pub status: Option<ApplicationStatus>, // None = 全部
    pub keyword: Option<String>,           // 店铺名模糊搜（可选）
}
pub async fn list_merchant_applications<C: ConnectionTrait>(
    db: &C,
    filter: ApplicationFilter,
    page: u64,
    page_size: u64,
) -> Result<
    (
        Vec<(Model, Option<UserModel>, Option<BackofficeUserModel>)>,
        u64,
    ),
    StoreError,
> {
    let mut query = MerchantApplications::find()
        .find_also_related(Users)
        .find_also_related(BackofficeUsers)
        .order_by_desc(Column::CreateTime)
        .order_by_desc(Column::Id);

    if let Some(status) = filter.status {
        query = query.filter(Column::Status.eq(status));
    }

    // 空字符串当"没传"处理，否则 LIKE '%%' 等于全表
    if let Some(keyword) = filter.keyword.as_deref().map(str::trim) {
        if !keyword.is_empty() {
            query = query.filter(Column::ShopName.like(format!("%{keyword}%")));
        }
    }

    let paginator = query.paginate(db, page_size.max(1));
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page.saturating_sub(1)).await?;

    Ok((rows, total))
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

pub async fn update_merchant_application_status<C: ConnectionTrait>(
    db: &C,
    id: Uuid,
    status: ApplicationStatus,
    reviewed_by: Uuid,
    review_remark: Option<String>,
) -> Result<Model, StoreError> {
    let mut app: ActiveModel = MerchantApplications::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| StoreError::NotFound("商户申请不存在".into()))?
        .into_active_model();

    app.status = Set(status);
    app.reviewed_by = Set(Some(reviewed_by));
    app.reviewed_at = Set(Some(chrono::Utc::now().into()));
    app.update_time = Set(chrono::Utc::now().into());
    app.review_remark = Set(review_remark);

    app.update(db).await.map_err(StoreError::from)
}
