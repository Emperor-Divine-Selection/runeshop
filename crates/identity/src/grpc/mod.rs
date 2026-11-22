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

/// 成功时的业务状态。
fn ok_status() -> BusinessStatus {
    BusinessStatus {
        code: ApiCode::Ok as i32,
        message: "ok".to_owned(),
    }
}

/// 业务错误 → 响应体里的 status。
///
/// 返回 `None` 表示这是技术错误（Internal），调用方应转成 `Err(Status)`，
/// 走传输层而不是业务层。
fn business_status(err: &ServiceError) -> Option<BusinessStatus> {
    match err.code() {
        ApiCode::Internal => None,
        code => Some(BusinessStatus {
            code: code as i32,
            message: err.public_message().to_owned(),
        }),
    }
}

/// 技术错误 → tonic::Status。只用于「服务内部坏了」，业务错误不进这里。
fn to_status(err: ServiceError) -> Status {
    // 原始错误只进日志，不发给客户端 —— 可能含 SQL / 表名
    tracing::error!(error = %err, "identity 技术错误");
    Status::internal("internal error")
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

#[tonic::async_trait]
impl identity_service_server::IdentityService for IdentityGrpc {
    async fn register_customer(
        &self,
        request: Request<RegisterCustomerRequest>,
    ) -> Result<Response<RegisterCustomerResponse>, Status> {
        let input = request.into_inner();

        let result = register(
            &self.db,
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
            Err(err) => match business_status(&err) {
                Some(status) => Ok(Response::new(RegisterCustomerResponse {
                    status: Some(status),
                    ..Default::default()
                })),
                None => Err(to_status(err)),
            },
        }
    }

    async fn login_customer(
        &self,
        request: Request<LoginCustomerRequest>,
    ) -> Result<Response<LoginCustomerResponse>, Status> {
        let input = request.into_inner();

        let result = login(
            &self.db,
            &self.sessions,
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
            Err(err) => match business_status(&err) {
                Some(status) => Ok(Response::new(LoginCustomerResponse {
                    status: Some(status),
                    ..Default::default()
                })),
                None => Err(to_status(err)),
            },
        }
    }

    async fn login_merchant(
        &self,
        _request: Request<LoginMerchantRequest>,
    ) -> Result<Response<LoginMerchantResponse>, Status> {
        // TODO：merchants / shops 表建好后再实现（§11.4）
        Err(Status::unimplemented("商户登录尚未实现"))
    }

    async fn login_operator(
        &self,
        _request: Request<LoginOperatorRequest>,
    ) -> Result<Response<LoginOperatorResponse>, Status> {
        // TODO：operators 表建好后再实现（§11.5）
        Err(Status::unimplemented("运营登录尚未实现"))
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
            status: Some(ok_status()),
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

        match me(&self.db, id).await {
            Ok(model) => Ok(Response::new(GetCustomerResponse {
                status: Some(ok_status()),
                id: model.id.to_string(),
                username: model.username,
                email: model.email,
            })),
            Err(err) => match business_status(&err) {
                Some(status) => Ok(Response::new(GetCustomerResponse {
                    status: Some(status),
                    ..Default::default()
                })),
                None => Err(to_status(err)),
            },
        }
    }

    async fn get_merchant(
        &self,
        _request: Request<GetMerchantRequest>,
    ) -> Result<Response<GetMerchantResponse>, Status> {
        // TODO：merchants 表建好后再实现
        Err(Status::unimplemented("商户资料查询尚未实现"))
    }

    async fn get_operator(
        &self,
        _request: Request<GetOperatorRequest>,
    ) -> Result<Response<GetOperatorResponse>, Status> {
        // TODO：operators 表建好后再实现
        Err(Status::unimplemented("运营资料查询尚未实现"))
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let input = request.into_inner();

        logout(&self.sessions, &input.token)
            .await
            .map_err(to_status)?;

        Ok(Response::new(LogoutResponse {
            status: Some(ok_status()),
        }))
    }
}
