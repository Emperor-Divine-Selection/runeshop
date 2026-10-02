use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::service::errors::ServiceError;

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
            Self::Service(ServiceError::Technical(e)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
