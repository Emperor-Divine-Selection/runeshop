use axum::{Router, routing::post};

use crate::handler::customer;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(customer::register))
        .route("/auth/login", post(customer::login))
}
