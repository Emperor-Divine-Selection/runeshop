use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::service::LoginInput;
use crate::service::customer::{RegisterInput, login, me, register};

use super::{
    GetCustomerRequest, GetCustomerResponse, IdentityState, LoginCustomerRequest,
    LoginCustomerResponse, RegisterCustomerRequest, RegisterCustomerResponse, ok_status,
    to_business_status, to_transport_status,
};

pub async fn register_customer(
    grpc: &IdentityState,
    request: Request<RegisterCustomerRequest>,
) -> Result<Response<RegisterCustomerResponse>, Status> {
    let input = request.into_inner();

    let result = register(
        &grpc.db,
        RegisterInput {
            username: input.username,
            email: input.email,
            password: input.password,
        },
    )
    .await;

    match result {
        Ok(model) => Ok(Response::new(RegisterCustomerResponse {
            status: Some(ok_status()),
            id: model.id.to_string(),
            username: model.username,
            email: model.email,
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(RegisterCustomerResponse {
                status: Some(status),
                ..Default::default()
            })),
            None => Err(to_transport_status(err)),
        },
    }
}

pub async fn login_customer(
    grpc: &IdentityState,
    request: Request<LoginCustomerRequest>,
) -> Result<Response<LoginCustomerResponse>, Status> {
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
        Ok(token) => Ok(Response::new(LoginCustomerResponse {
            status: Some(ok_status()),
            token,
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(LoginCustomerResponse {
                status: Some(status),
                ..Default::default()
            })),
            None => Err(to_transport_status(err)),
        },
    }
}

pub async fn get_customer(
    grpc: &IdentityState,
    request: Request<GetCustomerRequest>,
) -> Result<Response<GetCustomerResponse>, Status> {
    let input = request.into_inner();

    // user_id 来自 gateway（它已经调过 ValidateSession），不是 token
    let id = Uuid::parse_str(&input.user_id)
        .map_err(|_| Status::invalid_argument("user_id 不是合法 UUID"))?;

    match me(&grpc.db, id).await {
        Ok(model) => Ok(Response::new(GetCustomerResponse {
            status: Some(ok_status()),
            id: model.id.to_string(),
            username: model.username,
            email: model.email,
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(GetCustomerResponse {
                status: Some(status),
                ..Default::default()
            })),
            None => Err(to_transport_status(err)),
        },
    }
}
