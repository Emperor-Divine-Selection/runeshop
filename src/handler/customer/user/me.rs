use axum::{Json, extract::State, http::StatusCode};

use crate::handler::error::ApiError;
use crate::handler::extractor::CustomerUser;
use crate::handler::response::ApiResponse;
use crate::service::customer::user::profile;
use crate::state::AppState;

use super::UserResponse;

/// GET /customer/me —— 取当前登录用户
/// 提取器 CustomerUser 失败（没带 token / token 失效 / 不是顾客端）→ 自动 401
pub async fn me_handler(
    CustomerUser { user_id }: CustomerUser,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<UserResponse>>, ApiError> {
    let user = profile(&state.db, &user_id).await?;
    Ok(Json(ApiResponse::ok(
        StatusCode::OK.as_u16(),
        UserResponse::from(user), // 也可以写 user.into()，同一个 From
    )))
}
