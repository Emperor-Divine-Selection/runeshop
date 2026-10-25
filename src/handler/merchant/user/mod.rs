pub mod login;
pub mod me;

use sea_orm::prelude::*;
use serde::Serialize;

use crate::handler::merchant::status_text;
use crate::model::merchants::Model;

#[derive(Serialize)]
pub struct MerchantResponse {
    pub id: Uuid,
    pub shop_name: String,
    pub status: String,
    pub create_time: DateTimeWithTimeZone,
}

impl From<Model> for MerchantResponse {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            shop_name: value.shop_name,
            status: status_text(&value.status).to_string(),
            create_time: value.create_time,
        }
    }
}
