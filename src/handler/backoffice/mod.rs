pub mod application;
pub mod auth;
pub mod profile;

use crate::model::identity::operators::Model;
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
