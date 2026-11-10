use sea_orm::ActiveValue::Set;
use sea_orm::PaginatorTrait;
use sea_orm::sqlx::types::Uuid;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder,
};

use crate::model::merchant::merchant_applications::Entity as MerchantApplications;
use crate::model::merchant::merchant_applications::{ActiveModel, Column, Model};
use crate::model::merchant::sea_orm_active_enums::ApplicationStatus;
use crate::store::error::StoreError;

/// 创建申请单的入参；密码已经是哈希值
pub struct NewMerchantApplication {
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub shop_name: String,
}

pub async fn create_merchant_application<C: ConnectionTrait>(
    db: &C,
    data: NewMerchantApplication,
) -> Result<Model, StoreError> {
    Ok(ActiveModel {
        username: Set(data.username),
        password_hash: Set(data.password_hash),
        email: Set(data.email),
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

/// 分页查申请单；返回 `(rows, total)`
///
/// 不再 JOIN customers —— 申请人现在就是申请单自己的一条记录（用户名 + 邮箱），
/// 审核人名字由 service 层批量补（跨域没有外键，不能 JOIN）
pub async fn list_merchant_applications<C: ConnectionTrait>(
    db: &C,
    filter: ApplicationFilter,
    page: u64,
    page_size: u64,
) -> Result<(Vec<Model>, u64), StoreError> {
    let mut query = MerchantApplications::find()
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

/// 按用户名查「最近一条」申请（不限状态）
///
/// 登录时要靠它区分：待审 / 待补正 / 已驳回
pub async fn find_latest_application_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<Model>, StoreError> {
    Ok(MerchantApplications::find()
        .filter(Column::Username.eq(username))
        .order_by_desc(Column::CreateTime)
        .order_by_desc(Column::Id)
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

/// 写审核结论（通过 / 驳回 / 要求补正都走它）
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

/// 申请人补正资料（用户名不可改）
pub struct UpdateApplicationContent {
    pub shop_name: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
}

/// 改资料并把状态推回 pending
///
/// `reviewed_*` 故意不清空 —— 那是「被打回过一次」的痕迹
pub async fn update_application_content<C: ConnectionTrait>(
    db: &C,
    id: Uuid,
    data: UpdateApplicationContent,
) -> Result<Model, StoreError> {
    let mut app: ActiveModel = MerchantApplications::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| StoreError::NotFound("商户申请不存在".into()))?
        .into_active_model();

    if let Some(shop_name) = data.shop_name {
        app.shop_name = Set(shop_name);
    }
    if let Some(email) = data.email {
        app.email = Set(email);
    }
    if let Some(password_hash) = data.password_hash {
        app.password_hash = Set(password_hash);
    }
    app.status = Set(ApplicationStatus::Pending);
    app.update_time = Set(chrono::Utc::now().into());

    app.update(db).await.map_err(StoreError::from)
}
