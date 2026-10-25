use fred::prelude::*;
use sea_orm::prelude::Uuid;
use std::time::Duration;

use crate::cache::error::CacheError;
use crate::cache::kv;

/// token 存活时长：24小时。service 传了具体值就用它
const DEFAULT_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// key 前缀。将来还有 user:1:xxx 等，靠前缀区分避免撞车
const KEY_PREFIX: &str = "token:";

/// token 属于哪一端。登录时写进去，提取器按端校验
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Customer,
    Merchant,
    Backoffice,
}

impl TokenKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Customer => "customer",
            Self::Merchant => "merchant",
            Self::Backoffice => "backoffice",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "customer" => Some(Self::Customer),
            "merchant" => Some(Self::Merchant),
            "backoffice" => Some(Self::Backoffice),
            _ => None,
        }
    }
}

/// 拼出 Valkey 里的完整 key
fn key(token: &str) -> String {
    format!("{KEY_PREFIX}{token}")
}

/// 生成 32 字节随机数 -> hex 编码成 64 字符
pub fn generate() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    hex::encode(bytes)
}

/// 存 token → `端:user_id`；`ttl` 传 `None` 时用默认 24 小时
pub async fn save(
    client: &Client,
    token: &str,
    kind: TokenKind,
    user_id: Uuid,
    ttl: Option<Duration>,
) -> Result<(), CacheError> {
    let value = format!("{}:{}", kind.as_str(), user_id);
    kv::set_ex(
        client,
        &key(token),
        value,
        Some(ttl.unwrap_or(DEFAULT_TTL)),
    )
    .await
}

/// 取 token 对应的「端 + user_id」；不存在、已过期、或格式无效都返 `Ok(None)`
pub async fn get(client: &Client, token: &str) -> Result<Option<(TokenKind, Uuid)>, CacheError> {
    let Some(raw) = kv::get::<String>(client, &key(token)).await? else {
        return Ok(None);
    };
    let Some((kind, user_id)) = raw.split_once(':') else {
        return Ok(None);
    };
    let Some(kind) = TokenKind::parse(kind) else {
        return Ok(None);
    };
    let Ok(user_id) = user_id.parse() else {
        return Ok(None);
    };
    Ok(Some((kind, user_id)))
}

/// 注销：删掉 token
pub async fn delete(client: &Client, token: &str) -> Result<(), CacheError> {
    kv::del(client, &key(token)).await
}
