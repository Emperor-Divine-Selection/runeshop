use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::handler::customer::user::UserResponse;
use crate::service::customer::user::register;
use crate::state::AppState;
use crate::store::user::NewUser;

use crate::handler::error::ApiError;
use crate::handler::response::ApiResponse;

/// 注册请求体
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
}

/// POST /register —— 注册；用户名/邮箱重复返回 409
pub async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, ApiError> {
    let model = register(
        &state.db,
        NewUser {
            username: req.username,
            password: req.password,
            email: req.email,
            avatar: req.avatar,
            bio: req.bio,
        },
    )
    .await?;
    Ok(Json(ApiResponse::ok(
        StatusCode::CREATED.as_u16(),
        model.into(),
    )))
}
