use crate::model::merchant::merchants::Model;
use crate::model::merchant::sea_orm_active_enums::ApplicationStatus;
use crate::model::merchant::sea_orm_active_enums::MerchantStatus;

use sea_orm::{ConnectionTrait, sqlx::types::Uuid};

use crate::{
    cache::token::{generate, save, TokenKind},
    store::merchant::{find_merchant_by_id, find_merchant_by_username},
    store::merchant_application::find_latest_application_by_username,
    store::password::verify_password,
};

use crate::service::error::ServiceError;

/// 登录成功后拿到的是哪种身份 —— 前端要据此决定跳哪一页
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginRole {
    Merchant,
    Applicant,
}

/// 按 merchant_id 取商户并校验状态；提取器每次请求都要过这一关
///
/// token 只证明「你是谁」，不证明「你现在还能不能进这个门」
pub async fn authenticate_merchant<C: ConnectionTrait>(
    db: &C,
    merchant_id: Uuid,
) -> Result<Model, ServiceError> {
    let Some(merchant) = find_merchant_by_id(db, merchant_id).await? else {
        return Err(ServiceError::Forbidden("不是商户".into()));
    };
    if merchant.status != MerchantStatus::Active {
        return Err(ServiceError::Forbidden("商户未激活".into()));
    }
    Ok(merchant)
}

/// 商户端登录：**商户和申请人共用一个入口**
///
/// 1. 先查 `merchants`：是商户 → 验密码 → 按状态发 Merchant token
/// 2. 不是商户 → 查最近一条申请：验密码 → 按状态回不同结果
///
/// ⚠️ 「用户名不存在」和「密码错误」必须回同一句话（防枚举）
pub async fn merchant_login<C: ConnectionTrait>(
    db: &C,
    cache: &fred::clients::Client,
    username: &str,
    password: &str,
) -> Result<(String, LoginRole), ServiceError> {
    let username = username.trim().to_lowercase();
    let password = password.trim();

    if let Some(merchant) = find_merchant_by_username(db, &username).await? {
        return login_as_merchant(cache, merchant, password).await;
    }

    let Some(application) = find_latest_application_by_username(db, &username).await? else {
        return Err(ServiceError::Unauthorized("用户名或密码错误".into()));
    };
    if !verify_password(&application.password_hash, password) {
        return Err(ServiceError::Unauthorized("用户名或密码错误".into()));
    }

    match application.status {
        ApplicationStatus::SupplementRequired => {
            let token = generate();
            save(
                cache,
                &token,
                TokenKind::Applicant,
                application.id,
                None,
            )
            .await?;
            Ok((token, LoginRole::Applicant))
        }
        ApplicationStatus::Pending => Err(ServiceError::Business(
            "申请正在审核中，请耐心等待".into(),
        )),
        ApplicationStatus::Rejected => Err(ServiceError::Business(format!(
            "申请已被驳回{}",
            application
                .review_remark
                .map(|remark| format!("：{remark}"))
                .unwrap_or_default()
        ))),
        ApplicationStatus::Approved => Err(ServiceError::Business(
            "申请已通过，请重新登录".into(),
        )),
    }
}

/// 商户分支：验密码 + 查状态 → 发 token
async fn login_as_merchant(
    cache: &fred::clients::Client,
    merchant: Model,
    password: &str,
) -> Result<(String, LoginRole), ServiceError> {
    if !verify_password(&merchant.password_hash, password) {
        return Err(ServiceError::Unauthorized("用户名或密码错误".into()));
    }

    match merchant.status {
        MerchantStatus::Active => {
            let token = generate();
            // 注意：signature 是 merchant.id，不是顾客 id —— 商户已经是独立账号
            save(cache, &token, TokenKind::Merchant, merchant.id, None).await?;
            Ok((token, LoginRole::Merchant))
        }
        MerchantStatus::Frozen => Err(ServiceError::Business("商户已被冻结".into())),
    }
}
