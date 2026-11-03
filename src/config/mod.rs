use std::env;

pub struct Config {
    pub postgres_url: String,
    pub valkey_url: String,
    pub bootstrap_admin_username: String,
    pub bootstrap_admin_email: String,
    /// 没配就跳过创建（生产环境靠这条关掉）
    pub bootstrap_admin_password: Option<String>,
    pub storage_endpoint: String,
    pub storage_region: String,
    pub storage_bucket: String,
    pub storage_access_key: String,
    pub storage_secret_key: String,
    /// 浏览器访问图片的前缀（Garage 的 website 端口，和 S3 端口不是同一个）
    pub storage_public_url: String,
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
            storage_endpoint: env::var("GARAGE_ENDPOINT").expect("缺少环境变量_GARAGE_ENDPOINT"),
            storage_region: env::var("GARAGE_REGION").expect("缺少环境变量_GARAGE_REGION"),
            storage_bucket: env::var("GARAGE_BUCKET").expect("缺少环境变量_GARAGE_BUCKET"),
            storage_access_key: env::var("GARAGE_ACCESS_KEY")
                .expect("缺少环境变量_GARAGE_ACCESS_KEY"),
            storage_secret_key: env::var("GARAGE_SECRET_KEY")
                .expect("缺少环境变量_GARAGE_SECRET_KEY"),
            storage_public_url: env::var("GARAGE_PUBLIC_URL")
                .expect("缺少环境变量_GARAGE_PUBLIC_URL"),
        }
    }
}
