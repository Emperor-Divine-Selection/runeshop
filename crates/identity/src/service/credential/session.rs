use common::role::Role;
use redis::{AsyncCommands, aio::ConnectionManager};
use sea_orm::sqlx::types::Uuid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("[Technical] {0}")]
    ValKey(#[from] redis::RedisError),
}

pub struct SessionStore {
    connection: ConnectionManager,
    ttl_seconds: u64,
}

impl SessionStore {
    pub async fn connect(url: &str, ttl_seconds: u64) -> Result<Self, SessionError> {
        let client = redis::Client::open(url)?;
        let connection = client.get_connection_manager().await?;
        Ok(Self {
            connection,
            ttl_seconds,
        })
    }

    pub async fn save(&self, token: &str, role: Role, user_id: &Uuid) -> Result<(), SessionError> {
        let mut connection = self.connection.clone();
        let key = format!("session:{token}");
        let role = role.as_str();
        let value = format!("{role}:{user_id}");
        let _: () = connection.set_ex(key, value, self.ttl_seconds).await?;
        Ok(())
    }

    pub async fn load(&self, token: &str) -> Result<Option<String>, SessionError> {
        let mut connection = self.connection.clone();
        let key = format!("session:{token}");
        let value: Option<String> = connection.get(key).await?;
        Ok(value)
    }

    pub async fn delete(&self, token: &str) -> Result<(), SessionError> {
        let mut connection = self.connection.clone();
        let key = format!("session:{token}");
        let _: i64 = connection.del(key).await?;
        Ok(())
    }
}
