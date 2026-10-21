pub mod login;
pub mod me;
pub mod register;

use sea_orm::prelude::*;
use serde::Serialize;

use crate::model::users::Model;

/// 用户出参（脱敏：绝不含 password_hash）
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
