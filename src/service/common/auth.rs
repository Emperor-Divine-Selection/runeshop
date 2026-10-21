use crate::{cache::token, service::error::ServiceError};
use fred::clients::Client;
use sea_orm::sqlx::types::Uuid;

pub async fn authenticate(cache: &Client, token: &str) -> Result<Uuid, ServiceError> {
    let user_id = token::get(cache, token)
        .await?
        .ok_or_else(|| ServiceError::Unauthorized("登录状态已失效".into()))?;
    Ok(user_id)
}
