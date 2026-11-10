pub mod apply;
pub mod me;
pub mod supplement;

use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

use crate::handler::common::status::application_status_text;
use crate::model::merchant::merchant_applications::Model;

/// 申请人视角的申请单：带审核意见，方便前端展示「为什么被打回 / 驳回」
#[derive(Serialize)]
pub struct ApplicantApplicationResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub shop_name: String,
    pub status: String,
    pub create_time: DateTimeWithTimeZone,
    pub reviewed_at: Option<DateTimeWithTimeZone>,
    pub review_remark: Option<String>,
}

impl From<Model> for ApplicantApplicationResponse {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            shop_name: value.shop_name,
            status: application_status_text(&value.status).to_string(),
            create_time: value.create_time,
            reviewed_at: value.reviewed_at,
            review_remark: value.review_remark,
        }
    }
}

/// 提交申请（公开接口）
///
/// 申请人自己填商户端的登录名和密码 —— 审核通过后就是商户账号
#[derive(Deserialize)]
pub struct ApplyRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub shop_name: String,
}

/// 补正资料：三个字段都可选，但至少要改一项
#[derive(Deserialize)]
pub struct SupplementRequest {
    pub shop_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}
