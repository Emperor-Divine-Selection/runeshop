use crate::state::AppState;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};

use crate::handler::customer::application::get::get_application_handler;
use crate::handler::customer::profile::avatar::update_avatar_handler;
use crate::handler::customer::auth::login::login_handler;
use crate::handler::customer::profile::me::me_handler;
use crate::handler::customer::auth::register::register_handler;

/// 头像请求体上限
///
/// 故意比 service 层的 MAX_AVATAR_BYTES（5MB）大一点：
/// multipart 的边界和头部本身也占字节，如果两个值相等，
/// 正好 5MB 的图片会被 HTTP 层直接掐掉，用户看不到 service 那句友好的提示。
const AVATAR_BODY_LIMIT: usize = 6 * 1024 * 1024;

/// 组装 HTTP 路由；返回已套好中间件的 Router
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/customer/register", post(register_handler))
        .route("/customer/auth/login", post(login_handler))
        .route("/customer/me", get(me_handler))
        .route("/customer/application", get(get_application_handler))
        .route(
            "/customer/avatar",
            post(update_avatar_handler).layer(DefaultBodyLimit::max(AVATAR_BODY_LIMIT)),
        )
}
