use axum::Router;
use axum::routing::post;

use crate::handler::merchant::user::login::merchant_login_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/merchant/auth/login", post(merchant_login_handler))
}
