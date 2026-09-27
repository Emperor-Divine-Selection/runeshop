mod config;
mod model;
mod store;

use config::Config;
use sea_orm::{Database, prelude::*};

#[derive(Clone, Debug)]
struct AppState {
    db: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    let cfg = Config::new();
    let db = Database::connect(&cfg.postgres_url).await;

    println!("Hello, world!");
}
