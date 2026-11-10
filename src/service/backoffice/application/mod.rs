use std::collections::{HashMap, HashSet};

use sea_orm::sqlx::types::Uuid;
use sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait};

use crate::model::merchant::merchant_applications::Model;
use crate::model::merchant::sea_orm_active_enums::ApplicationStatus;
use crate::service::error::ServiceError;
use crate::store::backoffice_user::find_backoffice_users_by_ids;
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

/// 分页查申请单；返回 `(rows, total)`
///
/// tuples 的第二个值是**审核人名字**：`reviewed_by` 现在跨 schema 没有外键，
/// 不能 JOIN，只能「先查列表 → 收集 id → 批量查运营 → 拼名字」
pub async fn list_applications<C: ConnectionTrait>(
    db: &C,
    data: ListApplicationsQuery,
) -> Result<(Vec<(Model, Option<String>)>, u64), ServiceError> {
    let page = data.page.max(1);
    let page_size = data.page_size.clamp(1, MAX_PAGE_SIZE);
    let filter = ApplicationFilter {
        status: data.status,
        keyword: data.keyword,
    };

    let (rows, total) = list_merchant_applications(db, filter, page, page_size).await?;

    let reviewer_ids: Vec<Uuid> = rows
        .iter()
        .filter_map(|row| row.reviewed_by)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let operators = find_backoffice_users_by_ids(db, reviewer_ids).await?;
    let reviewer_names: HashMap<Uuid, String> =
        operators.into_iter().map(|o| (o.id, o.username)).collect();

    let items = rows
        .into_iter()
        .map(|row| {
            let name = row
                .reviewed_by
                .and_then(|id| reviewer_names.get(&id).cloned());
            (row, name)
        })
        .collect();

    Ok((items, total))
}

/// 审核通过：改申请单状态 + 用申请单里的凭证建商户（同一个事务）
pub async fn approve_application(
    db: &DatabaseConnection,
    application_id: Uuid,
    reviewer_id: Uuid,
    review_remark: Option<String>,
) -> Result<Model, ServiceError> {
    let review_remark = normalize_remark(review_remark);
    let txn = db.begin().await?; // ① 开事务

    let application = load_pending(&txn, application_id).await?; // ② 用 txn 查

    let updated = update_merchant_application_status(
        &txn, // ③ 用 txn 写
        application_id,
        ApplicationStatus::Approved,
        reviewer_id,
        review_remark,
    )
    .await?;

    // ④ 商户的凭证就是申请单里的那一份，原样搬过去
    create_merchant(
        &txn,
        NewMerchant {
            username: application.username,
            password_hash: application.password_hash,
            email: application.email,
            shop_name: application.shop_name,
        },
    )
    .await?;

    txn.commit().await?; // ⑤ 提交
    Ok(updated)
}

/// 驳回：理由必填
pub async fn reject_application(
    db: &DatabaseConnection,
    application_id: Uuid,
    reviewer_id: Uuid,
    review_remark: Option<String>,
) -> Result<Model, ServiceError> {
    let Some(review_remark) = normalize_remark(review_remark) else {
        return Err(ServiceError::InvalidParams("驳回必须填写理由".into()));
    };

    let txn = db.begin().await?;
    load_pending(&txn, application_id).await?;

    let updated = update_merchant_application_status(
        &txn,
        application_id,
        ApplicationStatus::Rejected,
        reviewer_id,
        Some(review_remark),
    )
    .await?;

    txn.commit().await?;
    Ok(updated)
}

/// 要求补正：理由必填，申请人登录后能看到并改资料
pub async fn request_supplement(
    db: &DatabaseConnection,
    application_id: Uuid,
    reviewer_id: Uuid,
    review_remark: Option<String>,
) -> Result<Model, ServiceError> {
    let Some(review_remark) = normalize_remark(review_remark) else {
        return Err(ServiceError::InvalidParams("要求补正必须填写理由".into()));
    };

    let txn = db.begin().await?;
    load_pending(&txn, application_id).await?;

    let updated = update_merchant_application_status(
        &txn,
        application_id,
        ApplicationStatus::SupplementRequired,
        reviewer_id,
        Some(review_remark),
    )
    .await?;

    txn.commit().await?;
    Ok(updated)
}

/// 取申请单并确认它还在待审 —— 三个审核动作共用的卫兵
async fn load_pending<C: ConnectionTrait>(
    db: &C,
    application_id: Uuid,
) -> Result<Model, ServiceError> {
    let Some(application) = find_merchant_application_by_id(db, application_id).await? else {
        return Err(ServiceError::NotFound("申请单不存在".into()));
    };
    if application.status != ApplicationStatus::Pending {
        return Err(ServiceError::Conflict("该申请正在等申请人补正，不能直接审核".into()));
    }
    Ok(application)
}

/// 审核意见归一化：空串 / 纯空格 → None，其余 trim 后保留
fn normalize_remark(remark: Option<String>) -> Option<String> {
    remark
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
