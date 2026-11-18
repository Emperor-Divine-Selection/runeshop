mod error;
mod grpc;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/healthz", get(|| async { "ok" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    tracing::info!("gateway 监听 8080");
    axum::serve(listener, app).await.unwrap();
}
