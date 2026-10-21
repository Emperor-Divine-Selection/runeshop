use crate::handler::error::ApiError;
use crate::service::common::auth::authenticate;
use crate::service::error::ServiceError::Unauthorized;
use crate::state::AppState;

use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;

use sea_orm::sqlx::types::Uuid;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(&parts.headers)?;
        let user_id = authenticate(&state.cache, token).await?;
        Ok(AuthUser { user_id })
    }
}

/// 从 `Authorization: Bearer <token>` 里取出 token
///
/// 失败一律 401：缺头 / 头不是合法字符串 / 方案名不对 / token 为空
pub fn bearer_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    let Some(header) = headers.get(AUTHORIZATION) else {
        return Err(ApiError::Service(Unauthorized(
            "缺少 Authorization 头".into(),
        )));
    };

    let Ok(header) = header.to_str() else {
        return Err(ApiError::Service(Unauthorized(
            "Authorization 头不是合法字符串".into(),
        )));
    };

    let Some((scheme, token)) = header.split_once(' ') else {
        return Err(ApiError::Service(Unauthorized(
            "Authorization 头格式应为 Bearer <token>".into(),
        )));
    };

    if !scheme.eq_ignore_ascii_case("bearer") {
        return Err(ApiError::Service(Unauthorized(
            "Authorization 头格式应为 Bearer <token>".into(),
        )));
    };

    let token = token.trim();
    if token.is_empty() {
        return Err(ApiError::Service(Unauthorized("token 不能为空".into())));
    }

    Ok(token)
}
