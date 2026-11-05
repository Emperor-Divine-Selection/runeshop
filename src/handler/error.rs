use crate::handler::response::ApiResponse;
use axum::{http::StatusCode, response::IntoResponse};

use crate::service::error::ServiceError;

pub enum ApiError {
    Service(ServiceError),
}

impl From<ServiceError> for ApiError {
    fn from(value: ServiceError) -> Self {
        Self::Service(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::Service(ServiceError::NotFound(msg)) => (StatusCode::NOT_FOUND, msg),
            Self::Service(ServiceError::Business(msg)) => (StatusCode::BAD_REQUEST, msg),
            Self::Service(ServiceError::InvalidParams(msg)) => {
                (StatusCode::UNPROCESSABLE_ENTITY, msg)
            }
            Self::Service(ServiceError::Conflict(msg)) => (StatusCode::CONFLICT, msg),
            Self::Service(ServiceError::Unauthorized(msg)) => (StatusCode::UNAUTHORIZED, msg),
            Self::Service(ServiceError::Forbidden(msg)) => (StatusCode::FORBIDDEN, msg),
            Self::Service(ServiceError::Database(e)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::Service(ServiceError::Cache(e)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::Service(ServiceError::Storage(e)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            Self::Service(ServiceError::Image(e)) => {
                // 细节进日志，响应只给一句人话 —— ImageError 的文案是英文技术细节
                tracing::warn!(%e, "图片处理失败");
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "图片格式不支持或文件已损坏".to_string(),
                )
            }
        };
        ApiResponse::err(status.as_u16(), message).into_response()
    }
}
