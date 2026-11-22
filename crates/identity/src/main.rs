mod grpc;
mod model;
mod service;
mod store;

use common::telemetry::log_init;
use sea_orm::Database;
use std::error::Error;
use tonic::transport::Server;

use crate::grpc::IdentityState;
use crate::grpc::identity_service_server::IdentityServiceServer;
use crate::service::credential::session::SessionStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    log_init("identity");

    let database_url = std::env::var("POSTGRES_URL_IDENTITY")?;
    let valkey_url = std::env::var("VALKEY_URL")?;
    let ttl_seconds = std::env::var("SESSION_TTL_SECONDS")?;

    let db = Database::connect(&database_url).await?;
    let ttl = ttl_seconds.parse::<u64>()?;
    let sessions = SessionStore::connect(&valkey_url, ttl).await?;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/identity_descriptor.bin"
        )))
        .build_v1()?;

    let addr: std::net::SocketAddr = "0.0.0.0:50051".parse()?;

    Server::builder()
        .add_service(IdentityServiceServer::new(IdentityState { db, sessions }))
        .add_service(reflection)
        .serve(addr)
        .await?;

    Ok(())
}
