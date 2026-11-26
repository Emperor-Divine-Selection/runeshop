use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::model::merchant_applications::{ActiveModel, Column, Model};
use crate::model::prelude::MerchantApplications;
use crate::model::sea_orm_active_enums::ApplicationStatus;
use crate::store::error::StoreError;

#[derive(Debug)]
pub struct NewApplication {
    pub id: Uuid,
    pub shop_name: String,
    pub username: String,
    pub email: String,
}

pub async fn create<C: ConnectionTrait>(
    db: &C,
    data: NewApplication,
) -> Result<Model, StoreError> {
    Ok(ActiveModel {
        id: Set(data.id),
        shop_name: Set(data.shop_name),
        username: Set(data.username),
        email: Set(data.email),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

/// 审核用：锁住这一行，防止两个人同时审同一张单。
pub async fn find_by_id_for_update<C: ConnectionTrait>(
    db: &C,
    id: Uuid,
) -> Result<Option<Model>, StoreError> {
    Ok(MerchantApplications::find_by_id(id)
        .lock_exclusive()
        .one(db)
        .await?)
}

pub async fn find_pending_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(MerchantApplications::find()
        .filter(Column::Username.eq(username))
        .filter(Column::Status.eq(ApplicationStatus::Pending))
        .one(db)
        .await?)
}

pub async fn find_pending_by_email<C: ConnectionTrait>(
    db: &C,
    email: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(MerchantApplications::find()
        .filter(Column::Email.eq(email))
        .filter(Column::Status.eq(ApplicationStatus::Pending))
        .one(db)
        .await?)
}

pub async fn find_pending_by_shop_name<C: ConnectionTrait>(
    db: &C,
    shop_name: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(MerchantApplications::find()
        .filter(Column::ShopName.eq(shop_name))
        .filter(Column::Status.eq(ApplicationStatus::Pending))
        .one(db)
        .await?)
}

/// 分页 + 可选状态筛选。排序带 `id` 做 tiebreaker（踩坑 101）。
pub async fn list_page<C: ConnectionTrait>(
    db: &C,
    status: Option<ApplicationStatus>,
    page: u64,
    page_size: u64,
) -> Result<(Vec<Model>, u64), StoreError> {
    let mut query = MerchantApplications::find();

    if let Some(status) = status {
        query = query.filter(Column::Status.eq(status));
    }

    let paginator = query
        .order_by_desc(Column::CreatedAt)
        .order_by_desc(Column::Id)
        .paginate(db, page_size);

    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page.saturating_sub(1)).await?;

    Ok((items, total))
}

/// 写入审核结论：状态 + 审核人 + 时间 + 备注。
pub async fn apply_review<C: ConnectionTrait>(
    db: &C,
    application: Model,
    status: ApplicationStatus,
    reviewer_id: Uuid,
    note: Option<String>,
) -> Result<Model, StoreError> {
    let now = chrono::Utc::now().fixed_offset();

    let mut active: ActiveModel = application.into();
    active.status = Set(status);
    active.reviewed_by = Set(Some(reviewer_id));
    active.reviewed_at = Set(Some(now));
    active.review_note = Set(note);
    active.updated_at = Set(now);

    Ok(active.update(db).await?)
}
