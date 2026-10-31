use crate::handler::backoffice::application::{
    approve::approve_application_handler, list::list_applications_handler,
    reject::reject_application_handler,
};
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
        .route("/backoffice/applications", get(list_applications_handler))
        .route(
            "/backoffice/applications/{id}/approve",
            post(approve_application_handler),
        )
        .route(
            "/backoffice/applications/{id}/reject",
            post(reject_application_handler),
        )
}
