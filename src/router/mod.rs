mod backoffice;
mod common;
mod customer;
mod merchant;

use crate::state::AppState;
use axum::Router;
use tower_http::trace::TraceLayer;

pub fn app(state: AppState) -> Router {
    common::routes()
        .merge(customer::routes())
        .merge(merchant::routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
