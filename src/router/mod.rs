use crate::state::AppState;
use axum::Router;
use axum::routing::post;
use tower_http::trace::TraceLayer;

use crate::handler::register::register_handler;

/// 组装 HTTP 路由；返回已套好中间件的 Router
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
