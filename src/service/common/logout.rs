use crate::cache::token::delete;
use crate::service::error::ServiceError;

/// 用户登出
pub async fn logout(cache: &fred::clients::Client, token: &str) -> Result<(), ServiceError> {
    Ok(delete(cache, token).await?)
}
