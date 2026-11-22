use tonic::transport::Channel;

use crate::grpc::identity_service_client::IdentityServiceClient;

#[derive(Clone)]
pub struct GatewayState {
    pub identity: IdentityServiceClient<Channel>,
    /// 与 identity 的 Valkey TTL 同源（都读 SESSION_TTL_SECONDS）。
    /// Cookie 的 `Max-Age` 和 /auth/login 的语义都由它决定（§4.2）。
    pub session_ttl_seconds: u64,
}
