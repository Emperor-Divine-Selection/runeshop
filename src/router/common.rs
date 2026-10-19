use axum::Router;
use axum::routing::post;

use crate::handler::common::logout::logout_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/auth/logout", post(logout_handler))
}
