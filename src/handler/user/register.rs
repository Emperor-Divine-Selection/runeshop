use axum::{Json, extract::State, http::StatusCode};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

use crate::model::users::Model;
use crate::service::user::register;
use crate::state::AppState;
use crate::store::user::NewUser;

use crate::handler::errors::ApiError;
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

/// 注册响应体（脱敏：绝不含 password_hash）
#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub create_time: DateTimeWithTimeZone,
}

impl From<Model> for UserResponse {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            avatar: value.avatar,
            bio: value.bio,
            create_time: value.create_time,
        }
    }
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
