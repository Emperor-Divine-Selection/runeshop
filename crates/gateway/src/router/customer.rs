use axum::{
    Router, middleware,
    routing::{get, post},
};
use common::role::Role;

use crate::handler::customer;
use crate::middleware::auth::auth_middleware;
use crate::state::GatewayState;

pub fn routes(state: GatewayState) -> Router<GatewayState> {
    Router::new()
        .route("/customer/auth/register", post(customer::register))
        .route("/customer/auth/login", post(customer::login))
        .route("/customer/auth/logout", post(customer::logout))
        .route(
            "/customer/auth/me",
            // route_layer 只作用在这条路由：register / login / logout 不能被拦住
            get(customer::me).route_layer(middleware::from_fn_with_state(
                (state.clone(), Role::Customer),
                auth_middleware,
            )),
        )
}
