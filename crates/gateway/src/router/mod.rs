pub mod auth;

use crate::state::AppState;
use axum::{Router, routing::get};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .merge(auth::routes(state.clone()))
        .with_state(state)
}
