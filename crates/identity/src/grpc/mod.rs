use common::code::ApiCode;
// proto 生成的枚举也叫 Role，这里给领域角色起个别名避免撞名
use common::role::Role as DomainRole;
use sea_orm::DatabaseConnection;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::service::credential::session::SessionStore;
use crate::service::credential::session::validate_session;
use crate::service::customer::{LoginInput, RegisterInput, login, logout, me, register};
use crate::service::error::ServiceError;

include!(concat!(env!("OUT_DIR"), "/identity.v1.rs"));

/// proto 的 Role → 领域 Role。`ROLE_UNSPECIFIED` 和未知值一律拒绝。
fn to_domain_role(value: i32) -> Result<DomainRole, Status> {
    match Role::try_from(value) {
        Ok(Role::Customer) => Ok(DomainRole::Customer),
        Ok(Role::Merchant) => Ok(DomainRole::Merchant),
        Ok(Role::Operator) => Ok(DomainRole::Operator),
        _ => Err(Status::invalid_argument("role 不合法")),
    }
}

/// 领域 Role → proto Role 的字段值。
fn to_proto_role(role: DomainRole) -> i32 {
    match role {
        DomainRole::Customer => Role::Customer as i32,
        DomainRole::Merchant => Role::Merchant as i32,
        DomainRole::Operator => Role::Operator as i32,
    }
}

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
        let role = to_domain_role(input.role)?;

        // TODO：商户 / 运营登录还没实现（表与 store 待建）。
        // 在那之前只放行顾客，避免 role 被悄悄忽略。
        if role != DomainRole::Customer {
            return Err(Status::unimplemented("该角色登录尚未实现"));
        }

        let token = login(
            &self.db,
            &self.sessions,
            LoginInput {
                username: input.username,
                password: input.password,
            },
        )
        .await
        .map_err(to_status)?;

        Ok(Response::new(LoginResponse { token }))
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
            Some(i) => (true, i.user_id.to_string(), to_proto_role(i.role)),
            // proto3 的枚举没有 Option，无效时给 ROLE_UNSPECIFIED；调用方应只看 valid
            None => (false, String::new(), Role::Unspecified as i32),
        };

        Ok(Response::new(ValidateSessionResponse {
            valid,
            user_id,
            role,
        }))
    }

    async fn get_account(
        &self,
        request: Request<GetAccountRequest>,
    ) -> Result<Response<GetAccountResponse>, Status> {
        let input = request.into_inner();
        let role = to_domain_role(input.role)?;

        // TODO：商户 / 运营查询还没实现（表与 store 待建）
        if role != DomainRole::Customer {
            return Err(Status::unimplemented("该角色查询尚未实现"));
        }

        // user_id 来自 gateway（它已经调过 ValidateSession），不是 token
        let id = Uuid::parse_str(&input.user_id)
            .map_err(|_| Status::invalid_argument("user_id 不是合法 UUID"))?;

        let model = me(&self.db, id).await.map_err(to_status)?;

        Ok(Response::new(GetAccountResponse {
            id: model.id.to_string(),
            username: model.username,
            email: model.email,
            role: to_proto_role(DomainRole::Customer),
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
