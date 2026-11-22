use axum::{
    Router, middleware,
    routing::{get, post},
};
use common::role::Role;

use crate::handler::merchant;
use crate::middleware::auth::auth_middleware;
use crate::state::GatewayState;

pub fn routes(state: GatewayState) -> Router<GatewayState> {
    Router::new()
        .route("/merchant/auth/login", post(merchant::login))
        .route("/merchant/auth/logout", post(merchant::logout))
        .route(
            "/merchant/auth/me",
            // route_layer 只作用在这条路由：login / logout 不能被拦住
            get(merchant::me).route_layer(middleware::from_fn_with_state(
                (state.clone(), Role::Merchant),
                auth_middleware,
            )),
        )
}
