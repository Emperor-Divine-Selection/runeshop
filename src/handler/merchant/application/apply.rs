use axum::{Json, extract::State, http::StatusCode};

use super::{ApplicantApplicationResponse, ApplyRequest};

use crate::handler::error::ApiError;
use crate::handler::response::ApiResponse;
use crate::service::merchant::application::{ApplyMerchant, apply_merchant};
use crate::state::AppState;

/// POST /merchant/apply —— 提交商户申请（**公开，不需要登录**）
pub async fn apply_merchant_handler(
    State(state): State<AppState>,
    Json(req): Json<ApplyRequest>,
) -> Result<Json<ApiResponse<ApplicantApplicationResponse>>, ApiError> {
    let application = apply_merchant(
        &state.db,
        ApplyMerchant {
            username: req.username,
            password: req.password,
            email: req.email,
            shop_name: req.shop_name,
        },
    )
    .await?;

    Ok(Json(ApiResponse::ok(
        StatusCode::CREATED.as_u16(),
        ApplicantApplicationResponse::from(application),
    )))
}
