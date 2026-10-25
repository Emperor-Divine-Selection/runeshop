// src/handler/backoffice/user/register.rs
use crate::handler::backoffice::user::BackofficeUserResponse;
use crate::handler::{error::ApiError, response::ApiResponse};
use crate::service::backoffice::user::backoffice_register;
use crate::state::AppState;
use crate::store::backoffice_user::NewBackofficeUser;
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
}

pub async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<BackofficeUserResponse>>, ApiError> {
    let model = backoffice_register(
        &state.db,
        NewBackofficeUser {
            username: req.username,
            password: req.password,
            email: req.email,
        },
    )
    .await?;
    Ok(Json(ApiResponse::ok(
        StatusCode::CREATED.as_u16(),
        model.into(),
    )))
}
