use crate::state::AppState;
use axum::Router;
use axum::routing::post;

use crate::handler::customer::user::login::login_handler;
use crate::handler::customer::user::register::register_handler;

/// 组装 HTTP 路由；返回已套好中间件的 Router
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/customer/register", post(register_handler))
        .route("/customer/auth/login", post(login_handler))
}
