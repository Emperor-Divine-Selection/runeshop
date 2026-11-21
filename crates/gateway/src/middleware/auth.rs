use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use common::code::ApiCode;
use common::role::Role;

use crate::{error::ApiError, grpc::ValidateSessionRequest, state::AppState};

use super::CurrentUser; // 或者本文件定义

pub async fn auth_middleware(
    State(state): State<AppState>,
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

    let role = Role::parse(&reply.role).ok_or(ApiError {
        code: ApiCode::Unauthorized,
        message: "登录状态异常".to_owned(),
    })?;

    request.extensions_mut().insert(CurrentUser {
        id: reply.user_id,
        role,
    });

    Ok(next.run(request).await)
}
