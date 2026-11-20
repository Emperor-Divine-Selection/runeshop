use axum::{
    Json,
    extract::State,
    http::{HeaderValue, header::SET_COOKIE},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use cookie::time::Duration;
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    grpc::{LoginRequest, RegisterRequest},
    response::Envelope,
    state::AppState,
};

#[derive(Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct RegisterReply {
    pub id: String,
    pub username: String,
    pub email: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<Envelope<RegisterReply>, ApiError> {
    let mut client = state.identity.clone();

    let reply = client
        .register(RegisterRequest {
            username: body.username,
            password: body.password,
            email: body.email,
        })
        .await
        .map_err(ApiError::from)?
        .into_inner();

    Ok(Envelope::created(RegisterReply {
        id: reply.id,
        username: reply.username,
        email: reply.email,
    }))
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

/// 登录成功只回 `code: 0`（`data: null`）—— 凭据在 Set-Cookie 里。
///
/// 用户信息一律由 `GET /auth/me` 提供：那是唯一入口，登录不再复制一份
/// （否则同一个状态有两个来源，迟早漂移）。
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<Envelope<()>, ApiError> {
    let mut client = state.identity.clone();

    let reply = client
        .login(LoginRequest {
            username: body.username,
            password: body.password,
        })
        .await
        .map_err(ApiError::from)?
        .into_inner();

    let cookie = Cookie::build(("session", reply.token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(Duration::seconds(state.session_ttl_seconds as i64))
        .build();

    // token 是 base64url（A-Za-z0-9-_），不会出现非法头字符；
    // 但类型上仍是 Result —— 走 ? 而不是 unwrap（请求路径禁止 panic）
    let value = HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| ApiError::internal("Set-Cookie 头构造失败"))?;

    Ok(Envelope::ok(()).header(SET_COOKIE, value))
}
