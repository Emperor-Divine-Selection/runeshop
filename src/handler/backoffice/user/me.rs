use axum::{Json, http::StatusCode};

use crate::handler::error::ApiError;
use crate::handler::extractor::BackofficeUser;
use crate::handler::response::ApiResponse;

use super::BackofficeUserResponse;

pub async fn me_handler(
    BackofficeUser { user }: BackofficeUser,
) -> Result<Json<ApiResponse<BackofficeUserResponse>>, ApiError> {
    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        BackofficeUserResponse::from(user),
    )))
}
