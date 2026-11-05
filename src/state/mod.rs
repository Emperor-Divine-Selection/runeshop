use std::sync::Arc;

use fred::clients::Client;
use object_store::ObjectStore;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub cache: Client,
    pub storage: Arc<dyn ObjectStore>,
    /// 浏览器访问对象的前缀；数据库里只存 key，拼 URL 的规则集中在这里
    pub storage_public_url: String,
}
