use axum::Router;
use axum::routing::{get, post};

use crate::handler::customer::application::apply::apply_merchant_handler;
use crate::handler::merchant::product::create::create_product_handler;
use crate::handler::merchant::auth::login::merchant_login_handler;
use crate::handler::merchant::profile::me::me_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/merchant/apply", post(apply_merchant_handler))
        .route("/merchant/auth/login", post(merchant_login_handler))
        .route("/merchant/me", get(me_handler))
        .route("/merchant/products", post(create_product_handler))
}
