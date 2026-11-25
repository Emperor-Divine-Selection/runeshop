use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::service::LoginInput;
use crate::service::operator::{login, me_operator};

use super::{
    GetOperatorRequest, GetOperatorResponse, IdentityState, LoginOperatorRequest,
    LoginOperatorResponse, ok_status, to_business_status, to_transport_status,
};

pub async fn login_operator(
    grpc: &IdentityState,
    request: Request<LoginOperatorRequest>,
) -> Result<Response<LoginOperatorResponse>, Status> {
    let input = request.into_inner();

    let result = login(
        &grpc.db,
        &grpc.sessions,
        LoginInput {
            username: input.username,
            password: input.password,
        },
    )
    .await;

    match result {
        Ok(token) => Ok(Response::new(LoginOperatorResponse {
            status: Some(ok_status()),
            token,
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(LoginOperatorResponse {
                status: Some(status),
                ..Default::default()
            })),
            None => Err(to_transport_status(err)),
        },
    }
}

pub async fn get_operator(
    grpc: &IdentityState,
    request: Request<GetOperatorRequest>,
) -> Result<Response<GetOperatorResponse>, Status> {
    let input = request.into_inner();

    let id = Uuid::parse_str(&input.user_id)
        .map_err(|_| Status::invalid_argument("user_id 不是合法 UUID"))?;

    let result = me_operator(&grpc.db, id).await;

    match result {
        Ok(profile) => Ok(Response::new(GetOperatorResponse {
            status: Some(ok_status()),
            id: profile.id.to_string(),
            username: profile.username,
            email: profile.email,
            permissions: profile.permissions,
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(GetOperatorResponse {
                status: Some(status),
                ..Default::default()
            })),
            None => Err(to_transport_status(err)),
        },
    }
}
