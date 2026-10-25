use axum::Router;
use axum::routing::{get, post};

use crate::handler::merchant::application::apply::apply_merchant_handler;
use crate::handler::merchant::user::login::merchant_login_handler;
use crate::handler::merchant::user::me::me_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/merchant/apply", post(apply_merchant_handler))
        .route("/merchant/auth/login", post(merchant_login_handler))
        .route("/merchant/me", get(me_handler))
}
