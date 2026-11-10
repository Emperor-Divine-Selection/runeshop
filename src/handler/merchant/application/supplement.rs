use axum::{Json, extract::State, http::StatusCode};

use super::{ApplicantApplicationResponse, SupplementRequest};

use crate::handler::error::ApiError;
use crate::handler::extractor::ApplicantUser;
use crate::handler::response::ApiResponse;
use crate::service::merchant::application::{SupplementApplication, supplement_application};
use crate::state::AppState;

/// PATCH /merchant/application/me —— 补正资料（仅当状态是 supplement_required）
pub async fn supplement_handler(
    ApplicantUser { application_id }: ApplicantUser,
    State(state): State<AppState>,
    Json(req): Json<SupplementRequest>,
) -> Result<Json<ApiResponse<ApplicantApplicationResponse>>, ApiError> {
    let updated = supplement_application(
        &state.db,
        application_id,
        SupplementApplication {
            shop_name: req.shop_name,
            email: req.email,
            password: req.password,
        },
    )
    .await?;

    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        ApplicantApplicationResponse::from(updated),
    )))
}
