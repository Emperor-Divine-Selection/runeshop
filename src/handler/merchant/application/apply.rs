use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::handler::error::ApiError;
use crate::handler::extractor::CustomerUser;
use crate::handler::merchant::application::MerchantApplicationResponse;
use crate::handler::response::ApiResponse;
use crate::service::merchant::application::apply_merchant;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ApplyRequest {
    pub shop_name: String,
}

/// POST /merchant/apply —— 顾客提交商户申请
pub async fn apply_merchant_handler(
    CustomerUser { user_id }: CustomerUser,
    State(state): State<AppState>,
    Json(req): Json<ApplyRequest>,
) -> Result<Json<ApiResponse<MerchantApplicationResponse>>, ApiError> {
    let application = apply_merchant(&state.db, user_id, &req.shop_name).await?;
    Ok(Json(ApiResponse::ok(
        StatusCode::CREATED.as_u16(),
        MerchantApplicationResponse::from(application),
    )))
}
