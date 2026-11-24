use axum::{Extension, Json, extract::State, http::header::SET_COOKIE};
use axum_extra::extract::CookieJar;
use common::role::Role;
use serde::{Deserialize, Serialize};

use crate::{
    cookie::{build_clear_cookie, build_session_cookie, session_cookie_name},
    error::ApiError,
    grpc::{GetCustomerRequest, LoginCustomerRequest, LogoutRequest, RegisterCustomerRequest},
    middleware::CurrentUser,
    response::Envelope,
    state::GatewayState,
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
    State(state): State<GatewayState>,
    Json(body): Json<RegisterBody>,
) -> Result<Envelope<RegisterReply>, ApiError> {
    let mut client = state.identity.clone();

    let reply = client
        .register_customer(RegisterCustomerRequest {
            username: body.username,
            password: body.password,
            email: body.email,
        })
        .await?
        .into_inner();

    let Some(status) = reply.status else {
        return Err(ApiError::internal("identity 响应缺少 status"));
    };
    if status.code != 0 {
        return Ok(Envelope::business(status));
    }

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
/// 用户信息一律由 `/customer/auth/me` 提供：那是唯一入口，登录不再复制一份
/// （否则同一个状态有两个来源，迟早漂移）。
pub async fn login(
    State(state): State<GatewayState>,
    Json(body): Json<LoginBody>,
) -> Result<Envelope<()>, ApiError> {
    let mut client = state.identity.clone();

    let reply = client
        .login_customer(LoginCustomerRequest {
            username: body.username,
            password: body.password,
        })
        .await?
        .into_inner();

    let Some(status) = reply.status else {
        return Err(ApiError::internal("identity 响应缺少 status"));
    };
    if status.code != 0 {
        return Ok(Envelope::business(status));
    }

    if reply.token.is_empty() {
        return Err(ApiError::internal("identity 返回了空 token"));
    }

    let value = build_session_cookie(Role::Customer, reply.token, state.session_ttl_seconds)?;

    Ok(Envelope::ok(()).header(SET_COOKIE, value))
}

pub async fn logout(
    State(state): State<GatewayState>,
    jar: CookieJar,
) -> Result<Envelope<()>, ApiError> {
    // 尽力删服务端 session：失败只记日志，不能让用户「登不出去」
    if let Some(token) = jar
        .get(session_cookie_name(Role::Customer))
        .map(|cookie| cookie.value().to_owned())
    {
        let mut client = state.identity.clone();

        match client.logout(LogoutRequest { token }).await {
            Ok(response) => match response.into_inner().status {
                Some(status) if status.code != 0 => {
                    tracing::warn!(
                        code = status.code,
                        message = %status.message,
                        "identity 登出返回业务错误"
                    );
                }
                None => tracing::warn!("identity 登出响应缺少 status"),
                Some(_) => {}
            },
            Err(status) => {
                tracing::error!(
                    grpc_code = ?status.code(),
                    message = status.message(),
                    "调用 identity 登出失败"
                );
            }
        }
    }

    let value = build_clear_cookie(Role::Customer)?;

    Ok(Envelope::ok(()).header(SET_COOKIE, value))
}

/// `/customer/auth/me` 的响应。
///
/// 字段和 §7 的规格一一对应 —— 这是用户信息**唯一**的出口，登录只回 `code: 0`。
#[derive(Serialize)]
pub struct MeReply {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
}

pub async fn me(
    State(state): State<GatewayState>,
    Extension(current): Extension<CurrentUser>,
) -> Result<Envelope<MeReply>, ApiError> {
    let mut client = state.identity.clone();

    let reply = client
        .get_customer(GetCustomerRequest {
            user_id: current.id, // ← 只能来自 token，不能来自请求参数（§9.1）
        })
        .await?
        .into_inner();

    let Some(status) = reply.status else {
        return Err(ApiError::internal("identity 响应缺少 status"));
    };
    if status.code != 0 {
        return Ok(Envelope::business(status));
    }

    Ok(Envelope::ok(MeReply {
        id: reply.id,
        username: reply.username,
        email: reply.email,
        role: current.role.as_str().to_owned(),
    }))
}
