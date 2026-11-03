pub mod get;

use sea_orm::prelude::*;
use serde::Serialize;

use crate::handler::common::status::application_status_text;
use crate::model::merchant_applications::Model;

/// 顾客视角的申请单：带审核意见，方便前端直接展示「为什么被驳回」
#[derive(Serialize)]
pub struct CustomerApplicationResponse {
    pub id: Uuid,
    pub shop_name: String,
    pub status: String,
    pub create_time: DateTimeWithTimeZone,
    pub reviewed_at: Option<DateTimeWithTimeZone>,
    pub review_remark: Option<String>,
}

impl From<Model> for CustomerApplicationResponse {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            shop_name: value.shop_name,
            status: application_status_text(&value.status).to_string(),
            create_time: value.create_time,
            reviewed_at: value.reviewed_at,
            review_remark: value.review_remark,
        }
    }
}
