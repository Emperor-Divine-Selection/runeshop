use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sea_orm::sqlx::types::Uuid;

use super::ReviewRequest;
use crate::handler::backoffice::application::ApplicationResponse;
use crate::handler::error::ApiError;
use crate::handler::extractor::BackofficeUser;
use crate::handler::response::ApiResponse;
use crate::service::backoffice::application::request_supplement;
use crate::state::AppState;

/// POST /backoffice/applications/{id}/supplement —— 要求补正
pub async fn request_supplement_handler(
    BackofficeUser { user }: BackofficeUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ReviewRequest>,
) -> Result<Json<ApiResponse<ApplicationResponse>>, ApiError> {
    let updated = request_supplement(&state.db, id, user.id, req.remark).await?;
    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        updated.into(),
    )))
}
