pub mod create;

use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

use crate::handler::common::status::product_status_text;
use crate::model::products::Model;

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub kinds: Vec<CreateKindRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKindRequest {
    pub name: String,
    /// 前端不传就用 0，不加 #[serde(default)] 会报 missing field
    #[serde(default)]
    pub sort_order: i32,
    pub items: Vec<CreateItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
    #[serde(default)]
    pub sort_order: i32,
    /// 用 String 收：JSON 的数字是 f64，99.99 会丢精度
    pub price: String,
    pub stock: i32,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub name: String,
    pub cover_image: Option<String>,
    pub status: String,
    pub create_time: DateTimeWithTimeZone,
}

impl From<Model> for ProductResponse {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            cover_image: value.cover_image,
            status: product_status_text(&value.status).to_string(),
            create_time: value.create_time,
        }
    }
}
