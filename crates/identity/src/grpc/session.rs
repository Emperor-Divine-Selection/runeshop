use tonic::{Request, Response, Status};

use crate::service::credential::session::validate_session as load_session_identity;
use crate::service::customer::logout as delete_session;

use super::{
    IdentityState, LogoutRequest, LogoutResponse, Role, ValidateSessionRequest,
    ValidateSessionResponse, ok_status, to_proto_role, to_transport_status,
};

pub(super) async fn validate_session(
    grpc: &IdentityState,
    request: Request<ValidateSessionRequest>,
) -> Result<Response<ValidateSessionResponse>, Status> {
    let input = request.into_inner();

    // 只读 Valkey，查一次库都不做（§4.4：session 是凭证缓存，不是资料缓存）
    let ident = load_session_identity(&grpc.sessions, &input.token)
        .await
        .map_err(to_transport_status)?;

    let (valid, user_id, role) = match ident {
        Some(i) => (true, i.user_id.to_string(), to_proto_role(i.role)),
        // proto3 的枚举没有 Option，无效时给 ROLE_UNSPECIFIED；调用方应只看 valid
        None => (false, String::new(), Role::Unspecified as i32),
    };

    Ok(Response::new(ValidateSessionResponse {
        status: Some(ok_status()),
        valid,
        user_id,
        role,
    }))
}

pub(super) async fn logout(
    grpc: &IdentityState,
    request: Request<LogoutRequest>,
) -> Result<Response<LogoutResponse>, Status> {
    let input = request.into_inner();

    delete_session(&grpc.sessions, &input.token)
        .await
        .map_err(to_transport_status)?;

    Ok(Response::new(LogoutResponse {
        status: Some(ok_status()),
    }))
}
