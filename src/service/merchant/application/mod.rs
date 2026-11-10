use sea_orm::{ConnectionTrait, sqlx::types::Uuid};

use crate::model::merchant::merchant_applications::Model;
use crate::model::merchant::sea_orm_active_enums::ApplicationStatus;
use crate::service::common::validation::{MIN_PASSWORD_LEN, is_valid_email, validate};
use crate::service::error::ServiceError;
use crate::store::merchant::{find_merchant_by_email, find_merchant_by_username};
use crate::store::merchant_application::{
    NewMerchantApplication, UpdateApplicationContent, create_merchant_application,
    find_latest_application_by_username, find_merchant_application_by_id,
    find_merchant_application_by_shop_name, update_application_content,
};
use crate::store::password::hash_password;

const MAX_SHOP_NAME_LEN: usize = 50;

pub struct ApplyMerchant {
    pub username: String,
    pub password: String,
    pub email: String,
    pub shop_name: String,
}

/// 提交商户申请（公开接口，不需要登录）
///
/// 申请人自己填商户端的登录名和密码，审核通过后原样搬进 `merchants`
pub async fn apply_merchant<C: ConnectionTrait>(
    db: &C,
    mut req: ApplyMerchant,
) -> Result<Model, ServiceError> {
    normalize(&mut req);
    validate_shop_name(&req.shop_name)?;
    validate(&req.username, &req.email, &req.password)?;

    // 查重：不能和已有商户撞，也不能和在流程中的申请撞
    if find_merchant_by_username(db, &req.username).await?.is_some() {
        return Err(ServiceError::Conflict("该用户名已被商户使用".into()));
    }
    if find_merchant_by_email(db, &req.email).await?.is_some() {
        return Err(ServiceError::Conflict("该邮箱已被商户使用".into()));
    }
    if let Some(previous) = find_latest_application_by_username(db, &req.username).await? {
        if is_active(&previous.status) {
            return Err(ServiceError::Conflict(
                "该用户名已有待处理的申请".into(),
            ));
        }
    }
    if find_merchant_application_by_shop_name(db, &req.shop_name)
        .await?
        .is_some()
    {
        return Err(ServiceError::Conflict(format!(
            "店铺名称{}已被占用",
            req.shop_name
        )));
    }

    Ok(create_merchant_application(
        db,
        NewMerchantApplication {
            username: req.username,
            password_hash: hash_password(&req.password)?,
            email: req.email,
            shop_name: req.shop_name,
        },
    )
    .await?)
}

/// 查「我」这条申请；查不到返 `None`
pub async fn find_own_application<C: ConnectionTrait>(
    db: &C,
    application_id: Uuid,
) -> Result<Option<Model>, ServiceError> {
    Ok(find_merchant_application_by_id(db, application_id).await?)
}

pub struct SupplementApplication {
    pub shop_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

/// 补正资料；只有 `supplement_required` 状态能改，改完自动回到 `pending`
pub async fn supplement_application<C: ConnectionTrait>(
    db: &C,
    application_id: Uuid,
    req: SupplementApplication,
) -> Result<Model, ServiceError> {
    let Some(application) = find_merchant_application_by_id(db, application_id).await? else {
        return Err(ServiceError::NotFound("申请单不存在".into()));
    };
    if application.status != ApplicationStatus::SupplementRequired {
        return Err(ServiceError::Conflict("当前状态不允许修改".into()));
    }

    let shop_name = clean(req.shop_name);
    let email = clean(req.email).map(|value| value.to_lowercase());
    let password = clean(req.password);

    if shop_name.is_none() && email.is_none() && password.is_none() {
        return Err(ServiceError::InvalidParams("至少修改一项".into()));
    }
    if let Some(value) = shop_name.as_deref() {
        validate_shop_name(value)?;
    }
    if let Some(value) = email.as_deref() {
        if !is_valid_email(value) {
            return Err(ServiceError::InvalidParams("邮箱格式不正确".into()));
        }
    }
    if let Some(value) = password.as_deref() {
        if value.chars().count() < MIN_PASSWORD_LEN {
            return Err(ServiceError::InvalidParams(format!(
                "密码至少 {MIN_PASSWORD_LEN} 位"
            )));
        }
    }

    let password_hash = match password.as_deref() {
        Some(plain) => Some(hash_password(plain)?),
        None => None,
    };

    Ok(update_application_content(
        db,
        application_id,
        UpdateApplicationContent {
            shop_name,
            email,
            password_hash,
        },
    )
    .await?)
}

/// 归一化：去空白，用户名 / 邮箱转小写（和登录、注册规则保持一致）
fn normalize(req: &mut ApplyMerchant) {
    req.username = req.username.trim().to_lowercase();
    req.email = req.email.trim().to_lowercase();
    req.password = req.password.trim().to_string();
    req.shop_name = req.shop_name.trim().to_string();
}

fn validate_shop_name(shop_name: &str) -> Result<(), ServiceError> {
    if shop_name.is_empty() {
        return Err(ServiceError::InvalidParams("店铺名称不能为空".into()));
    }
    if shop_name.chars().count() > MAX_SHOP_NAME_LEN {
        return Err(ServiceError::InvalidParams(format!(
            "店铺名称不能超过 {MAX_SHOP_NAME_LEN} 个字"
        )));
    }
    Ok(())
}

/// 空串 / 纯空格一律当「没传」
fn clean(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn is_active(status: &ApplicationStatus) -> bool {
    matches!(
        status,
        ApplicationStatus::Pending | ApplicationStatus::SupplementRequired
    )
}
