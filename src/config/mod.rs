use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub postgres_url: String,
    pub valkey_url: String,
}

impl Config {
    pub fn new() -> Config {
        dotenv().ok();
        return Config {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT 必须是数字"),
            postgres_url: env::var("POSTGRES_URL").expect("缺少 POSTGRES_URL 字段"),
            valkey_url: env::var("VALKEY_URL").expect("缺少 VALKEY_URL 字段"),
        };
    }
}
