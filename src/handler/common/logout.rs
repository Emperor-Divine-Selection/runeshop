use axum::{Json, extract::State, http::HeaderMap, http::StatusCode};

use crate::handler::response::ApiResponse;
use crate::service::common::logout;
use crate::service::error::ServiceError;
use crate::state::AppState;

use crate::handler::error::ApiError;

pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let Some(token) = headers.get("token") else {
        return Err(ApiError::Service(ServiceError::Business(
            "请求头未包含 token".into(),
        )));
    };

    let Ok(token) = token.to_str() else {
        // to_str() 返回 Result<&str, ToStrError>
        return Err(ApiError::Service(ServiceError::InvalidParams(
            "token 参数错误".into(),
        )));
    };

    logout(&state.cache, token).await?;
    Ok(Json(ApiResponse::ok(StatusCode::OK.as_u16(), ())))
}
