use axum::{
    Router, middleware,
    routing::{get, post},
};
use common::role::Role;

use crate::handler::operator;
use crate::middleware::auth::auth_middleware;
use crate::state::GatewayState;

pub fn routes(state: GatewayState) -> Router<GatewayState> {
    Router::new()
        .route("/operator/auth/login", post(operator::login))
        .route("/operator/auth/logout", post(operator::logout))
        .route(
            "/operator/auth/me",
            // route_layer 只作用在这条路由：login / logout 不能被拦住
            get(operator::me).route_layer(middleware::from_fn_with_state(
                (state.clone(), Role::Operator),
                auth_middleware,
            )),
        )
}
