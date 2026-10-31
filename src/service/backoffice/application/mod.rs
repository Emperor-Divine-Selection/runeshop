use sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait, sqlx::types::Uuid};

use crate::model::merchant_applications::Model;
use crate::model::sea_orm_active_enums::ApplicationStatus;
use crate::model::users;
use crate::service::error::ServiceError;
use crate::store::merchant::{NewMerchant, create_merchant};
use crate::store::merchant_application::{
    ApplicationFilter, find_merchant_application_by_id, list_merchant_applications,
    update_merchant_application_status,
};

const MAX_PAGE_SIZE: u64 = 100;

pub struct ListApplicationsQuery {
    pub status: Option<ApplicationStatus>,
    pub keyword: Option<String>,
    pub page: u64,
    pub page_size: u64,
}
pub async fn list_applications<C: ConnectionTrait>(
    db: &C,
    data: ListApplicationsQuery,
) -> Result<(Vec<(Model, Option<users::Model>)>, u64), ServiceError> {
    let page = data.page.max(1);
    let page_size = data.page_size.clamp(1, MAX_PAGE_SIZE);
    let status = data.status;
    let keyword = data.keyword;
    let filter = ApplicationFilter { status, keyword };

    let list = list_merchant_applications(db, filter, page, page_size).await?;
    Ok(list)
}

pub async fn approve_application(
    db: &DatabaseConnection,
    application_id: Uuid,
    reviewer_id: Uuid,
) -> Result<Model, ServiceError> {
    let txn = db.begin().await?; // ① 开事务

    let application = find_merchant_application_by_id(&txn, application_id) // ② 用 txn 查
        .await?
        .ok_or_else(|| ServiceError::NotFound("申请单不存在".into()))?;

    if application.status != ApplicationStatus::Pending {
        // ③ 卫兵
        return Err(ServiceError::Conflict("该申请已审核过".into()));
    }

    let updated = update_merchant_application_status(
        &txn, // ④ 用 txn 写
        application_id,
        ApplicationStatus::Approved,
        reviewer_id,
    )
    .await?;

    create_merchant(
        &txn, // ⑤ 用 txn 建商户
        NewMerchant {
            user_id: application.user_id,
            shop_name: application.shop_name,
        },
    )
    .await?;

    txn.commit().await?; // ⑥ 提交
    Ok(updated)
}

pub async fn reject_application(
    db: &DatabaseConnection,
    application_id: Uuid,
    reviewer_id: Uuid,
) -> Result<Model, ServiceError> {
    let txn = db.begin().await?; // ① 开事务

    let application = find_merchant_application_by_id(&txn, application_id) // ② 用 txn 查
        .await?
        .ok_or_else(|| ServiceError::NotFound("申请单不存在".into()))?;

    if application.status != ApplicationStatus::Pending {
        // ③ 卫兵
        return Err(ServiceError::Conflict("该申请已审核过".into()));
    }

    let updated = update_merchant_application_status(
        &txn, // ④ 用 txn 写
        application_id,
        ApplicationStatus::Rejected,
        reviewer_id,
    )
    .await?;

    txn.commit().await?; // ⑥ 提交
    Ok(updated)
}
