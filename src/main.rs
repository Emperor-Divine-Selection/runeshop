mod config;
mod handler;
mod model;
mod service;
mod store;

use axum::Router;
use axum::routing::post;
use config::Config;
use sea_orm::Database;
use tower_http::trace::TraceLayer;

use handler::register::register_handler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = Config::new();
    let db = Database::connect(&cfg.postgres_url)
        .await
        .expect("数据库连接失败");

    let app = Router::new()
        .route("/users", post(register_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("绑定端口失败");

    println!("listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.expect("服务启动失败");
}
