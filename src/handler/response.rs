use axum::response::{IntoResponse, Response};
use axum::{Json, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    code: u16,
    data: Option<T>,
    msg: String,
}

impl<T> ApiResponse<T> {
    pub fn ok(code: u16, data: T) -> Self {
        Self {
            code,
            msg: "ok".into(),
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
    pub fn err(code: u16, msg: impl Into<String>) -> Self {
        Self {
            code,
            msg: msg.into(),
            data: None,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::OK);
        (status, Json(self)).into_response()
    }
}
