use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handler::customer;
use crate::middleware::auth::auth_middleware;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/customer/auth/register", post(customer::register))
        .route("/customer/auth/login", post(customer::login))
        .route("/customer/auth/logout", post(customer::logout))
        .route(
            "/customer/auth/me",
            // route_layer 只作用在这条路由：register / login / logout 不能被拦住
            get(customer::me).route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            )),
        )
}
