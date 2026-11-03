use axum::http::{HeaderMap, HeaderValue, header};
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

/// cookie 名，和 extractor 里读的必须一致
pub const TOKEN_COOKIE: &str = "token";

/// 和 cache::token 的 DEFAULT_TTL 对齐（24h）
const TOKEN_MAX_AGE: u64 = 24 * 60 * 60;

/// 登录成功后要带的 Set-Cookie
pub fn token_cookie(token: &str) -> HeaderMap {
    set_cookie(format!(
        "{TOKEN_COOKIE}={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={TOKEN_MAX_AGE}"
    ))
}

/// 登出：Max-Age=0 让浏览器立刻删
pub fn clear_token_cookie() -> HeaderMap {
    set_cookie(format!(
        "{TOKEN_COOKIE}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0"
    ))
}

fn set_cookie(value: String) -> HeaderMap {
    let mut headers = HeaderMap::new();
    // token 是 64 位 hex，必然合法；转不了说明内存出问题，静默跳过比 panic 好
    if let Ok(value) = HeaderValue::from_str(&value) {
        headers.insert(header::SET_COOKIE, value);
    }
    headers
}

/// 分页信封：列表接口统一返回 items + 总条数 + 当前页信息
#[derive(Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}
