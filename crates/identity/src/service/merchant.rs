use common::password::verify_password;
use common::role::Role;
use sea_orm::ConnectionTrait;
use uuid::Uuid;

use crate::model::sea_orm_active_enums::MerchantStatus;
use crate::service::LoginInput;
use crate::service::credential::{session::SessionStore, token::generate_token};
use crate::service::error::ServiceError;
use crate::service::normalize::normalize_username;
use crate::store::merchant::{find_by_id, find_by_username};
use crate::store::{merchant_role, merchant_role_permission, shop};

/// 商户登录。
///
/// 顺序纪律（踩坑 78）：所有判定走完、token 算好，**最后**才写 Valkey。
pub async fn login<C: ConnectionTrait>(
    db: &C,
    session: &SessionStore,
    input: LoginInput,
) -> Result<String, ServiceError> {
    let username = normalize_username(&input.username);

    // 用户名不存在与密码错必须返回【同一条】错误（踩坑 51）
    let Some(merchant) = find_by_username(db, &username).await? else {
        return Err(ServiceError::InvalidCredentials(
            "用户名或密码错误".to_owned(),
        ));
    };

    // 未激活（还没通过邀请链接设密码）：同样返回「用户名或密码错误」，
    // 否则等于告诉攻击者「这个账号存在，只是没激活」
    let Some(password_hash) = merchant.password_hash.as_deref() else {
        return Err(ServiceError::InvalidCredentials(
            "用户名或密码错误".to_owned(),
        ));
    };

    if !verify_password(password_hash, &input.password)? {
        return Err(ServiceError::InvalidCredentials(
            "用户名或密码错误".to_owned(),
        ));
    }

    // 密码验证通过之后才检查状态 —— 否则会泄露「账号存在但被冻结」
    if merchant.status != MerchantStatus::Active {
        return Err(ServiceError::Unauthorized("账号已被冻结".to_owned()));
    }

    let token = generate_token()?;
    session.save(&token, Role::Merchant, &merchant.id).await?;

    Ok(token)
}


/// 商户资料：账号 + 店铺 + 权限（`/merchant/me` 用这个）。
#[derive(Debug)]
pub struct MerchantProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub shop_id: Uuid,
    pub shop_name: String,
    pub permissions: Vec<String>,
}

pub async fn me_merchant<C: ConnectionTrait>(
    db: &C,
    merchant_id: Uuid,
) -> Result<MerchantProfile, ServiceError> {
    let Some(merchant) = find_by_id(db, merchant_id).await? else {
        return Err(ServiceError::Unauthorized("用户不存在".to_owned()));
    };

    // 冻结的账号即使还有效的 session，也不该继续拿资料
    if merchant.status != MerchantStatus::Active {
        return Err(ServiceError::Unauthorized("账号已被冻结".to_owned()));
    }

    let Some(shop) = shop::find_by_id(db, merchant.shop_id).await? else {
        tracing::error!(
            merchant_id = %merchant_id,
            shop_id = %merchant.shop_id,
            "商户的店铺不存在"
        );
        return Err(ServiceError::Internal("商户数据不完整".to_owned()));
    };

    let Some(role) = merchant_role::find_by_id(db, merchant.role_id).await? else {
        tracing::error!(
            merchant_id = %merchant_id,
            role_id = %merchant.role_id,
            "商户的角色不存在"
        );
        return Err(ServiceError::Internal("商户数据不完整".to_owned()));
    };

    // owner 返回 "*"（全部权限），和 §11.5 的特判保持一致
    let permissions = if role.permission == "owner" {
        vec!["*".to_owned()]
    } else {
        merchant_role_permission::find_permissions(db, merchant.role_id).await?
    };

    Ok(MerchantProfile {
        id: merchant.id,
        username: merchant.username,
        email: merchant.email,
        shop_id: shop.id,
        shop_name: shop.name,
        permissions,
    })
}
