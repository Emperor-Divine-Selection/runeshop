use common::code::ApiCode;
// proto 生成的枚举也叫 Role，这里给领域角色起个别名避免撞名
use common::role::Role as DomainRole;
use sea_orm::DatabaseConnection;
use tonic::{Request, Response, Status};

use crate::service::credential::session::SessionStore;
use crate::service::error::ServiceError;

mod customer;
mod merchant;
mod operator;
mod session;

include!(concat!(env!("OUT_DIR"), "/identity.v1.rs"));

/// identity 的共享状态：gRPC 处理器需要的依赖。
///
/// `db` 是 PG 连接池，`sessions` 是 Valkey 连接 —— 两者内部都是 Arc，
/// clone 只加引用计数，不新建连接。
pub struct IdentityState {
    pub db: DatabaseConnection,
    pub sessions: SessionStore,
}

/// 这个 impl 块只是**路由表**：每个方法一行，转发到对应角色的子模块。
///
/// trait 实现不能拆成多个 impl 块（E0119），所以逻辑放子模块，
/// 这里只保留「哪个 rpc 归哪个文件」。
#[tonic::async_trait]
impl identity_service_server::IdentityService for IdentityState {
    async fn register_customer(
        &self,
        request: Request<RegisterCustomerRequest>,
    ) -> Result<Response<RegisterCustomerResponse>, Status> {
        customer::register_customer(self, request).await
    }

    async fn login_customer(
        &self,
        request: Request<LoginCustomerRequest>,
    ) -> Result<Response<LoginCustomerResponse>, Status> {
        customer::login_customer(self, request).await
    }

    async fn login_merchant(
        &self,
        request: Request<LoginMerchantRequest>,
    ) -> Result<Response<LoginMerchantResponse>, Status> {
        merchant::login_merchant(self, request).await
    }

    async fn login_operator(
        &self,
        request: Request<LoginOperatorRequest>,
    ) -> Result<Response<LoginOperatorResponse>, Status> {
        operator::login_operator(self, request).await
    }

    async fn validate_session(
        &self,
        request: Request<ValidateSessionRequest>,
    ) -> Result<Response<ValidateSessionResponse>, Status> {
        session::validate_session(self, request).await
    }

    async fn get_customer(
        &self,
        request: Request<GetCustomerRequest>,
    ) -> Result<Response<GetCustomerResponse>, Status> {
        customer::get_customer(self, request).await
    }

    async fn get_merchant(
        &self,
        request: Request<GetMerchantRequest>,
    ) -> Result<Response<GetMerchantResponse>, Status> {
        merchant::get_merchant(self, request).await
    }

    async fn get_operator(
        &self,
        request: Request<GetOperatorRequest>,
    ) -> Result<Response<GetOperatorResponse>, Status> {
        operator::get_operator(self, request).await
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        session::logout(self, request).await
    }
}

/// 成功时的业务状态。
fn ok_status() -> BusinessStatus {
    BusinessStatus {
        code: ApiCode::Ok as i32,
        message: "ok".to_owned(),
    }
}

/// 业务问题 → 响应体里的 `BusinessStatus`（gRPC 调用本身是 Ok）。
///
/// 返回 `None` = 技术问题，调用方应改走 `Err(Status)`（传输层）。
fn to_business_status(err: &ServiceError) -> Option<BusinessStatus> {
    match err.code() {
        ApiCode::Internal => None,
        code => Some(BusinessStatus {
            code: code as i32,
            message: err.public_message().to_owned(),
        }),
    }
}

/// 技术问题 → `tonic::Status`（传输层失败，gateway 映射成 4xx/5xx）。
///
/// 业务错误绝不走这里 —— 它们由 `to_business_status` 放进响应体。
fn to_transport_status(err: ServiceError) -> Status {
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
