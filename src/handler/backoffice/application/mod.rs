pub mod approve;
pub mod list;
pub mod reject;
pub mod supplement;

use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sqlx::types::Uuid;
use serde::{Deserialize, Serialize};

use crate::handler::common::status::application_status_text;
use crate::model::merchant::merchant_applications::Model;
use crate::model::merchant::sea_orm_active_enums::ApplicationStatus;
use crate::service::error::ServiceError;

#[derive(Deserialize)]
pub struct ReviewRequest {
    #[serde(default)] // 允许 body 里不写这个字段
    pub remark: Option<String>,
}

/// 一条申请单的对外形态
///
/// 申请人信息（username / email）现在直接来自申请单本身 —— 申请不再挂顾客账号
#[derive(Serialize)]
pub struct ApplicationResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub shop_name: String,
    pub status: String,                // pending / approved / rejected / supplement_required
    pub reviewer_name: Option<String>, // 审核人名字（跨域批量查出来的）
    pub review_remark: Option<String>,
    pub create_time: DateTimeWithTimeZone,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTimeWithTimeZone>,
}

impl ApplicationResponse {
    pub fn from_model(application: Model, reviewer_name: Option<String>) -> Self {
        Self {
            id: application.id,
            username: application.username,
            email: application.email,
            shop_name: application.shop_name,
            status: application_status_text(&application.status).to_string(),
            reviewer_name,
            review_remark: application.review_remark,
            create_time: application.create_time,
            reviewed_by: application.reviewed_by,
            reviewed_at: application.reviewed_at,
        }
    }
}

impl From<Model> for ApplicationResponse {
    /// 审核动作的返回值没有审核人名字（就是当前登录的人），不带
    fn from(application: Model) -> Self {
        Self::from_model(application, None)
    }
}

/// 查询参数里的状态字符串 → 枚举；未知值返 422
pub fn parse_status(value: &str) -> Result<ApplicationStatus, ServiceError> {
    match value.trim().to_lowercase().as_str() {
        "pending" => Ok(ApplicationStatus::Pending),
        "approved" => Ok(ApplicationStatus::Approved),
        "rejected" => Ok(ApplicationStatus::Rejected),
        "supplement_required" => Ok(ApplicationStatus::SupplementRequired),
        other => Err(ServiceError::InvalidParams(format!(
            "未知的申请状态：{other}"
        ))),
    }
}
