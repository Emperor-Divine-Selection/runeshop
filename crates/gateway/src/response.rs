use axum::{
    Json,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use common::response::ApiResponse;
use serde::Serialize;

/// 成功响应的 HTTP 包装。
///
/// `common::ApiResponse` 只描述「信封长什么样」；把它变成 HTTP 响应是 gateway 的事。
/// 不能把 `IntoResponse` 写在 common —— 那会让 identity / catalog / order 都被迫依赖 axum
/// （§2：只有 gateway 说 HTTP）。这里用 newtype 绕过孤儿规则。
///
/// 响应头也由它携带，所以所有 handler 的签名都是 `Result<Envelope<T>, ApiError>`：
/// 挂 Cookie / trace_id 只是函数体里多一句 `.header(...)`，不会改变签名。
pub struct Envelope<T> {
    body: ApiResponse<T>,
    status: StatusCode,
    headers: HeaderMap,
}

impl<T: Serialize> Envelope<T> {
    pub fn ok(data: T) -> Self {
        Self::with_status(StatusCode::OK, data)
    }

    /// 注册成功用 201（§7），而不是 200。
    pub fn created(data: T) -> Self {
        Self::with_status(StatusCode::CREATED, data)
    }

    fn with_status(status: StatusCode, data: T) -> Self {
        Self {
            body: ApiResponse::ok(data),
            status,
            headers: HeaderMap::new(),
        }
    }

    /// 追加一个响应头。
    ///
    /// 用 `append` 而不是 `insert`：`Set-Cookie` 允许出现多次，
    /// `insert` 会把前一条覆盖掉。
    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.append(name, value);
        self
    }
}

impl<T: Serialize> IntoResponse for Envelope<T> {
    fn into_response(self) -> Response {
        let mut response = (self.status, Json(self.body)).into_response();

        // HeaderMap 的 IntoIter 会把同名多值编成 (Some(name), v1) / (None, v2)，
        // 对应的 Extend 实现是追加语义 —— 多个 Set-Cookie 不会互相覆盖。
        response.headers_mut().extend(self.headers);

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header::SET_COOKIE;

    #[tokio::test]
    async fn cookie_header_is_attached_to_response() {
        let cookie = HeaderValue::from_static("session=abc; HttpOnly; Path=/");

        let response = Envelope::ok(()).header(SET_COOKIE, cookie).into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(SET_COOKIE).unwrap(),
            "session=abc; HttpOnly; Path=/"
        );
    }

    #[tokio::test]
    async fn multiple_set_cookie_headers_are_kept() {
        let response = Envelope::ok(())
            .header(SET_COOKIE, HeaderValue::from_static("a=1"))
            .header(SET_COOKIE, HeaderValue::from_static("b=2"))
            .into_response();

        let values: Vec<_> = response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .map(|value| value.to_str().unwrap())
            .collect();

        assert_eq!(values, ["a=1", "b=2"]);
    }

    #[tokio::test]
    async fn unit_data_serializes_to_null() {
        let response = Envelope::ok(()).into_response();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["data"].is_null(), "body: {json}");
        assert_eq!(json["code"], 0, "body: {json}");
    }
}
