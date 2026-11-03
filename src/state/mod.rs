use std::sync::Arc;

use fred::clients::Client;
use object_store::ObjectStore;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub cache: Client,
    pub storage: Arc<dyn ObjectStore>,
}
