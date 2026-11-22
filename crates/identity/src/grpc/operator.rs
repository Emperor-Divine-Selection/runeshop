use tonic::{Request, Response, Status};

use super::{
    GetOperatorRequest, GetOperatorResponse, IdentityState, LoginOperatorRequest,
    LoginOperatorResponse,
};

pub(super) async fn login_operator(
    _grpc: &IdentityState,
    _request: Request<LoginOperatorRequest>,
) -> Result<Response<LoginOperatorResponse>, Status> {
    // TODO：operators / roles 表建好后再实现（§11.5）
    Err(Status::unimplemented("运营登录尚未实现"))
}

pub(super) async fn get_operator(
    _grpc: &IdentityState,
    _request: Request<GetOperatorRequest>,
) -> Result<Response<GetOperatorResponse>, Status> {
    // TODO：operators / roles 表建好后再实现
    Err(Status::unimplemented("运营资料查询尚未实现"))
}
