use axum::Router;
use axum::routing::post;
use sea_orm::DatabaseConnection;
use tower_http::trace::TraceLayer;

use crate::handler::register::register_handler;

/// 组装 HTTP 路由；返回已套好中间件的 Router
pub fn app(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/users", post(register_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(db)
}
