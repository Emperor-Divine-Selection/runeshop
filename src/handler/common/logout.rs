use axum::{Json, extract::State, http::HeaderMap, http::StatusCode};

use crate::handler::error::ApiError;
use crate::handler::extractor::extract_token;
use crate::handler::response::{ApiResponse, clear_token_cookie};
use crate::service::common::logout::logout;
use crate::state::AppState;

pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<ApiResponse<()>>), ApiError> {
    // 登出必须幂等：没 token / token 已过期，也要把 cookie 清掉，
    // 不能返 401 把用户卡在「点了登出却还在登录态」的页面上
    if let Ok(token) = extract_token(&headers) {
        if let Err(error) = logout(&state.cache, token).await {
            // Valkey 出错不能让 cookie 清不掉，但必须留日志 ——
            // 否则那个 token 会在服务端活满 24 小时，没人知道
            tracing::warn!(%error, "登出时删除 token 失败");
        }
    }

    Ok((
        clear_token_cookie(),
        Json(ApiResponse::ok(StatusCode::OK.as_u16(), ())),
    ))
}
