use axum::{Json, extract::State, http::StatusCode};

use crate::handler::error::ApiError;
use crate::handler::extractor::CustomerUser;
use crate::handler::response::ApiResponse;
use crate::service::customer::application::find_own_application;
use crate::state::AppState;

use super::CustomerApplicationResponse;

/// GET /customer/application —— 查自己提交的商户申请
///
/// 没提交过返 `data: null`（不是 404）—— 前端用「有没有数据」决定显示表单还是状态
pub async fn get_application_handler(
    CustomerUser { user_id }: CustomerUser,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Option<CustomerApplicationResponse>>>, ApiError> {
    let application = find_own_application(&state.db, user_id).await?;
    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        application.map(CustomerApplicationResponse::from),
    )))
}
