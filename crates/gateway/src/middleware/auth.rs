use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use common::code::ApiCode;
use common::role::Role;

use crate::{
    cookie::session_cookie_name,
    error::ApiError,
    grpc::{BusinessStatus, Role as ProtoRole, ValidateSessionRequest},
    response::Envelope,
    state::GatewayState,
};

use super::CurrentUser;

/// 认证中间件：Cookie → ValidateSession → 校验角色 → 注入 CurrentUser
///
/// `state` 是个元组 —— 除了 GatewayState 还带着「这个端的路由要求哪个角色」。
/// 角色校验放在这里而不是 handler：将来 /merchant/*、/orders/* 都挂同一个函数，
/// 漏写一个就是越权洞，而编译器不会提醒（踩坑 94 的正面用法）。
///
/// 返回 `Response` 而不是 `Result`：未登录 / token 失效是**业务失败**
/// （HTTP 200 + code 1002），传输层故障才是 4xx/5xx —— 两类都在函数内处理完。
pub async fn auth_middleware(
    State((state, required)): State<(GatewayState, Role)>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    // 按端取各自的 Cookie 名 —— 三个端互不覆盖（踩坑 105）
    let Some(token) = jar
        .get(session_cookie_name(required))
        .map(|c| c.value().to_owned())
    else {
        return unauthorized();
    };

    let mut client = state.identity.clone();
    let reply = match client.validate_session(ValidateSessionRequest { token }).await {
        Ok(response) => response.into_inner(),
        Err(status) => return ApiError::from(status).into_response(),
    };

    let Some(status) = reply.status else {
        return ApiError::internal("identity 响应缺少 status").into_response();
    };
    if status.code != 0 {
        return Envelope::<()>::business(status).into_response();
    }

    if !reply.valid {
        return unauthorized();
    }

    let role = match ProtoRole::try_from(reply.role) {
        Ok(ProtoRole::Customer) => Role::Customer,
        Ok(ProtoRole::Merchant) => Role::Merchant,
        Ok(ProtoRole::Operator) => Role::Operator,
        _ => return unauthorized(),
    };

    // 跨端串用：商户的 token 打到顾客端 → 1002（客户端动作就是重新登录）
    if role != required {
        return unauthorized();
    }

    request.extensions_mut().insert(CurrentUser {
        id: reply.user_id,
        role,
    });

    next.run(request).await
}

/// 未登录 / token 失效 / 跨端串用：统一的业务失败（HTTP 200 + code 1002）。
fn unauthorized() -> Response {
    Envelope::<()>::business(BusinessStatus {
        code: ApiCode::Unauthorized as i32,
        message: "未登录或登录已过期".to_owned(),
    })
    .into_response()
}
