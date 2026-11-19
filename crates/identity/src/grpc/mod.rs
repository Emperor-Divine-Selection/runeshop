use common::code::ApiCode;
use common::role::Role;
use sea_orm::DatabaseConnection;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::service::credential::session::SessionStore;
use crate::service::customer::{LoginInput, RegisterInput, login, logout, me, register};
use crate::service::error::ServiceError;
use crate::service::credential::session::validate_session;

include!(concat!(env!("OUT_DIR"), "/identity.v1.rs"));

/// gRPC 层：只做协议翻译，不写业务逻辑。
pub struct IdentityGrpc {
    pub db: DatabaseConnection,
    pub sessions: SessionStore,
}

/// ServiceError → tonic::Status 的唯一映射点。
fn to_status(err: impl Into<ServiceError>) -> Status {
    let err = err.into();
    let public = err.public_message();
    match err.code() {
        ApiCode::InvalidCredentials | ApiCode::Unauthorized => Status::unauthenticated(public),
        ApiCode::InvalidParams => Status::invalid_argument(public),
        ApiCode::Conflict => Status::already_exists(public),
        // 内部错误绝不把细节发给客户端 —— 可能含 SQL / 表名
        ApiCode::Internal => Status::internal("internal error"),
        ApiCode::Ok => unreachable!("成功不会被当错误处理"),
    }
}

#[tonic::async_trait]
impl identity_service_server::IdentityService for IdentityGrpc {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let input = request.into_inner();

        let model = register(
            &self.db,
            RegisterInput {
                username: input.username,
                email: input.email,
                password: input.password,
            },
        )
        .await
        .map_err(to_status)?;

        Ok(Response::new(RegisterResponse {
            id: model.id.to_string(),
            username: model.username,
            email: model.email,
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let input = request.into_inner();

        let (token, model) = login(
            &self.db,
            &self.sessions,
            LoginInput {
                username: input.username,
                password: input.password,
            },
        )
        .await
        .map_err(to_status)?;

        Ok(Response::new(LoginResponse {
            token,
            user_id: model.id.to_string(),
            role: Role::Customer.as_str().to_string(),
        }))
    }

    async fn validate_session(
        &self,
        request: Request<ValidateSessionRequest>,
    ) -> Result<Response<ValidateSessionResponse>, Status> {
        let input = request.into_inner();

        // 只读 Valkey，查一次库都不做（§4.4：session 是凭证缓存，不是资料缓存）
        let ident = validate_session(&self.sessions, &input.token)
            .await
            .map_err(to_status)?;

        let (valid, user_id, role) = match ident {
            Some(i) => (true, i.user_id.to_string(), i.role.as_str().to_owned()),
            // proto3 的 string 没有 Option，无效时用空串；调用方应只看 valid
            None => (false, String::new(), String::new()),
        };

        Ok(Response::new(ValidateSessionResponse {
            valid,
            user_id,
            role,
        }))
    }

    async fn get_customer(
        &self,
        request: Request<GetCustomerRequest>,
    ) -> Result<Response<GetCustomerResponse>, Status> {
        let input = request.into_inner();

        // user_id 来自 gateway（它已经调过 ValidateSession），不是 token
        let id = Uuid::parse_str(&input.user_id)
            .map_err(|_| Status::invalid_argument("user_id 不是合法 UUID"))?;

        let model = me(&self.db, id).await.map_err(to_status)?;

        Ok(Response::new(GetCustomerResponse {
            id: model.id.to_string(),
            username: model.username,
            email: model.email,
            role: Role::Customer.as_str().to_owned(),
        }))
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let input = request.into_inner();

        let _ = logout(&self.sessions, &input.token)
            .await
            .map_err(to_status)?;

        Ok(Response::new(LogoutResponse {}))
    }
}
