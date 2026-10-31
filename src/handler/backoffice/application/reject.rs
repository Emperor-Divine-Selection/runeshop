use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sea_orm::sqlx::types::Uuid;

use super::ReviewRequest;
use crate::handler::backoffice::application::ApplicationResponse;
use crate::handler::error::ApiError;
use crate::handler::extractor::BackofficeUser;
use crate::handler::response::ApiResponse;
use crate::service::backoffice::application::reject_application;
use crate::state::AppState;

/// POST /backoffice/applications/{id}/reject —— 审核驳回
pub async fn reject_application_handler(
    BackofficeUser { user }: BackofficeUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ReviewRequest>,
) -> Result<Json<ApiResponse<ApplicationResponse>>, ApiError> {
    let updated = reject_application(&state.db, id, user.id, req.remark).await?;
    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        updated.into(),
    )))
}
