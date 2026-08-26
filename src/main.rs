mod config;

use config::Config;

#[tokio::main]
async fn main() {
    let cfg = Config::new();
    println!("这是配置：{:?}", cfg);
}
