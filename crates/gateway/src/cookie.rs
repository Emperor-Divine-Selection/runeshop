use axum::http::HeaderValue;
use axum_extra::extract::cookie::{Cookie, SameSite};
use common::role::Role;
use cookie::time::Duration;

use crate::error::ApiError;

/// 三个端各用各的 Cookie 名 —— 浏览器不按端口隔离 Cookie（踩坑 105）。
pub fn session_cookie_name(role: Role) -> &'static str {
    match role {
        Role::Customer => "session_customer",
        Role::Merchant => "session_merchant",
        Role::Operator => "session_operator",
    }
}

/// 登录时写 Cookie（§4.2 四个属性一个不能少）。
pub fn build_session_cookie(
    role: Role,
    token: String,
    ttl_seconds: u64,
) -> Result<HeaderValue, ApiError> {
    let cookie = Cookie::build((session_cookie_name(role), token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(Duration::seconds(ttl_seconds as i64))
        .build();

    HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| ApiError::internal("Set-Cookie 头构造失败"))
}

/// 登出时清 Cookie（名字 / Path 必须和写入时一致，否则删不掉）。
pub fn build_clear_cookie(role: Role) -> Result<HeaderValue, ApiError> {
    let cookie = Cookie::build((session_cookie_name(role), ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(Duration::ZERO)
        .build();

    HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| ApiError::internal("Set-Cookie 头构造失败"))
}
