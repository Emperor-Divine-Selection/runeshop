use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, grpc::RegisterRequest, response::ApiOk, state::AppState};

#[derive(Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct RegisterReply {
    pub id: String,
    pub username: String,
    pub email: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<ApiOk<RegisterReply>, ApiError> {
    let mut client = state.identity.clone();

    let reply = client
        .register(RegisterRequest {
            username: body.username,
            password: body.password,
            email: body.email,
        })
        .await
        .map_err(ApiError::from)?
        .into_inner();

    Ok(ApiOk::created(RegisterReply {
        id: reply.id,
        username: reply.username,
        email: reply.email,
    }))
}
