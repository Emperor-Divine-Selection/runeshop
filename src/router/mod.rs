mod backoffice;
mod common;
mod customer;
mod merchant;

use crate::state::AppState;
use axum::Router;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn app(state: AppState) -> Router {
    common::routes()
        .merge(customer::routes())
        .merge(merchant::routes())
        .merge(backoffice::routes())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
