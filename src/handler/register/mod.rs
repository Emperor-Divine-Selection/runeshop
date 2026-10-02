use axum::{Json, extract::State, http::StatusCode};
use sea_orm::{DatabaseConnection, prelude::*};
use serde::{Deserialize, Serialize};

use crate::model::users::Model;
use crate::service::user::register;
use crate::store::user::NewUser;

use super::errors::ApiError;

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
    pub id: i32,
    pub username: String,
    pub email: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub create_time: DateTimeWithTimeZone,
}

impl From<Model> for UserResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            username: m.username,
            email: m.email,
            avatar: m.avatar,
            bio: m.bio,
            create_time: m.create_time,
        }
    }
}

/// POST /users —— 注册；用户名/邮箱重复返回 409
pub async fn register_handler(
    State(db): State<DatabaseConnection>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    let model = register(
        &db,
        NewUser {
            username: req.username,
            password: req.password,
            email: req.email,
            avatar: req.avatar,
            bio: req.bio,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(model.into())))
}
