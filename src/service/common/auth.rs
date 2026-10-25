use crate::{
    cache::token::{self, TokenKind},
    service::error::ServiceError,
};
use fred::clients::Client;
use sea_orm::sqlx::types::Uuid;

/// 校验 token 有效且属于 `expected` 端，返回 user_id
pub async fn authenticate(
    cache: &Client,
    token: &str,
    expected: TokenKind,
) -> Result<Uuid, ServiceError> {
    let Some((kind, user_id)) = token::get(cache, token).await? else {
        return Err(ServiceError::Unauthorized("登录状态已失效".into()));
    };
    if kind != expected {
        return Err(ServiceError::Unauthorized("token 不属于当前端".into()));
    }
    Ok(user_id)
}
