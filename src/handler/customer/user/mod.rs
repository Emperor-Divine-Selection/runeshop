pub mod avatar;
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

impl UserResponse {
    /// `avatar` 在库里存的是对象 key，对外要拼成完整 URL
    ///
    /// 拼 URL 的规则只存在于两处：这里和 `handler/customer/user/avatar.rs`
    pub fn from_model(model: Model, storage_public_url: &str) -> Self {
        Self {
            id: model.id,
            username: model.username,
            email: model.email,
            avatar: model
                .avatar
                .map(|key| format!("{storage_public_url}/{key}")),
            bio: model.bio,
            create_time: model.create_time,
        }
    }
}
