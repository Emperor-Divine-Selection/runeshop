use sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait};
use uuid::Uuid;

use crate::model::merchant_applications::Model;
use crate::model::sea_orm_active_enums::ApplicationStatus;
use crate::service::error::ServiceError;
use crate::service::normalize::{normalize_email, normalize_username};
use crate::service::validate::{validate_email, validate_shop_name, validate_username};
use crate::store::merchant_application::{self, NewApplication};
use crate::store::{merchant, merchant_role, shop};

#[derive(Debug)]
pub struct SubmitInput {
    pub shop_name: String,
    pub username: String,
    pub email: String,
}

/// 提交入驻申请。不建账号、不发邮件 —— 只是 `pending` 的一行（§11.4）。
pub async fn submit<C: ConnectionTrait>(db: &C, input: SubmitInput) -> Result<Uuid, ServiceError> {
    let shop_name = input.shop_name.trim().to_owned();
    let username = normalize_username(&input.username);
    let email = normalize_email(&input.email);

    validate_shop_name(&shop_name)?;
    validate_username(&username)?;
    validate_email(&email)?;

    // 待审唯一性：先查一次给友好提示；真正兜底是数据库的部分唯一索引
    if merchant_application::find_pending_by_username(db, &username)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict("该用户名已有待审申请".to_owned()));
    }

    if merchant_application::find_pending_by_email(db, &email)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict("该邮箱已有待审申请".to_owned()));
    }

    if merchant_application::find_pending_by_shop_name(db, &shop_name)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict("该店铺名已有待审申请".to_owned()));
    }

    let model = merchant_application::create(
        db,
        NewApplication {
            id: Uuid::now_v7(),
            shop_name,
            username,
            email,
        },
    )
    .await?;

    Ok(model.id)
}


#[derive(Debug)]
pub struct ApplicationPage {
    pub items: Vec<Model>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

/// 运营看待审列表。`status = None` 表示全部。
pub async fn list<C: ConnectionTrait>(
    db: &C,
    status: Option<ApplicationStatus>,
    page: u32,
    page_size: u32,
) -> Result<ApplicationPage, ServiceError> {
    // 参数夹取：翻页从 1 开始，每页最多 100（§19：分页上限是业务规则）
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);

    let (items, total) =
        merchant_application::list_page(db, status, page as u64, page_size as u64).await?;

    Ok(ApplicationPage {
        items,
        total,
        page,
        page_size,
    })
}


/// 审批通过：一个事务里改申请单 + 建店铺 + 建店主账号。
///
/// 店主账号 `password_hash = NULL`（未激活）—— 设密码走邀请链接（邮件通道待建）。
pub async fn approve(
    db: &DatabaseConnection,
    application_id: Uuid,
    reviewer_id: Uuid,
) -> Result<Uuid, ServiceError> {
    let txn = db.begin().await?;

    // 锁行：两个人同时点「通过」时，第二个会等第一个提交后再看到 status
    let Some(application) =
        merchant_application::find_by_id_for_update(&txn, application_id).await?
    else {
        return Err(ServiceError::StateChanged(
            "申请单不存在或已处理".to_owned(),
        ));
    };

    if application.status != ApplicationStatus::Pending {
        return Err(ServiceError::StateChanged("申请单已被处理".to_owned()));
    }

    // 跨表唯一性：申请表的部分索引管不到 merchants / shops
    if merchant::find_by_username(&txn, &application.username)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict("该用户名已被其他商户占用".to_owned()));
    }
    if merchant::find_by_email(&txn, &application.email)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict("该邮箱已被其他商户占用".to_owned()));
    }
    if shop::find_by_name(&txn, &application.shop_name)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict("该店铺名已被占用".to_owned()));
    }

    let Some(owner_role) = merchant_role::find_by_permission(&txn, "owner").await? else {
        return Err(ServiceError::Internal("缺少 owner 角色".to_owned()));
    };

    let shop = shop::create(
        &txn,
        shop::NewShop {
            id: Uuid::now_v7(),
            name: application.shop_name.clone(),
        },
    )
    .await?;

    let merchant = merchant::create(
        &txn,
        merchant::NewMerchant {
            id: Uuid::now_v7(),
            shop_id: shop.id,
            role_id: owner_role.id,
            username: application.username.clone(),
            password_hash: None,
            email: application.email.clone(),
        },
    )
    .await?;

    merchant_application::apply_review(
        &txn,
        application,
        ApplicationStatus::Approved,
        reviewer_id,
        None,
    )
    .await?;

    txn.commit().await?;

    Ok(merchant.id)
}


/// 审批拒绝：只改申请单，不建任何账号。拒绝原因必填。
pub async fn reject(
    db: &DatabaseConnection,
    application_id: Uuid,
    reviewer_id: Uuid,
    note: String,
) -> Result<(), ServiceError> {
    let note = note.trim().to_owned();
    if note.is_empty() {
        return Err(ServiceError::InvalidParams("拒绝原因不能为空".to_owned()));
    }

    let txn = db.begin().await?;

    let Some(application) =
        merchant_application::find_by_id_for_update(&txn, application_id).await?
    else {
        return Err(ServiceError::StateChanged(
            "申请单不存在或已处理".to_owned(),
        ));
    };

    if application.status != ApplicationStatus::Pending {
        return Err(ServiceError::StateChanged("申请单已被处理".to_owned()));
    }

    merchant_application::apply_review(
        &txn,
        application,
        ApplicationStatus::Rejected,
        reviewer_id,
        Some(note),
    )
    .await?;

    txn.commit().await?;

    Ok(())
}
