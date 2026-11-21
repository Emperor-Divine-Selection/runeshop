use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderValue, header::SET_COOKIE},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use cookie::time::Duration;
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    grpc::{GetAccountRequest, LoginRequest, LogoutRequest, RegisterRequest, Role as ProtoRole},
    middleware::CurrentUser,
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
        .await?
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
            role: ProtoRole::Customer as i32,
        })
        .await?
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

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Envelope<()>, ApiError> {
    if let Some(token) = jar.get("session").map(|cookie| cookie.value().to_owned()) {
        let mut client = state.identity.clone();
        client.logout(LogoutRequest { token }).await?;
    }

    let cleared = Cookie::build(("session", ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(Duration::ZERO)
        .build();

    let value = HeaderValue::from_str(&cleared.to_string())
        .map_err(|_| ApiError::internal("Set-Cookie 头构造失败"))?;

    Ok(Envelope::ok(()).header(SET_COOKIE, value))
}

/// `/customer/auth/me` 的响应。
///
/// 字段和 `/auth/me` 的规格（§7）一一对应 —— 这是用户信息**唯一**的出口，
/// 登录只回 `code: 0`。
#[derive(Serialize)]
pub struct MeReply {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
}

pub async fn me(
    State(state): State<AppState>,
    Extension(current): Extension<CurrentUser>,
) -> Result<Envelope<MeReply>, ApiError> {
    let mut client = state.identity.clone();

    let reply = client
        .get_account(GetAccountRequest {
            role: ProtoRole::Customer as i32,
            user_id: current.id, // ← 只能来自 token，不能来自请求参数（§9.1）
        })
        .await?
        .into_inner();

    Ok(Envelope::ok(MeReply {
        id: reply.id,
        username: reply.username,
        email: reply.email,
        role: current.role.as_str().to_owned(),
    }))
}
