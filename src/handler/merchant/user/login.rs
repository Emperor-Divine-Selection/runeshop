use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::handler::response::ApiResponse;
use crate::service::merchant::user::merchant_login;
use crate::state::AppState;

use crate::handler::error::ApiError;

/// 登录请求体
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应体：只发 token
#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

/// POST /auth/login —— 用户名密码换 token
/// 用户名不存在或密码错误都返 400「用户名或密码错误」
pub async fn merchant_login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, ApiError> {
    let token = merchant_login(&state.db, &state.cache, &req.username, &req.password).await?;
    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        LoginResponse { token },
    )))
}
