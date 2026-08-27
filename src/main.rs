mod config;
mod model;

use config::Config;
use sea_orm::{Database, EntityTrait, PaginatorTrait};

#[tokio::main]
async fn main() {
    let cfg = Config::new();
    let db = Database::connect(&cfg.postgres_url)
        .await
        .expect("数据库连接失败");

    let count = model::users::Entity::find()
        .count(&db)
        .await
        .expect("查询失败");

    println!("users 表当前有 {} 行", count);
}
