use axum::{
    Router,
    routing::{get, post},
};

use crate::handler;
use crate::state::AppState;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/auth/register", post(handler::customer::register))
        .with_state(state)
}
