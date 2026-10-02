mod cache;
mod config;
mod handler;
mod model;
mod router;
mod service;
mod store;

use config::Config;
use sea_orm::Database;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = Config::new();
    let db = Database::connect(&cfg.postgres_url)
        .await
        .expect("数据库连接失败");

    let app = router::app(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("绑定端口失败");

    println!("listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.expect("服务启动失败");
}
