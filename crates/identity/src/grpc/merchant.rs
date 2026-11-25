use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::service::LoginInput;
use crate::service::merchant::{login, me_merchant};

use super::{
    GetMerchantRequest, GetMerchantResponse, IdentityState, LoginMerchantRequest,
    LoginMerchantResponse, ok_status, to_business_status, to_transport_status,
};

pub async fn login_merchant(
    grpc: &IdentityState,
    request: Request<LoginMerchantRequest>,
) -> Result<Response<LoginMerchantResponse>, Status> {
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
        Ok(token) => Ok(Response::new(LoginMerchantResponse {
            status: Some(ok_status()),
            token,
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(LoginMerchantResponse {
                status: Some(status),
                ..Default::default()
            })),
            None => Err(to_transport_status(err)),
        },
    }
}

pub async fn get_merchant(
    grpc: &IdentityState,
    request: Request<GetMerchantRequest>,
) -> Result<Response<GetMerchantResponse>, Status> {
    let input = request.into_inner();

    let id = Uuid::parse_str(&input.user_id)
        .map_err(|_| Status::invalid_argument("user_id 不是合法 UUID"))?;

    let result = me_merchant(&grpc.db, id).await;

    match result {
        Ok(profile) => Ok(Response::new(GetMerchantResponse {
            status: Some(ok_status()),
            id: profile.id.to_string(),
            username: profile.username,
            email: profile.email,
            shop_id: profile.shop_id.to_string(),
            shop_name: profile.shop_name,
            permissions: profile.permissions,
        })),
        Err(err) => match to_business_status(&err) {
            Some(status) => Ok(Response::new(GetMerchantResponse {
                status: Some(status),
                ..Default::default()
            })),
            None => Err(to_transport_status(err)),
        },
    }
}
