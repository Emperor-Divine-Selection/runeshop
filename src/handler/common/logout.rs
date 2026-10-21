use axum::{Json, extract::State, http::HeaderMap, http::StatusCode};

use crate::handler::extractor::bearer_token;
use crate::handler::response::ApiResponse;
use crate::service::common::logout::logout;
use crate::state::AppState;

use crate::handler::error::ApiError;

pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let token = bearer_token(&headers)?;
    logout(&state.cache, token).await?;
    Ok(Json(ApiResponse::ok(StatusCode::OK.as_u16(), ())))
}
