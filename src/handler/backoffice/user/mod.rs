// src/handler/backoffice/user/mod.rs
pub mod login;
pub mod me;
pub mod register;

use crate::model::backoffice_users::Model;
use sea_orm::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct BackofficeUserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub create_time: DateTimeWithTimeZone,
}

impl From<Model> for BackofficeUserResponse {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            create_time: value.create_time,
        }
    }
}
