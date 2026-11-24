use axum::{Extension, Json, extract::State, http::header::SET_COOKIE};
use axum_extra::extract::CookieJar;
use common::role::Role;
use serde::{Deserialize, Serialize};

use crate::{
    cookie::{build_clear_cookie, build_session_cookie, session_cookie_name},
    error::ApiError,
    grpc::{GetMerchantRequest, LoginMerchantRequest, LogoutRequest},
    middleware::CurrentUser,
    response::Envelope,
    state::GatewayState,
};

#[derive(Deserialize)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

/// 商户登录：成功只回 `code: 0`，凭据在 `session_merchant` Cookie 里。
pub async fn login(
    State(state): State<GatewayState>,
    Json(body): Json<LoginBody>,
) -> Result<Envelope<()>, ApiError> {
    let mut client = state.identity.clone();

    let reply = client
        .login_merchant(LoginMerchantRequest {
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

    let value = build_session_cookie(Role::Merchant, reply.token, state.session_ttl_seconds)?;

    Ok(Envelope::ok(()).header(SET_COOKIE, value))
}

pub async fn logout(
    State(state): State<GatewayState>,
    jar: CookieJar,
) -> Result<Envelope<()>, ApiError> {
    // 尽力删服务端 session：失败只记日志，不能让用户「登不出去」
    if let Some(token) = jar
        .get(session_cookie_name(Role::Merchant))
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

    let value = build_clear_cookie(Role::Merchant)?;

    Ok(Envelope::ok(()).header(SET_COOKIE, value))
}

#[derive(Serialize)]
pub struct MeReply {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub shop_id: String,
    pub shop_name: String,
    pub permissions: Vec<String>,
}

pub async fn me(
    State(state): State<GatewayState>,
    Extension(current): Extension<CurrentUser>,
) -> Result<Envelope<MeReply>, ApiError> {
    let mut client = state.identity.clone();

    let reply = client
        .get_merchant(GetMerchantRequest {
            user_id: current.id, // ← 只能来自 token（§9.1）
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
        shop_id: reply.shop_id,
        shop_name: reply.shop_name,
        permissions: reply.permissions,
    }))
}
