pub mod customer;
pub mod merchant;
pub mod operator;

use crate::state::GatewayState;
use axum::{Router, routing::get};

pub fn app(state: GatewayState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .merge(customer::routes(state.clone()))
        .merge(merchant::routes(state.clone()))
        .merge(operator::routes(state.clone()))
        .with_state(state)
}
