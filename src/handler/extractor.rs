use crate::cache::token::TokenKind;
use crate::handler::error::ApiError;
use crate::model::backoffice_users::Model as BackofficeUserModel;
use crate::model::merchants::Model as MerchantModel;
use crate::service::backoffice::user::authenticate_backoffice;
use crate::service::common::auth::authenticate;
use crate::service::error::ServiceError::Unauthorized;
use crate::service::merchant::user::authenticate_merchant;
use crate::state::AppState;

use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;

use sea_orm::sqlx::types::Uuid;

#[derive(Debug, Clone)]
pub struct CustomerUser {
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for CustomerUser {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(&parts.headers)?;
        let user_id = authenticate(&state.cache, token, TokenKind::Customer).await?;
        Ok(CustomerUser { user_id })
    }
}

#[derive(Debug, Clone)]
pub struct MerchantUser {
    pub merchant: MerchantModel,
}

impl FromRequestParts<AppState> for MerchantUser {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(&parts.headers)?;
        let user_id = authenticate(&state.cache, token, TokenKind::Merchant).await?;
        let merchant = authenticate_merchant(&state.db, user_id).await?;
        Ok(MerchantUser { merchant })
    }
}

#[derive(Debug, Clone)]
pub struct BackofficeUser {
    pub user: BackofficeUserModel,
}

impl FromRequestParts<AppState> for BackofficeUser {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(&parts.headers)?;
        let user_id = authenticate(&state.cache, token, TokenKind::Backoffice).await?;
        let user = authenticate_backoffice(&state.db, user_id).await?;
        Ok(BackofficeUser { user })
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
    }

    let token = token.trim();
    if token.is_empty() {
        return Err(ApiError::Service(Unauthorized("token 不能为空".into())));
    }

    Ok(token)
}
