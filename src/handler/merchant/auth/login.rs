use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::handler::error::ApiError;
use crate::handler::response::ApiResponse;
use crate::service::merchant::auth::{LoginRole, merchant_login};
use crate::state::AppState;

/// 登录请求体
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应体
///
/// `role` 告诉前端该跳哪一页：商户进店铺页，申请人进补正页
#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub role: &'static str,
}

/// POST /merchant/auth/login
///
/// **商户和申请人共用**。用户名不存在或密码错误都返「用户名或密码错误」（防枚举）
pub async fn merchant_login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, ApiError> {
    let (token, role) =
        merchant_login(&state.db, &state.cache, &req.username, &req.password).await?;

    let role = match role {
        LoginRole::Merchant => "merchant",
        LoginRole::Applicant => "applicant",
    };

    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        LoginResponse { token, role },
    )))
}
