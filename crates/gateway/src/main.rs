mod error;
mod grpc;
mod handler;
mod response;
mod router;
mod state;

use std::error::Error;

use crate::grpc::identity_service_client::IdentityServiceClient;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let channel = tonic::transport::Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;

    let identity = IdentityServiceClient::new(channel);

    let state = AppState { identity };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    tracing::info!("gateway 监听 8080");
    axum::serve(listener, router::app(state)).await?;
    Ok(())
}
