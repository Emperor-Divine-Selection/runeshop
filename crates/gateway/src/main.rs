mod error;
mod grpc;
mod handler;
mod response;
mod router;
mod state;

use std::error::Error;

use common::telemetry::log_init;
use tonic::transport::Channel;

use crate::grpc::identity_service_client::IdentityServiceClient;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    log_init("gateway");

    let session_ttl_seconds: u64 = std::env::var("SESSION_TTL_SECONDS")?.parse()?;

    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;

    let identity = IdentityServiceClient::new(channel);

    let state = AppState {
        identity,
        session_ttl_seconds,
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;

    tracing::info!("gateway 监听 8080");
    axum::serve(listener, router::app(state)).await?;
    Ok(())
}
