use std::fmt::Display;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use common::code::ApiCode;
use common::response::{ApiResponse, Meta};
use tonic::Status;

/// 传输层错误。
///
/// **只表示「这次调用在网络 / 协议 / 服务可用性上出了问题」**，不携带业务语义。
/// 业务失败走 `Envelope::business(...)`（HTTP 200 + 信封里的 code）。
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    /// 我们自己发现了「不该发生」的情况（比如 identity 响应缺字段）。
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.status.as_u16(), self.message)
    }
}

impl std::error::Error for ApiError {}

/// gRPC 传输状态 → HTTP 状态码。
///
/// 5xx 不转发原始 message（可能含 SQL / 表名 / 地址），只记日志；
/// 4xx 是调用方的问题，可以把 message 给出去。
impl From<Status> for ApiError {
    fn from(value: Status) -> Self {
        let status = match value.code() {
            tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
            tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
            tonic::Code::PermissionDenied => StatusCode::FORBIDDEN,
            tonic::Code::NotFound => StatusCode::NOT_FOUND,
            tonic::Code::AlreadyExists => StatusCode::CONFLICT,
            tonic::Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
            tonic::Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            tonic::Code::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if status.is_server_error() {
            tracing::error!(
                grpc_code = ?value.code(),
                grpc_message = value.message(),
                "调用 identity 失败"
            );
            return Self {
                status,
                message: "服务暂时不可用".to_owned(),
            };
        }

        Self {
            status,
            message: value.message().to_owned(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ApiResponse::<()> {
            // 传输层失败没有业务码，统一用 5000；HTTP status 负责区分 400 / 503 / 504
            code: ApiCode::Internal as i32,
            message: self.message,
            data: None,
            meta: Meta::default(),
            trace_id: String::new(),
        };

        (self.status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 5xx：HTTP 用真实原因（503），body 不泄漏原始 message。
    #[tokio::test]
    async fn unavailable_maps_to_503_and_hides_message() {
        let err: ApiError = Status::unavailable("connection refused").into();

        assert_eq!(err.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(err.message, "服务暂时不可用");

        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("读取 body 失败");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("解析 JSON 失败");

        assert_eq!(json["code"], 5000, "body: {json}");
        assert_eq!(json["message"], "服务暂时不可用", "body: {json}");
        assert!(json["data"].is_null(), "body: {json}");
        assert!(json["meta"].is_object(), "body: {json}");
    }

    /// 4xx：调用方问题，message 原样返回。
    #[tokio::test]
    async fn invalid_argument_maps_to_400() {
        let err: ApiError = Status::invalid_argument("user_id 不是合法 UUID").into();

        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.message, "user_id 不是合法 UUID");
    }

    /// 超时不能报 500，要让前端 / 监控看到 504。
    #[tokio::test]
    async fn deadline_exceeded_maps_to_504() {
        let err: ApiError = Status::deadline_exceeded("timeout").into();
        assert_eq!(err.status, StatusCode::GATEWAY_TIMEOUT);
    }
}
