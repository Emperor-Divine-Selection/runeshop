use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use common::code::ApiCode;
use common::response::{ApiResponse, Meta};
use tonic::Status;

pub struct ApiError {
    pub code: ApiCode,
    pub message: String,
}

impl From<Status> for ApiError {
    fn from(value: Status) -> Self {
        match value.code() {
            tonic::Code::InvalidArgument => Self {
                code: ApiCode::InvalidParams,
                message: value.message().to_owned(),
            },
            tonic::Code::Unauthenticated => Self {
                code: ApiCode::Unauthorized,
                message: value.message().to_owned(),
            },
            tonic::Code::AlreadyExists => Self {
                code: ApiCode::Conflict,
                message: value.message().to_owned(),
            },

            _ => Self {
                code: ApiCode::Internal,
                message: "服务暂时不可用".to_owned(),
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code {
            ApiCode::Ok => StatusCode::OK,
            ApiCode::InvalidCredentials | ApiCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiCode::InvalidParams => StatusCode::BAD_REQUEST,
            ApiCode::Conflict => StatusCode::CONFLICT,
            ApiCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = ApiResponse::<()> {
            // data 失败恒 null，用 () 占位
            code: self.code as i32,
            message: self.message,
            data: None,
            meta: Meta::default(),
            trace_id: String::new(),
        };

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 踩坑 75 的纪律：HTTP status 和 body 的 code 由同一个字段驱动，
    /// 防的是「改了 status 却忘改 body」的漂移。
    #[tokio::test]
    async fn conflict_maps_to_409_and_1004() {
        let resp = ApiError {
            code: ApiCode::Conflict,
            message: "用户名已被占用".to_owned(),
        }
        .into_response();

        assert_eq!(resp.status(), StatusCode::CONFLICT);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], 1004, "body: {json}");
        assert_eq!(json["message"], "用户名已被占用", "body: {json}");
        assert!(json["data"].is_null(), "失败时 data 恒为 null，body: {json}");
        assert!(json["meta"].is_object(), "meta 恒存在，body: {json}");
    }

    /// Internal 不把 gRPC 的 message 带给客户端 —— 里面可能有 SQL / 表名。
    #[test]
    fn internal_status_hides_original_message() {
        let err: ApiError = Status::internal("db error: connection refused").into();

        assert_eq!(err.code, ApiCode::Internal);
        assert_eq!(err.message, "服务暂时不可用");
    }

    /// Unauthenticated → 401 + 1002（登录失败与未登录在这一层合并，见 error.rs 顶部注释）。
    #[tokio::test]
    async fn unauthenticated_maps_to_401_and_1002() {
        let err: ApiError =
            Status::unauthenticated("[InvalidCredentials] 用户名或密码错误").into();
        let resp = err.into_response();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], 1002, "body: {json}");
    }
}
