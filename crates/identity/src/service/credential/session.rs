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

pub struct SessionIdentity {
    pub role: Role,
    pub user_id: Uuid,
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

    pub async fn load(&self, token: &str) -> Result<Option<SessionIdentity>, SessionError> {
        let mut connection = self.connection.clone();
        let key = format!("session:{token}");
        let value: Option<String> = connection.get(key).await?;
        let Some(token) = value else {
            return Ok(None);
        };
        let Some((role, user_id)) = token.split_once(":") else {
            tracing::warn!(reason = "malformed_session_value", "session value 解析失败");
            return Ok(None);
        };
        let Some(role) = Role::parse(role) else {
            tracing::warn!(reason = "unknown_role", "session value 里的 role 不认识");
            return Ok(None);
        };
        let Ok(user_id) = Uuid::parse_str(user_id) else {
            tracing::warn!(reason = "bad_user_id", "session value 里的 user_id 不合法");
            return Ok(None);
        };
        Ok(Some(SessionIdentity { role, user_id }))
    }

    pub async fn delete(&self, token: &str) -> Result<(), SessionError> {
        let mut connection = self.connection.clone();
        let key = format!("session:{token}");
        let _: i64 = connection.del(key).await?;
        Ok(())
    }
}
