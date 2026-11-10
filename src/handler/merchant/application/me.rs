use axum::{Json, http::StatusCode};

use super::ApplicantApplicationResponse;

use crate::handler::error::ApiError;
use crate::handler::extractor::ApplicantUser;
use crate::handler::response::ApiResponse;
use crate::service::merchant::application::find_own_application;
use crate::state::AppState;

/// GET /merchant/application/me —— 看自己的申请状态 + 审核意见
pub async fn me_handler(
    ApplicantUser { application_id }: ApplicantUser,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<ApiResponse<ApplicantApplicationResponse>>, ApiError> {
    let Some(application) = find_own_application(&state.db, application_id).await? else {
        return Err(ApiError::Service(crate::service::error::ServiceError::NotFound(
            "申请单不存在".into(),
        )));
    };

    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        ApplicantApplicationResponse::from(application),
    )))
}
