use crate::model::sea_orm_active_enums::MerchantStatus;

use sea_orm::ConnectionTrait;

use crate::{
    cache::token::{generate, save},
    store::merchant::find_merchant_by_user_id,
    store::user::{find_by_username, verify_password},
};

use crate::service::error::ServiceError;

/// 用户登陆
pub async fn merchant_login<C: ConnectionTrait>(
    db: &C,
    cache: &fred::clients::Client,
    username: &str,
    password: &str,
) -> Result<String, ServiceError> {
    let username = username.trim().to_lowercase();

    let Some(user) = find_by_username(db, &username).await? else {
        return Err(ServiceError::Unauthorized("用户名或密码错误".into()));
    };
    if !verify_password(&user.password_hash, password).await {
        return Err(ServiceError::Unauthorized("用户名或密码错误".into()));
    };

    let Some(merchant) = find_merchant_by_user_id(db, user.id).await? else {
        return Err(ServiceError::NotFound("你不是商户".into()));
    };

    match merchant.status {
        MerchantStatus::Active => {
            let token = generate();
            save(cache, &token, user.id, None).await?;
            Ok(token)
        }
        MerchantStatus::Pending => Err(ServiceError::Business("商户申请审核中".into())),
        MerchantStatus::Frozen => Err(ServiceError::Business("商户已被冻结".into())),
        MerchantStatus::Rejected => Err(ServiceError::Business("商户申请已被拒绝".into())),
    }
}
