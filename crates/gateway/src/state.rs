use tonic::transport::Channel;

use crate::grpc::identity_service_client::IdentityServiceClient;

#[derive(Clone)]
pub struct AppState {
    pub identity: IdentityServiceClient<Channel>,
}
