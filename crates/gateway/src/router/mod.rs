pub mod customer;

use crate::state::AppState;
use axum::{Router, routing::get};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .merge(customer::routes(state.clone()))
        .with_state(state)
}
