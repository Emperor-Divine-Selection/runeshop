use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use common::response::ApiResponse;
use serde::Serialize;

/// 成功响应的 HTTP 包装。
///
/// `common::ApiResponse` 只描述「信封长什么样」；把它变成 HTTP 响应是 gateway 的事。
/// 不能把 `IntoResponse` 写在 common —— 那会让 identity / catalog / order 都被迫依赖 axum
/// （§2：只有 gateway 说 HTTP）。这里用 newtype 绕过孤儿规则。
pub struct ApiOk<T> {
    body: ApiResponse<T>,
    status: StatusCode,
}

impl<T: Serialize> ApiOk<T> {
    pub fn ok(data: T) -> Self {
        Self {
            body: ApiResponse::ok(data),
            status: StatusCode::OK,
        }
    }

    /// 注册成功用 201（§7），而不是 200。
    pub fn created(data: T) -> Self {
        Self {
            body: ApiResponse::ok(data),
            status: StatusCode::CREATED,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiOk<T> {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
