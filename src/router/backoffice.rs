use crate::handler::backoffice::user::{
    login::login_handler, me::me_handler, register::register_handler,
};
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/backoffice/register", post(register_handler))
        .route("/backoffice/auth/login", post(login_handler))
        .route("/backoffice/me", get(me_handler))
}
