mod cookie;
mod error;
mod grpc;
mod handler;
mod middleware;
mod response;
mod router;
mod state;

use std::error::Error;
use std::time::Duration;

use common::telemetry::log_init;
use tonic::transport::Channel;

use crate::grpc::identity_service_client::IdentityServiceClient;
use crate::state::GatewayState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    log_init("gateway");

    let session_ttl_seconds: u64 = std::env::var("SESSION_TTL_SECONDS")?.parse()?;

    // 没有超时的话：identity 活着但卡住（例如数据库连接不上），gateway 会无限期等待，
    // 前端只能一直转圈，比直接 500 更难排查
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .timeout(Duration::from_secs(3))
        .connect()
        .await?;

    let identity = IdentityServiceClient::new(channel);

    let state = GatewayState {
        identity,
        session_ttl_seconds,
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;

    tracing::info!("gateway 监听 8080");
    axum::serve(listener, router::app(state)).await?;
    Ok(())
}
