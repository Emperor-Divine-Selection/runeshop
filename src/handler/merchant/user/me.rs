use axum::{Json, http::StatusCode};

use crate::handler::error::ApiError;
use crate::handler::extractor::MerchantUser;
use crate::handler::response::ApiResponse;

use super::MerchantResponse;

pub async fn me_handler(
    MerchantUser { merchant }: MerchantUser,
) -> Result<Json<ApiResponse<MerchantResponse>>, ApiError> {
    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        MerchantResponse::from(merchant),
    )))
}
