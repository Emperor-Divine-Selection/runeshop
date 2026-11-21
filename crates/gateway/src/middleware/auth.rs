use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use common::code::ApiCode;
use common::role::Role;

use crate::{
    error::ApiError, grpc::Role as ProtoRole, grpc::ValidateSessionRequest, state::AppState,
};

use super::CurrentUser;

/// 认证中间件：Cookie → ValidateSession → 校验角色 → 注入 CurrentUser
///
/// `state` 是个元组 —— 除了 AppState 还带着「这个端的路由要求哪个角色」。
/// 角色校验放在这里而不是 handler：将来 /merchant/*、/orders/* 都挂同一个函数，
/// 漏写一个就是越权洞，而编译器不会提醒（踩坑 94 的正面用法）。
pub async fn auth_middleware(
    State((state, required)): State<(AppState, Role)>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let Some(token) = jar.get("session").map(|c| c.value().to_owned()) else {
        return Err(ApiError {
            code: ApiCode::Unauthorized,
            message: "未登录或登录已过期".to_owned(),
        });
    };

    let mut client = state.identity.clone();
    let reply = client
        .validate_session(ValidateSessionRequest { token })
        .await?
        .into_inner();

    if !reply.valid {
        return Err(ApiError {
            code: ApiCode::Unauthorized,
            message: "未登录或登录已过期".to_owned(),
        });
    }

    let role = match ProtoRole::try_from(reply.role) {
        Ok(ProtoRole::Customer) => Role::Customer,
        Ok(ProtoRole::Merchant) => Role::Merchant,
        Ok(ProtoRole::Operator) => Role::Operator,
        _ => {
            return Err(ApiError {
                code: ApiCode::Unauthorized,
                message: "登录状态异常".to_owned(),
            });
        }
    };

    // 跨端串用：商户的 token 打到顾客端 → 401 + 1002。
    // 语义上更像 403，但客户端动作和「未登录」一样（去登录页），
    // 按 §6 的判据合并成同一个码。
    if role != required {
        return Err(ApiError {
            code: ApiCode::Unauthorized,
            message: "未登录或登录已过期".to_owned(),
        });
    }

    request.extensions_mut().insert(CurrentUser {
        id: reply.user_id,
        role,
    });

    Ok(next.run(request).await)
}
