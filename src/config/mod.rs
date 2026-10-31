use std::env;

pub struct Config {
    pub postgres_url: String,
    pub valkey_url: String,
    pub bootstrap_admin_username: String,
    pub bootstrap_admin_email: String,
    /// 没配就跳过创建（生产环境靠这条关掉）
    pub bootstrap_admin_password: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();
        Self {
            postgres_url: env::var("POSTGRES_URL").expect("缺少环境变量_POSTGRES_URL"),
            valkey_url: env::var("VALKEY_URL").expect("缺少环境变量_VALKEY_URL"),
            bootstrap_admin_username: env::var("BOOTSTRAP_ADMIN_USERNAME")
                .unwrap_or_else(|_| "admin".into()),
            bootstrap_admin_email: env::var("BOOTSTRAP_ADMIN_EMAIL")
                .unwrap_or_else(|_| "admin@runeshop.local".into()),
            bootstrap_admin_password: env::var("BOOTSTRAP_ADMIN_PASSWORD").ok(),
        }
    }
}
