use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sea_orm::sqlx::types::Uuid;

use crate::handler::backoffice::application::ApplicationResponse;
use crate::handler::error::ApiError;
use crate::handler::extractor::BackofficeUser;
use crate::handler::response::ApiResponse;
use crate::service::backoffice::application::approve_application;
use crate::state::AppState;

/// POST /backoffice/applications/{id}/approve —— 审核通过（同时建商户）
pub async fn approve_application_handler(
    BackofficeUser { user }: BackofficeUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ApplicationResponse>>, ApiError> {
    let updated = approve_application(&state.db, id, user.id).await?;
    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        updated.into(),
    )))
}
