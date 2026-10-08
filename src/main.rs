mod cache;
mod config;
mod handler;
mod model;
mod router;
mod service;
mod state;
mod store;

use config::Config;
use fred::{
    interfaces::ClientLike,
    prelude::{Client as ValKeyClent, Config as ValKeyConfig},
};
use sea_orm::Database;
use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = Config::new();
    let db = Database::connect(&cfg.postgres_url)
        .await
        .expect("数据库连接失败");

    let cache_config = ValKeyConfig::from_url(&cfg.valkey_url).expect("未找到缓存地址");
    let cache = ValKeyClent::new(cache_config, None, None, None);
    // connect() 只是「后台启动连接任务」，返回的 ConnectHandle 要等连接【关闭】才 resolve，
    // 所以绝对不能 await 它 —— 会永久挂起。wait_for_connect() 才是「等连上」。
    cache.connect();
    cache
        .wait_for_connect()
        .await
        .expect("缓存连接失败");

    let state = AppState { db, cache };
    let app = router::app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("绑定端口失败");

    println!("listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.expect("服务启动失败");
}
