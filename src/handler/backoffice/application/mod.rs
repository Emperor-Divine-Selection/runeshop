pub mod approve;
pub mod list;
pub mod reject;

use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sqlx::types::Uuid;
use serde::Serialize;

use crate::handler::common::status::application_status_text;
use crate::model::merchant_applications::Model;
use crate::model::sea_orm_active_enums::ApplicationStatus;
use crate::model::users::Model as UsersModel;
use crate::service::error::ServiceError;

/// 一条申请单的对外形态
#[derive(Serialize)]
pub struct ApplicationResponse {
    pub id: Uuid,
    pub shop_name: String,
    pub status: String,           // "pending" / "approved" / "rejected"
    pub username: Option<String>, // 申请人，列表才有
    pub create_time: DateTimeWithTimeZone,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTimeWithTimeZone>,
}

impl From<Model> for ApplicationResponse {
    fn from(application: Model) -> Self {
        Self {
            id: application.id,
            shop_name: application.shop_name,
            status: application_status_text(&application.status).to_string(),
            username: None, // 单条查询（审核结果）不带申请人
            create_time: application.create_time,
            reviewed_by: application.reviewed_by,
            reviewed_at: application.reviewed_at,
        }
    }
}

impl From<(Model, Option<UsersModel>)> for ApplicationResponse {
    fn from((application, user): (Model, Option<UsersModel>)) -> Self {
        let mut response = Self::from(application);
        response.username = user.map(|u| u.username);
        response
    }
}

/// 查询参数里的状态字符串 → 枚举；未知值返 422
pub fn parse_status(value: &str) -> Result<ApplicationStatus, ServiceError> {
    match value.trim().to_lowercase().as_str() {
        "pending" => Ok(ApplicationStatus::Pending),
        "approved" => Ok(ApplicationStatus::Approved),
        "rejected" => Ok(ApplicationStatus::Rejected),
        other => Err(ServiceError::InvalidParams(format!(
            "未知的申请状态：{other}"
        ))),
    }
}
