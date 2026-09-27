use std::env;

pub struct Config {
    pub postgres_url: String,
    pub valkey_url: String,
}

impl Config {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();
        Self {
            postgres_url: env::var("POSTGRES_URL").expect("缺少环境变量_POSTGRES_URL"),
            valkey_url: env::var("VALKEY_URL").expect("缺少环境变量_VALKEY_URL"),
        }
    }
}
