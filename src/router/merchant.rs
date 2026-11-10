use axum::Router;
use axum::routing::{get, post};

use crate::handler::merchant::application::apply::apply_merchant_handler;
use crate::handler::merchant::application::me::me_handler as application_me_handler;
use crate::handler::merchant::application::supplement::supplement_handler;
use crate::handler::merchant::auth::login::merchant_login_handler;
use crate::handler::merchant::product::create::create_product_handler;
use crate::handler::merchant::profile::me::me_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        // 公开：提交开店申请（不需要登录）
        .route("/merchant/apply", post(apply_merchant_handler))
        // 公开：商户 / 申请人共用登录口
        .route("/merchant/auth/login", post(merchant_login_handler))
        // 申请人：看状态 / 补正资料
        .route(
            "/merchant/application/me",
            get(application_me_handler).patch(supplement_handler),
        )
        // 商户
        .route("/merchant/me", get(me_handler))
        .route("/merchant/products", post(create_product_handler))
}
